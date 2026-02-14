fn main() -> anyhow::Result<()> {
    // fuck you windows :(
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-arg=/FORCE:MULTIPLE");
        println!("cargo:rustc-link-arg=/NODEFAULTLIB:libcmt.lib");
    }

    generate_c_header()?;

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=../../include/dropbear.h");
    Ok(())
}

fn generate_c_header() -> anyhow::Result<()> {
    let manifest_dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| anyhow::anyhow!("Failed to locate workspace root"))?;

    let output_path = workspace_root.join("include").join("dropbear.h");
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let src_dir = manifest_dir.join("src");
    let mut functions = Vec::new();
    let mut structs = std::collections::HashMap::new();
    collect_exported_functions(&src_dir, &mut functions, &mut structs)?;

    let header = render_header(&functions, &structs);
    std::fs::write(&output_path, header)?;

    Ok(())
}

#[derive(Debug)]
struct ExportedFunction {
    name: String,
    params: Vec<ExportParam>,
    out_type: Option<String>,
}

#[derive(Debug, Clone)]
struct ExportParam {
    name: String,
    ty: String,
}

fn collect_exported_functions(
    dir: &std::path::Path,
    out: &mut Vec<ExportedFunction>,
    structs: &mut std::collections::HashMap<String, StructDef>,
) -> anyhow::Result<()> {
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                collect_exported_functions(&path, out, structs)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                let content = std::fs::read_to_string(&path)?;
                let file = syn::parse_file(&content)?;
                extract_exports_from_file(&file, out, structs)?;
                extract_structs_from_file(&file, structs)?;
            }
        }
    }
    Ok(())
}

fn extract_exports_from_file(
    file: &syn::File,
    out: &mut Vec<ExportedFunction>,
    _structs: &mut std::collections::HashMap<String, StructDef>,
) -> anyhow::Result<()> {
    for item in &file.items {
        if let syn::Item::Fn(func) = item {
            if let Some(export) = parse_export_attr(&func.attrs)? {
                if export.c.is_none() {
                    continue;
                }

                let c_name = export.c.and_then(|c| c.name).unwrap_or_else(|| {
                    format!("dropbear_{}", func.sig.ident)
                });

                let (out_type, out_is_optional) = extract_result_type(&func.sig.output)?;
                let mut params = Vec::new();
                for input in &func.sig.inputs {
                    if let syn::FnArg::Typed(pat_ty) = input {
                        let (define_ty, is_entity) = extract_arg_markers(&pat_ty.attrs);
                        let name = match &*pat_ty.pat {
                            syn::Pat::Ident(ident) => ident.ident.to_string(),
                            _ => continue,
                        };

                        let ty = if is_entity {
                            "uint64_t".to_string()
                        } else if let Some(define_ty) = define_ty {
                            type_to_c(&define_ty, true)
                        } else {
                            type_to_c(&pat_ty.ty, false)
                        };

                        params.push(ExportParam { name, ty });
                    }
                }

                if out_type.is_some() {
                    let out_ty = out_type.clone().unwrap();
                    params.push(ExportParam { name: "out0".to_string(), ty: format!("{}*", out_ty) });
                    if out_is_optional {
                        params.push(ExportParam { name: "out0_present".to_string(), ty: "bool*".to_string() });
                    }
                }

                out.push(ExportedFunction {
                    name: c_name,
                    params,
                    out_type,
                });
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct StructDef {
    name: String,
    fields: Vec<ExportParam>,
    is_repr_c: bool,
}

fn extract_structs_from_file(
    file: &syn::File,
    structs: &mut std::collections::HashMap<String, StructDef>,
) -> anyhow::Result<()> {
    for item in &file.items {
        if let syn::Item::Struct(strct) = item {
            let name = strct.ident.to_string();
            let is_repr_c = has_repr_c(&strct.attrs);
            let mut fields = Vec::new();

            if let syn::Fields::Named(named) = &strct.fields {
                for field in &named.named {
                    let field_name = field.ident.as_ref().map(|i| i.to_string()).unwrap_or_else(|| "field".to_string());
                    let ty = type_to_c(&field.ty, false);
                    fields.push(ExportParam { name: field_name, ty });
                }
            }

            structs.insert(name.clone(), StructDef { name, fields, is_repr_c });
        }
    }

    Ok(())
}

#[derive(Default)]
struct ExportAttr {
    c: Option<CArgs>,
}

#[derive(Default)]
struct CArgs {
    name: Option<String>,
}

fn parse_export_attr(attrs: &[syn::Attribute]) -> anyhow::Result<Option<ExportAttr>> {
    for attr in attrs {
        let path = attr.path();
        let is_export = path.segments.last().map(|s| s.ident == "export").unwrap_or(false)
            || (path.segments.iter().any(|s| s.ident == "dropbear_macro")
                && path.segments.last().map(|s| s.ident == "export").unwrap_or(false));
        if !is_export {
            continue;
        }

        let meta = &attr.meta;
        if let syn::Meta::List(list) = meta {
            let args = syn::parse2::<ExportArgs>(list.tokens.clone())?;
            return Ok(Some(ExportAttr { c: args.c }));
        }
    }

    Ok(None)
}

struct ExportArgs {
    c: Option<CArgs>,
}

impl syn::parse::Parse for ExportArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let items = syn::punctuated::Punctuated::<ExportItem, syn::Token![,]>::parse_terminated(input)?;
        let mut args = ExportArgs { c: None };

        for item in items {
            if let ExportItem::C(c) = item {
                args.c = Some(c);
            }
        }

        Ok(args)
    }
}

enum ExportItem {
    C(CArgs),
    Kotlin,
}

impl syn::parse::Parse for ExportItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        if ident == "c" {
            let args = if input.peek(syn::token::Paren) {
                let content;
                syn::parenthesized!(content in input);
                let mut c_args = CArgs::default();
                while !content.is_empty() {
                    let key: syn::Ident = content.parse()?;
                    content.parse::<syn::Token![=]>()?;
                    let value: syn::LitStr = content.parse()?;
                    if key == "name" {
                        c_args.name = Some(value.value());
                    }
                    if content.peek(syn::Token![,]) {
                        content.parse::<syn::Token![,]>()?;
                    }
                }
                c_args
            } else {
                CArgs::default()
            };
            return Ok(ExportItem::C(args));
        }

        if ident == "kotlin" {
            if input.peek(syn::token::Paren) {
                let content;
                syn::parenthesized!(content in input);
                while !content.is_empty() {
                    let _key: syn::Ident = content.parse()?;
                    content.parse::<syn::Token![=]>()?;
                    let _value: syn::LitStr = content.parse()?;
                    if content.peek(syn::Token![,]) {
                        content.parse::<syn::Token![,]>()?;
                    }
                }
            }
            return Ok(ExportItem::Kotlin);
        }

        Err(syn::Error::new(ident.span(), "Expected c or kotlin"))
    }
}

