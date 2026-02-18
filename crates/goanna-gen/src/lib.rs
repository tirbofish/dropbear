pub fn generate_c_header() -> anyhow::Result<()> {
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
    let mut enums = Vec::new();
    collect_exported_functions(&src_dir, &mut functions, &mut structs, &mut enums)?;

    functions.sort_by(|a, b| a.name.cmp(&b.name));
    enums.sort_by(|a, b| a.name.cmp(&b.name));

    let header = render_header(&functions, &structs, &enums);
    if let Ok(existing) = std::fs::read_to_string(&output_path) {
        if existing == header {
            return Ok(());
        }
    }

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
    enums: &mut Vec<EnumDef>,
) -> anyhow::Result<()> {
    if dir.is_dir() {
        let mut entries = std::fs::read_dir(dir)?
            .filter_map(Result::ok)
            .collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.path());

        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                collect_exported_functions(&path, out, structs, enums)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                let content = std::fs::read_to_string(&path)?;
                let file = syn::parse_file(&content)?;
                extract_exports_from_file(&file, &path, dir, out, structs)?;
                extract_structs_from_file(&file, structs)?;
                extract_repr_c_enums_from_file(&file, enums)?;
            }
        }
    }
    Ok(())
}

fn extract_exports_from_file(
    file: &syn::File,
    file_path: &std::path::Path,
    src_root: &std::path::Path,
    out: &mut Vec<ExportedFunction>,
    _structs: &mut std::collections::HashMap<String, StructDef>,
) -> anyhow::Result<()> {
    let module_path = module_path_from_file(file_path, src_root);
    for item in &file.items {
        if let syn::Item::Fn(func) = item {
            if let Some(export) = parse_export_attr(&func.attrs)? {
                if export.c.is_none() {
                    continue;
                }

                let c_name = export.c.and_then(|c| c.name).unwrap_or_else(|| {
                    if let Some(path) = module_path.as_deref() {
                        if !path.is_empty() {
                            return format!("dropbear_{}_{}", path, func.sig.ident);
                        }
                    }
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
                        } else if is_object_input(&pat_ty.ty) {
                            object_input_to_c(&pat_ty.ty)
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

fn module_path_from_file(
    file_path: &std::path::Path,
    src_root: &std::path::Path,
) -> Option<String> {
    let rel = file_path.strip_prefix(src_root).ok()?;
    let mut parts: Vec<String> = rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect();

    let file = parts.pop()?;
    let stem = file.strip_suffix(".rs").unwrap_or(&file);

    if stem != "mod" && stem != "lib" && stem != "main" {
        parts.push(stem.to_string());
    }

    if parts.is_empty() {
        return None;
    }

    let joined = parts.join("_");
    Some(joined.replace('-', "_"))
}

#[derive(Debug, Clone)]
struct StructDef {
    name: String,
    fields: Vec<ExportParam>,
    is_repr_c: bool,
}

#[derive(Debug)]
struct EnumDef {
    name: String,
    variants: Vec<EnumVariantDef>,
}

#[derive(Debug)]
struct EnumVariantDef {
    name: String,
    fields: Vec<ExportParam>,
}

fn extract_repr_c_enums_from_file(
    file: &syn::File,
    enums: &mut Vec<EnumDef>,
) -> anyhow::Result<()> {
    for item in &file.items {
        if let syn::Item::Enum(enm) = item {
            if !has_repr_c_enum_attr(&enm.attrs) {
                continue;
            }
            let mut variants = Vec::new();
            for variant in &enm.variants {
                let mut fields = Vec::new();
                match &variant.fields {
                    syn::Fields::Named(named) => {
                        for field in &named.named {
                            let name = field
                                .ident
                                .as_ref()
                                .map(|i| i.to_string())
                                .unwrap_or_else(|| "field".to_string());
                            let ty = type_to_c(&field.ty, false);
                            fields.push(ExportParam { name, ty });
                        }
                    }
                    syn::Fields::Unnamed(unnamed) => {
                        for (idx, field) in unnamed.unnamed.iter().enumerate() {
                            let name = format!("_{}", idx);
                            let ty = type_to_c(&field.ty, false);
                            fields.push(ExportParam { name, ty });
                        }
                    }
                    syn::Fields::Unit => {}
                }
                variants.push(EnumVariantDef { name: variant.ident.to_string(), fields });
            }
            enums.push(EnumDef { name: enm.ident.to_string(), variants });
        }
    }
    Ok(())
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
    if let Some(inner) = extract_option_inner(ty) {
        let inner_c = type_to_c(&inner, true);
        let mutability = if for_output { "" } else { "const " };
        return format!("{}{}*", mutability, inner_c);
    }
    if let Some(inner) = vec_inner_type(ty) {
        return array_struct_name_from_type(&inner);
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
            "usize" => "size_t".to_string(),
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

fn vec_inner_type(ty: &syn::Type) -> Option<syn::Type> {
    if let syn::Type::Path(path) = ty {
        let last = path.path.segments.last()?;
        if last.ident != "Vec" {
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

fn type_name_from_type(ty: &syn::Type) -> Option<String> {
    if let syn::Type::Path(path) = ty {
        return path.path.segments.last().map(|s| s.ident.to_string());
    }
    None
}

fn array_struct_name_from_type(ty: &syn::Type) -> String {
    if let Some(inner) = vec_inner_type(ty) {
        let inner_name = array_struct_name_from_type(&inner);
        return format!("{}Array", inner_name);
    }

    let name = type_name_from_type(ty).unwrap_or_else(|| "Unknown".to_string());
    format!("{}Array", name)
}

fn render_header(
    funcs: &[ExportedFunction],
    structs: &std::collections::HashMap<String, StructDef>,
    enums: &[EnumDef],
) -> String {
    let mut out = String::new();
    out.push_str("// Machine generated header bindings by goanna-gen.\n");
    out.push_str("// DO NOT EDIT UNLESS YOU KNOW WHAT YOU ARE DOING (it will get regenerated anyways with a modification to eucalyptus-core/src).\n");
    out.push_str("// Licensed under MIT or Apache 2.0 depending on your mood.\n");
    out.push_str("// part of the dropbear project, by tirbofish\n\n");
    out.push_str("#ifndef DROPBEAR_H\n");
    out.push_str("#define DROPBEAR_H\n\n");
    out.push_str("#include <stdbool.h>\n");
    out.push_str("#include <stdint.h>\n\n");
    out.push_str("#include <stddef.h>\n\n");

    let mut needed = std::collections::HashSet::new();
    for func in funcs {
        if let Some(out_ty) = &func.out_type {
            if is_custom_type(out_ty) {
                needed.insert(out_ty.clone());
            } else if is_opaque_ptr_name(out_ty) {
                needed.insert(out_ty.clone());
            }
        }
        for param in &func.params {
            if is_opaque_ptr_name(&param.ty) {
                needed.insert(param.ty.clone());
            }
            if let Some(base) = base_type_name(&param.ty) {
                if is_custom_type(&base) {
                    needed.insert(base);
                } else if is_opaque_ptr_name(&base) {
                    needed.insert(base);
                }
            }
        }
    }

    let mut emitted = std::collections::HashSet::new();

    if needed.contains("AssetKind") {
        out.push_str("typedef enum AssetKind {\n");
        out.push_str("    AssetKind_Texture = 0,\n");
        out.push_str("    AssetKind_Model = 1,\n");
        out.push_str("} AssetKind;\n\n");
        emitted.insert("AssetKind".to_string());
    }

    for enm in enums {
        emit_repr_c_enum(enm, structs, &mut emitted, &mut out);
    }
    let mut needed_list: Vec<String> = needed.into_iter().collect();
    needed_list.sort();
    for ty in needed_list {
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

fn emit_repr_c_enum(
    enm: &EnumDef,
    structs: &std::collections::HashMap<String, StructDef>,
    emitted: &mut std::collections::HashSet<String>,
    out: &mut String,
) {
    if emitted.contains(&enm.name) {
        return;
    }

    let tag_name = format!("{}Tag", enm.name);
    let data_name = format!("{}Data", enm.name);
    let ffi_name = format!("{}Ffi", enm.name);

    for var in &enm.variants {
        for field in &var.fields {
            emit_field_type_deps(&field.ty, structs, emitted, out);
        }
    }

    out.push_str(&format!("typedef enum {} {{\n", tag_name));
    for (idx, var) in enm.variants.iter().enumerate() {
        out.push_str(&format!("    {}_{} = {},\n", tag_name, var.name, idx));
    }
    out.push_str(&format!("}} {};\n\n", tag_name));

    for var in &enm.variants {
        let struct_name = format!("{}{}", enm.name, var.name);
        out.push_str(&format!("typedef struct {} {{\n", struct_name));
        for field in &var.fields {
            out.push_str(&format!("    {} {};\n", field.ty, field.name));
        }
        out.push_str(&format!("}} {};\n\n", struct_name));
    }

    out.push_str(&format!("typedef union {} {{\n", data_name));
    for var in &enm.variants {
        let struct_name = format!("{}{}", enm.name, var.name);
        out.push_str(&format!("    {} {};\n", struct_name, var.name));
    }
    out.push_str(&format!("}} {};\n\n", data_name));

    out.push_str(&format!("typedef struct {} {{\n", ffi_name));
    out.push_str(&format!("    {} tag;\n", tag_name));
    out.push_str(&format!("    {} data;\n", data_name));
    out.push_str(&format!("}} {};\n\n", ffi_name));

    out.push_str(&format!("typedef {} {};\n\n", ffi_name, enm.name));
    emitted.insert(ffi_name);
    emitted.insert(enm.name.clone());
}

fn has_repr_c_enum_attr(attrs: &[syn::Attribute]) -> bool {
    for attr in attrs {
        let path = attr.path();
        if path.segments.last().map(|s| s.ident == "repr_c_enum").unwrap_or(false) {
            return true;
        }
        if path.segments.iter().any(|s| s.ident == "dropbear_macro")
            && path.segments.last().map(|s| s.ident == "repr_c_enum").unwrap_or(false)
        {
            return true;
        }
    }
    false
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
    if is_opaque_ptr_name(ty) {
        return false;
    }
    !is_builtin_c_type(ty)
}

fn base_type_name(ty: &str) -> Option<String> {
    let mut t = ty.trim().to_string();
    if t.starts_with("const ") {
        t = t.trim_start_matches("const ").trim().to_string();
    }
    if t.ends_with('*') {
        t = t.trim_end_matches('*').trim().to_string();
        return Some(t);
    }
    None
}

fn is_object_input(ty: &syn::Type) -> bool {
    let inner = peel_reference(ty);
    if is_string_type(inner) || is_primitive_type(inner) {
        return false;
    }
    !is_define_or_entity_like(inner)
}

fn object_input_to_c(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Reference(reference) => {
            let inner = type_to_c(&reference.elem, false);
            let mutability = if reference.mutability.is_some() { "" } else { "const " };
            format!("{}{}*", mutability, inner)
        }
        _ => type_to_c(ty, false),
    }
}

fn is_define_or_entity_like(_ty: &syn::Type) -> bool {
    false
}

fn is_string_type(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(path) if path.path.segments.last().map(|s| s.ident == "String").unwrap_or(false))
}

fn is_primitive_type(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(path) if {
        let ident = path.path.segments.last().map(|s| s.ident.to_string()).unwrap_or_default();
        matches!(
            ident.as_str(),
            "i8" | "u8" | "i16" | "u16" | "i32" | "u32" | "i64" | "u64" |
            "isize" | "usize" | "f32" | "f64" | "bool"
        )
    })
}

fn peel_reference<'a>(ty: &'a syn::Type) -> &'a syn::Type {
    if let syn::Type::Reference(reference) = ty {
        &reference.elem
    } else {
        ty
    }
}

fn is_builtin_c_type(ty: &str) -> bool {
    matches!(
        ty,
        "int8_t" | "uint8_t" | "int16_t" | "uint16_t" | "int32_t" | "uint32_t" |
        "int64_t" | "uint64_t" | "intptr_t" | "uintptr_t" | "bool" | "float" |
        "double" | "char" | "char*" | "const char*" | "void*" | "size_t"
    )
}

fn is_opaque_ptr_name(name: &str) -> bool {
    name.ends_with("Ptr")
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

    if is_opaque_ptr_name(name) {
        out.push_str(&format!("typedef void* {};\n\n", name));
        emitted.insert(name.to_string());
        return;
    }

    if let Some(def) = structs.get(name) {
        for field in &def.fields {
            emit_field_type_deps(&field.ty, structs, emitted, out);
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
    } else if is_array_type(name) {
        emit_array_struct(name, structs, emitted, out);
    } else {
        out.push_str(&format!("typedef struct {} {};// opaque\n\n", name, name));
        emitted.insert(name.to_string());
    }
}

fn emit_field_type_deps(
    field_ty: &str,
    structs: &std::collections::HashMap<String, StructDef>,
    emitted: &mut std::collections::HashSet<String>,
    out: &mut String,
) {
    if is_array_type(field_ty) {
        emit_array_struct(field_ty, structs, emitted, out);
        return;
    }

    if let Some(base) = base_type_name(field_ty) {
        if is_custom_type(&base) || is_opaque_ptr_name(&base) || is_array_type(&base) {
            emit_structs_recursive(&base, structs, emitted, out);
        }
        return;
    }

    if is_custom_type(field_ty) || is_opaque_ptr_name(field_ty) {
        emit_structs_recursive(field_ty, structs, emitted, out);
    }
}

fn is_array_type(ty: &str) -> bool {
    ty.ends_with("Array")
}

fn emit_array_struct(
    name: &str,
    structs: &std::collections::HashMap<String, StructDef>,
    emitted: &mut std::collections::HashSet<String>,
    out: &mut String,
) {
    if emitted.contains(name) {
        return;
    }
    let elem = name.trim_end_matches("Array");

    if is_array_type(elem) {
        emit_array_struct(elem, structs, emitted, out);
    }

    if is_builtin_rust_primitive(elem) {
        let c_elem = map_primitive_name(elem);
        if !emitted.contains(elem) {
            emitted.insert(elem.to_string());
        }
        out.push_str(&format!("typedef struct {} {{\n", name));
        out.push_str(&format!("    {}* values;\n", c_elem));
        out.push_str("    size_t length;\n");
        out.push_str("    size_t capacity;\n");
        out.push_str(&format!("}} {};\n\n", name));
        emitted.insert(name.to_string());
        return;
    }

    if !emitted.contains(elem) {
        if structs.contains_key(elem) {
            emit_structs_recursive(elem, structs, emitted, out);
        } else {
            out.push_str(&format!("typedef struct {} {};// opaque\n\n", elem, elem));
            emitted.insert(elem.to_string());
        }
    }

    out.push_str(&format!("typedef struct {} {{\n", name));
    out.push_str(&format!("    {}* values;\n", elem));
    out.push_str("    size_t length;\n");
    out.push_str("    size_t capacity;\n");
    out.push_str(&format!("}} {};\n\n", name));
    emitted.insert(name.to_string());
}

fn is_builtin_rust_primitive(name: &str) -> bool {
    matches!(
        name,
        "i8" | "u8" | "i16" | "u16" | "i32" | "u32" | "i64" | "u64" |
        "isize" | "usize" | "f32" | "f64" | "bool"
    )
}

fn map_primitive_name(name: &str) -> &'static str {
    match name {
        "i8" => "int8_t",
        "u8" => "uint8_t",
        "i16" => "int16_t",
        "u16" => "uint16_t",
        "i32" => "int32_t",
        "u32" => "uint32_t",
        "i64" => "int64_t",
        "u64" => "uint64_t",
        "isize" => "intptr_t",
        "usize" => "size_t",
        "f32" => "float",
        "f64" => "double",
        "bool" => "bool",
        _ => "void",
    }
}