fn extract_arg_markers(attrs: &[syn::Attribute]) -> (Option<syn::Type>, bool) {
    let mut define_ty: Option<syn::Type> = None;
    let mut is_entity = false;
    for attr in attrs {
        let path = attr.path();
        let ident = path.segments.last().map(|s| s.ident.to_string()).unwrap_or_default();
        if ident == "define" {
            if let Ok(ty) = attr.parse_args::<syn::Type>() {
                define_ty = Some(ty);
            }
        }
        if ident == "entity" {
            is_entity = true;
        }
    }
    (define_ty, is_entity)
}

fn extract_result_type(output: &syn::ReturnType) -> anyhow::Result<(Option<String>, bool)> {
    let ty = match output {
        syn::ReturnType::Type(_, ty) => ty,
        syn::ReturnType::Default => return Ok((None, false)),
    };

    let inner = match &**ty {
        syn::Type::Path(path) => {
            let last = path.path.segments.last().ok_or_else(|| anyhow::anyhow!("Invalid return type"))?;
            if last.ident != "DropbearNativeResult" {
                return Ok((None, false));
            }
            match &last.arguments {
                syn::PathArguments::AngleBracketed(args) => args.args.first().and_then(|a| match a {
                    syn::GenericArgument::Type(t) => Some(t.clone()),
                    _ => None,
                }).ok_or_else(|| anyhow::anyhow!("DropbearNativeResult missing type"))?,
                _ => return Ok((None, false)),
            }
        }
        _ => return Ok((None, false)),
    };

    if is_unit_type(&inner) {
        return Ok((None, false));
    }

    if let Some(opt_inner) = extract_option_inner(&inner) {
        return Ok((Some(type_to_c(&opt_inner, true)), true));
    }

    Ok((Some(type_to_c(&inner, true)), false))
}

fn extract_option_inner(ty: &syn::Type) -> Option<syn::Type> {
    if let syn::Type::Path(path) = ty {
        let last = path.path.segments.last()?;
        if last.ident != "Option" {
            return None;
        }
        if let syn::PathArguments::AngleBracketed(args) = &last.arguments {
            if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
                return Some(inner.clone());
            }
        }
    }
    None
}

fn is_unit_type(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Tuple(tuple) if tuple.elems.is_empty())
}

fn type_to_c(ty: &syn::Type, for_output: bool) -> String {
    if let syn::Type::Reference(reference) = ty {
        let inner = type_to_c(&reference.elem, for_output);
        let mutability = if reference.mutability.is_some() { "" } else { "const " };
        return format!("{}{}*", mutability, inner);
    }
    if let syn::Type::Path(path) = ty {
        let ident = path.path.segments.last().map(|s| s.ident.to_string()).unwrap_or_else(|| "void".to_string());
        return match ident.as_str() {
            "i8" => "int8_t".to_string(),
            "u8" => "uint8_t".to_string(),
            "i16" => "int16_t".to_string(),
            "u16" => "uint16_t".to_string(),
            "i32" => "int32_t".to_string(),
            "u32" => "uint32_t".to_string(),
            "i64" => "int64_t".to_string(),
            "u64" => "uint64_t".to_string(),
            "isize" => "intptr_t".to_string(),
            "usize" => "uintptr_t".to_string(),
            "f32" => "float".to_string(),
            "f64" => "double".to_string(),
            "bool" => "bool".to_string(),
            "String" => {
                if for_output {
                    "char*".to_string()
                } else {
                    "const char*".to_string()
                }
            }
            _ => ident,
        };
    }

    "void".to_string()
}

fn render_header(
    funcs: &[ExportedFunction],
    structs: &std::collections::HashMap<String, StructDef>,
) -> String {
    let mut out = String::new();
    out.push_str("#ifndef DROPBEAR_H\n");
    out.push_str("#define DROPBEAR_H\n\n");
    out.push_str("#include <stdbool.h>\n");
    out.push_str("#include <stdint.h>\n\n");

    let mut needed = std::collections::HashSet::new();
    for func in funcs {
        if let Some(out_ty) = &func.out_type {
            if is_custom_type(out_ty) {
                needed.insert(out_ty.clone());
            }
        }
    }

    let mut emitted = std::collections::HashSet::new();
    for ty in needed {
        emit_structs_recursive(&ty, structs, &mut emitted, &mut out);
    }

    for func in funcs {
        let params = if func.params.is_empty() {
            "void".to_string()
        } else {
            func.params.iter()
                .map(|p| format!("{} {}", p.ty, p.name))
                .collect::<Vec<_>>()
                .join(", ")
        };
        out.push_str(&format!("int32_t {}({});\n", func.name, params));
    }

    out.push_str("\n#endif /* DROPBEAR_H */\n");
    out
}

fn has_repr_c(attrs: &[syn::Attribute]) -> bool {
    for attr in attrs {
        if !attr.path().is_ident("repr") {
            continue;
        }
        if let syn::Meta::List(list) = &attr.meta {
            let tokens = list.tokens.to_string();
            if tokens.contains("C") || tokens.contains("transparent") {
                return true;
            }
        }
    }
    false
}

fn is_custom_type(ty: &str) -> bool {
    if ty.ends_with('*') {
        return false;
    }
    !is_builtin_c_type(ty)
}

fn is_builtin_c_type(ty: &str) -> bool {
    matches!(
        ty,
        "int8_t" | "uint8_t" | "int16_t" | "uint16_t" | "int32_t" | "uint32_t" |
        "int64_t" | "uint64_t" | "intptr_t" | "uintptr_t" | "bool" | "float" |
        "double" | "char" | "char*" | "const char*"
    )
}

fn emit_structs_recursive(
    name: &str,
    structs: &std::collections::HashMap<String, StructDef>,
    emitted: &mut std::collections::HashSet<String>,
    out: &mut String,
) {
    if emitted.contains(name) {
        return;
    }

    if let Some(def) = structs.get(name) {
        for field in &def.fields {
            if is_custom_type(&field.ty) {
                emit_structs_recursive(&field.ty, structs, emitted, out);
            }
        }

        if !def.is_repr_c {
            out.push_str("// NOTE: type is not #[repr(C)] in Rust; ensure C ABI safety.\n");
        }

        out.push_str(&format!("typedef struct {} {{\n", def.name));
        for field in &def.fields {
            out.push_str(&format!("    {} {};\n", field.ty, field.name));
        }
        out.push_str(&format!("}} {};\n\n", def.name));
        emitted.insert(def.name.clone());
    } else {
        out.push_str(&format!("typedef struct {} {};// opaque\n\n", name, name));
        emitted.insert(name.to_string());
    }
}
