use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
    spanned::Spanned,
    DeriveInput, FnArg, GenericArgument, Ident, Item, ItemFn, ItemMod, ItemEnum, LitStr, PathArguments,
    ReturnType, Token, Type,
};

/// A `derive` macro that converts a struct to a usable [SerializableComponent].
///
/// You have to implement `serde::Serialize`, `serde::Deserialize` and `Clone` for the
/// struct to be usable (it will throw errors anyway).
///
/// # Usage
/// ```
/// use dropbear_macro::SerializableComponent;
///
/// #[derive(Serialize, Deserialize, Clone, SerializableComponent)] // required to be implemented
/// struct MyComponent {
///     value1: String,
///     value2: i32,
/// }
/// ```
#[proc_macro_derive(SerializableComponent)]
pub fn derive_component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let name_str = name.to_string();

    let expanded = quote! {
        #[typetag::serde]
        impl SerializableComponent for #name {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }

            fn clone_boxed(&self) -> Box<dyn SerializableComponent> {
                Box::new(self.clone())
            }

            fn type_name(&self) -> &'static str {
                #name_str
            }
        }
    };

    TokenStream::from(expanded)
}

/// Converts a module's functions into C API compatible functions.
/// 
/// Each function must require a return type of `DropbearNativeResult<T>`. If the return type
/// is not `DropbearNativeResult<T>`, it is recommended that you move it to another module (such as `super::shared`). 
/// 
/// A function like that of:
/// ```
/// pub fn dropbear_mesh_renderer_exists_for_entity(
///      world_ptr: *mut hecs::World,
///      entity_id: u64,
///  ) -> DropbearNativeResult<bool> {...}
/// ``` 
/// will get converted to something like:
/// ```
/// pub unsafe extern "C" fn dropbear_mesh_renderer_exists_for_entity(world_ptr: WorldPtr, entity_id: u64, out_result: *mut bool) -> i32 {...}
/// ```
#[proc_macro_attribute]
pub fn impl_c_api(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut module = parse_macro_input!(item as ItemMod);

    let mut new_content = Vec::new();

    if let Some((brace, content)) = module.content {
        for item in content {
            match item {
                Item::Fn(func) => {
                    new_content.push(Item::Fn(transform_function(func)));
                }
                _ => new_content.push(item),
            }
        }
        module.content = Some((brace, new_content));
    }

    TokenStream::from(quote! { #module })
}

struct ExportArgs {
    c: Option<CArgs>,
    kotlin: Option<KotlinArgs>,
}

#[derive(Default)]
struct CArgs {
    name: Option<String>,
}

struct KotlinArgs {
    class: String,
    func: String,
    jni_path: Option<syn::Path>,
}

enum ExportItem {
    C(CArgs),
    Kotlin(KotlinArgs),
}

impl Parse for ExportArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let items = syn::punctuated::Punctuated::<ExportItem, Token![,]>::parse_terminated(input)?;
        let mut args = ExportArgs { c: None, kotlin: None };

        for item in items {
            match item {
                ExportItem::C(c) => args.c = Some(c),
                ExportItem::Kotlin(k) => args.kotlin = Some(k),
            }
        }

        Ok(args)
    }
}

impl Parse for ExportItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        if ident == "c" {
            let args = if input.peek(syn::token::Paren) {
                let content;
                syn::parenthesized!(content in input);
                let mut c_args = CArgs::default();
                while !content.is_empty() {
                    let key: Ident = content.parse()?;
                    content.parse::<Token![=]>()?;
                    let value: LitStr = content.parse()?;
                    if key == "name" {
                        c_args.name = Some(value.value());
                    } else {
                        return Err(syn::Error::new(key.span(), "Unknown c(...) key"));
                    }
                    if content.peek(Token![,]) {
                        content.parse::<Token![,]>()?;
                    }
                }
                c_args
            } else {
                CArgs::default()
            };
            return Ok(ExportItem::C(args));
        }

        if ident == "kotlin" {
            let content;
            syn::parenthesized!(content in input);
            let mut class: Option<String> = None;
            let mut func: Option<String> = None;
            let mut jni_path: Option<syn::Path> = None;
            while !content.is_empty() {
                let key: Ident = content.parse()?;
                content.parse::<Token![=]>()?;
                let value: LitStr = content.parse()?;
                if key == "class" {
                    class = Some(value.value());
                } else if key == "func" {
                    func = Some(value.value());
                } else if key == "jni" {
                    jni_path = Some(syn::parse_str::<syn::Path>(&value.value())?);
                } else {
                    return Err(syn::Error::new(key.span(), "Unknown kotlin(...) key"));
                }
                if content.peek(Token![,]) {
                    content.parse::<Token![,]>()?;
                }
            }

            let class = class.ok_or_else(|| syn::Error::new(ident.span(), "kotlin(class = ...) is required"))?;
            let func = func.ok_or_else(|| syn::Error::new(ident.span(), "kotlin(func = ...) is required"))?;
            return Ok(ExportItem::Kotlin(KotlinArgs { class, func, jni_path }));
        }

        Err(syn::Error::new(ident.span(), "Expected c or kotlin(...)"))
    }
}

/// Exports a Rust function as C and/or Kotlin/JNI wrappers with minimal boilerplate.
///
/// The function must return `DropbearNativeResult<T>`. Use `#[dropbear_macro::define(...)]`
/// on pointer args and `#[dropbear_macro::entity]` for `hecs::Entity` args.
///
/// # Example
/// ```rust
/// #[dropbear_macro::export(
///     kotlin(class = "com.dropbear.DropbearEngineNative", func = "getEntity"),
///     c(name = "dropbear_engine_get_entity")
/// )]
/// fn get_entity(
///     #[dropbear_macro::define(crate::ptr::WorldPtr)]
///     world: &World,
///     label: String,
/// ) -> DropbearNativeResult<u64> {
///     shared::get_entity(&world, &label)
/// }
/// ```
#[proc_macro_attribute]
pub fn export(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as ExportArgs);
    let mut func = parse_macro_input!(item as ItemFn);

    let original_name = func.sig.ident.clone();
    let inner_name = Ident::new(&format!("__dropbear_export_inner_{}", original_name), original_name.span());
    let return_type = func.sig.output.clone();

    let result_inner = match extract_dropbear_result_inner_type(&return_type) {
        Ok(inner) => inner,
        Err(err) => return err.to_compile_error().into(),
    };

    let (is_option, option_inner) = extract_option_inner_type(&result_inner);

    let mut cleaned_inputs = Vec::new();
    let mut arg_specs = Vec::new();

    for input in func.sig.inputs.iter_mut() {
        match input {
            FnArg::Receiver(_) => {
                return syn::Error::new(input.span(), "export does not support methods").to_compile_error().into();
            }
            FnArg::Typed(pat_ty) => {
                let (define_ty, is_entity) = extract_arg_markers(&mut pat_ty.attrs);
                let (arg_name, arg_ty) = match &*pat_ty.pat {
                    syn::Pat::Ident(ident) => (ident.ident.clone(), (*pat_ty.ty).clone()),
                    _ => {
                        return syn::Error::new(pat_ty.span(), "export only supports identifier arguments")
                            .to_compile_error()
                            .into();
                    }
                };

                cleaned_inputs.push(FnArg::Typed(pat_ty.clone()));
                arg_specs.push(ArgSpec { name: arg_name, ty: arg_ty, define_ty, is_entity });
            }
        }
    }

    func.sig.ident = inner_name.clone();
    func.sig.inputs = syn::punctuated::Punctuated::from_iter(cleaned_inputs);

    let c_wrapper = match args.c {
        Some(c_args) => build_c_wrapper(&original_name, &inner_name, &arg_specs, &result_inner, is_option, option_inner.as_ref(), c_args),
        None => quote! {},
    };

    let kotlin_wrapper = match args.kotlin {
        Some(k_args) => build_kotlin_wrapper(&original_name, &inner_name, &arg_specs, &result_inner, is_option, option_inner.as_ref(), k_args),
        None => quote! {},
    };

    let expanded = quote! {
        #func
        #c_wrapper
        #kotlin_wrapper
    };

    expanded.into()
}

#[proc_macro_attribute]
pub fn repr_c_enum(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemEnum);
    let enum_ident = input.ident.clone();
    let enum_name = enum_ident.to_string();
    let mod_ident = Ident::new(&format!("{}_ffi", to_snake_case(&enum_name)), enum_ident.span());
    let tag_ident = Ident::new(&format!("{}Tag", enum_name), enum_ident.span());
    let data_ident = Ident::new(&format!("{}Data", enum_name), enum_ident.span());
    let ffi_ident = Ident::new(&format!("{}Ffi", enum_name), enum_ident.span());

    let mut tag_variants = Vec::new();
    let mut variant_structs = Vec::new();
    let mut data_union_fields = Vec::new();
    let mut match_arms = Vec::new();
    let mut array_structs = std::collections::BTreeMap::new();

    for (index, variant) in input.variants.iter().enumerate() {
        let variant_ident = &variant.ident;
        let variant_name = variant_ident.to_string();
        let variant_struct_ident = Ident::new(&format!("{}{}", enum_name, variant_name), variant_ident.span());

        let index = index as u32;
        tag_variants.push(quote! { #variant_ident = #index });
        data_union_fields.push(quote! {
            pub #variant_ident: ::std::mem::ManuallyDrop<#variant_struct_ident>
        });

        let (fields, field_inits, match_pattern) = build_variant_fields(
            &enum_ident,
            variant,
            &mut array_structs,
        );

        variant_structs.push(quote! {
            #[repr(C)]
            #[derive(Clone, Debug)]
            pub struct #variant_struct_ident {
                #(#fields)*
            }
        });

        match_arms.push(quote! {
            #match_pattern => {
                let data = #data_ident {
                    #variant_ident: ::std::mem::ManuallyDrop::new(#variant_struct_ident { #(#field_inits)* })
                };
                #ffi_ident { tag: #tag_ident::#variant_ident, data }
            }
        });
    }

    let array_struct_defs = array_structs.values().cloned().collect::<Vec<_>>();

    let expanded = quote! {
        #input

        #[allow(non_snake_case)]
        pub mod #mod_ident {
            use super::*;

            #(#array_struct_defs)*

            #[repr(u32)]
            #[derive(Clone, Copy, Debug, PartialEq, Eq)]
            pub enum #tag_ident {
                #(#tag_variants,)*
            }

            #(#variant_structs)*

            #[repr(C)]
            #[allow(non_snake_case)]
            pub union #data_ident {
                #(#data_union_fields,)*
            }

            #[repr(C)]
            pub struct #ffi_ident {
                pub tag: #tag_ident,
                pub data: #data_ident,
            }

            impl From<&#enum_ident> for #ffi_ident {
                fn from(value: &#enum_ident) -> Self {
                    match value {
                        #(#match_arms)*
                    }
                }
            }
        }
    };

    expanded.into()
}

fn build_variant_fields(
    enum_ident: &Ident,
    variant: &syn::Variant,
    array_structs: &mut std::collections::BTreeMap<String, proc_macro2::TokenStream>,
) -> (Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>, proc_macro2::TokenStream) {
    let mut fields = Vec::new();
    let mut field_inits = Vec::new();

    match &variant.fields {
        syn::Fields::Named(named) => {
            let mut pat_fields = Vec::new();
            for field in &named.named {
                let field_ident = field.ident.as_ref().expect("named field");
                let (field_ty, init_expr) = map_field_type(enum_ident, &field.ty, field_ident, array_structs);
                fields.push(quote! { pub #field_ident: #field_ty, });
                field_inits.push(quote! { #field_ident: #init_expr, });
                pat_fields.push(quote! { #field_ident });
            }
            let variant_ident = &variant.ident;
            let match_pattern = quote! { #enum_ident::#variant_ident { #(#pat_fields),* } };
            (fields, field_inits, match_pattern)
        }
        syn::Fields::Unnamed(unnamed) => {
            let mut pat_fields = Vec::new();
            for (idx, field) in unnamed.unnamed.iter().enumerate() {
                let field_ident = Ident::new(&format!("_{}", idx), field.span());
                let (field_ty, init_expr) = map_field_type(enum_ident, &field.ty, &field_ident, array_structs);
                fields.push(quote! { pub #field_ident: #field_ty, });
                field_inits.push(quote! { #field_ident: #init_expr, });
                pat_fields.push(quote! { #field_ident });
            }
            let variant_ident = &variant.ident;
            let match_pattern = quote! { #enum_ident::#variant_ident( #(#pat_fields),* ) };
            (fields, field_inits, match_pattern)
        }
        syn::Fields::Unit => {
            let variant_ident = &variant.ident;
            let match_pattern = quote! { #enum_ident::#variant_ident };
            (fields, field_inits, match_pattern)
        }
    }
}

fn map_field_type(
    _enum_ident: &Ident,
    ty: &Type,
    field_ident: &Ident,
    array_structs: &mut std::collections::BTreeMap<String, proc_macro2::TokenStream>,
) -> (Type, proc_macro2::TokenStream) {
    if let Some(inner) = vec_inner_type(ty) {
        let inner_ident = type_ident_name(&inner).unwrap_or_else(|| "Unknown".to_string());
        let array_ident = Ident::new(&format!("{}ArrayFfi", inner_ident), field_ident.span());
        let array_struct = quote! {
            #[repr(C)]
            #[derive(Clone, Debug)]
            pub struct #array_ident {
                pub ptr: *const #inner,
                pub len: usize,
            }
        };
        array_structs.entry(array_ident.to_string()).or_insert(array_struct);
        let array_ty: Type = parse_quote!(#array_ident);
        let init_expr = quote! { #array_ident { ptr: #field_ident.as_ptr(), len: #field_ident.len() } };
        return (array_ty, init_expr);
    }

    let init_expr = quote! { #field_ident.clone() };
    (ty.clone(), init_expr)
}

fn vec_inner_type(ty: &Type) -> Option<Type> {
    if let Type::Path(path) = ty {
        let last = path.path.segments.last()?;
        if last.ident != "Vec" {
            return None;
        }
        if let PathArguments::AngleBracketed(args) = &last.arguments {
            if let Some(GenericArgument::Type(inner)) = args.args.first() {
                return Some(inner.clone());
            }
        }
    }
    None
}

fn type_ident_name(ty: &Type) -> Option<String> {
    if let Type::Path(path) = ty {
        return path.path.segments.last().map(|s| s.ident.to_string());
    }
    None
}

fn to_snake_case(input: &str) -> String {
    let mut out = String::new();
    for (i, ch) in input.chars().enumerate() {
        if ch.is_uppercase() {
            if i != 0 {
                out.push('_');
            }
            for lc in ch.to_lowercase() {
                out.push(lc);
            }
        } else {
            out.push(ch);
        }
    }
    out
}

struct ArgSpec {
    name: Ident,
    ty: Type,
    define_ty: Option<Type>,
    is_entity: bool,
}

fn extract_arg_markers(attrs: &mut Vec<syn::Attribute>) -> (Option<Type>, bool) {
    let mut define_ty: Option<Type> = None;
    let mut is_entity = false;
    attrs.retain(|attr| {
        let path = &attr.path();
        let ident = path.get_ident().map(|v| v.to_string()).unwrap_or_default();

        if ident == "define" || path.segments.last().map(|s| s.ident == "define").unwrap_or(false) {
            if let Ok(ty) = attr.parse_args::<Type>() {
                define_ty = Some(ty);
            }
            return false;
        }

        if ident == "entity" || path.segments.last().map(|s| s.ident == "entity").unwrap_or(false) {
            is_entity = true;
            return false;
        }

        true
    });

    (define_ty, is_entity)
}

fn extract_dropbear_result_inner_type(output: &ReturnType) -> syn::Result<Type> {
    match output {
        ReturnType::Type(_, ty) => {
            if let Type::Path(type_path) = &**ty {
                if let Some(segment) = type_path.path.segments.last() {
                    if segment.ident == "DropbearNativeResult" {
                        if let PathArguments::AngleBracketed(args) = &segment.arguments {
                            if let Some(GenericArgument::Type(inner)) = args.args.first() {
                                return Ok(inner.clone());
                            }
                        }
                    }
                }
            }
            Err(syn::Error::new(output.span(), "export requires DropbearNativeResult<T> return type"))
        }
        ReturnType::Default => Err(syn::Error::new(output.span(), "export requires DropbearNativeResult<T> return type")),
    }
}

fn extract_option_inner_type(ty: &Type) -> (bool, Option<Type>) {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Option" {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(GenericArgument::Type(inner)) = args.args.first() {
                        return (true, Some(inner.clone()));
                    }
                }
            }
        }
    }
    (false, None)
}

fn build_c_wrapper(
    original_name: &Ident,
    inner_name: &Ident,
    arg_specs: &[ArgSpec],
    result_inner: &Type,
    is_option: bool,
    option_inner: Option<&Type>,
    c_args: CArgs,
) -> proc_macro2::TokenStream {
    let c_name = c_args.name.unwrap_or_else(|| default_c_name(original_name));
    let c_ident = Ident::new(&c_name, original_name.span());

    let mut wrapper_inputs = Vec::new();
    let mut conversions = Vec::new();
    let mut call_args = Vec::new();

    for arg in arg_specs {
        let name = &arg.name;

        if arg.is_entity {
            wrapper_inputs.push(quote! { #name: u64 });
            conversions.push(quote! {
                let #name = match ::hecs::Entity::from_bits(#name) {
                    Some(v) => v,
                    None => return crate::scripting::native::DropbearNativeError::InvalidEntity.code(),
                };
            });
            call_args.push(quote! { #name });
            continue;
        }

        if let Some(define_ty) = &arg.define_ty {
            let (target_ty, is_mut_ref) = match &arg.ty {
                Type::Reference(reference) => (&*reference.elem, reference.mutability.is_some()),
                _ => {
                    return syn::Error::new(arg.ty.span(), "define(...) requires a reference argument")
                        .to_compile_error();
                }
            };

            wrapper_inputs.push(quote! { #name: #define_ty });

            let convert = if is_mut_ref {
                quote! { let #name = crate::convert_ptr!(mut #name => #target_ty); }
            } else {
                quote! { let #name = crate::convert_ptr!(#name => #target_ty); }
            };
            conversions.push(convert);
            call_args.push(quote! { #name });
            continue;
        }

        if is_string_type(&arg.ty) {
            wrapper_inputs.push(quote! { #name: *const i8 });
            conversions.push(quote! {
                let #name = match {
                    if #name.is_null() { return crate::scripting::native::DropbearNativeError::NullPointer.code(); }
                    unsafe { std::ffi::CStr::from_ptr(#name) }
                }.to_str() {
                    Ok(v) => v.to_string(),
                    Err(_) => return crate::scripting::native::DropbearNativeError::InvalidUTF8.code(),
                };
            });
            call_args.push(quote! { #name });
            continue;
        }

        if is_str_ref(&arg.ty) {
            return syn::Error::new(arg.ty.span(), "&str is not supported by export; use String")
                .to_compile_error();
        }

        if !is_primitive_type(&arg.ty) {
            let (target_ty, is_mut_ref) = match &arg.ty {
                Type::Reference(reference) => (&*reference.elem, reference.mutability.is_some()),
                _ => {
                    return syn::Error::new(arg.ty.span(), "Object inputs must be references")
                        .to_compile_error();
                }
            };

            let ptr_ty = if is_mut_ref {
                quote! { *mut #target_ty }
            } else {
                quote! { *const #target_ty }
            };

            wrapper_inputs.push(quote! { #name: #ptr_ty });
            conversions.push(quote! {
                if #name.is_null() {
                    return crate::scripting::native::DropbearNativeError::NullPointer.code();
                }
            });
            if is_mut_ref {
                conversions.push(quote! { let #name = unsafe { &mut *#name }; });
            } else {
                conversions.push(quote! { let #name = unsafe { &*#name }; });
            }
            call_args.push(quote! { #name });
            continue;
        }

        let ty = &arg.ty;
        wrapper_inputs.push(quote! { #name: #ty });
        call_args.push(quote! { #name });
    }

    let (out_params, out_checks) = if is_unit_type(result_inner) {
        (quote! {}, quote! {})
    } else if is_option {
        let inner = option_inner.expect("option inner");
        (
            quote! { , out0: *mut #inner, out0_present: *mut bool },
            quote! {
                if out0_present.is_null() {
                    return crate::scripting::native::DropbearNativeError::NullPointer.code();
                }
            }
        )
    } else {
        (
            quote! { , out0: *mut #result_inner },
            quote! {
                if out0.is_null() {
                    return crate::scripting::native::DropbearNativeError::NullPointer.code();
                }
            }
        )
    };

    let result_match = if is_unit_type(result_inner) {
        quote! {
            match #inner_name(#(#call_args),*) {
                crate::scripting::result::DropbearNativeResult::Ok(()) => crate::scripting::native::DropbearNativeError::Success.code(),
                crate::scripting::result::DropbearNativeResult::Err(e) => e.code(),
            }
        }
    } else if is_option {
        quote! {
            match #inner_name(#(#call_args),*) {
                crate::scripting::result::DropbearNativeResult::Ok(val_opt) => {
                    match val_opt {
                        Some(v) => {
                            if out0.is_null() {
                                return crate::scripting::native::DropbearNativeError::NullPointer.code();
                            }
                            unsafe { *out0 = v; }
                            unsafe { *out0_present = true; }
                        }
                        None => {
                            unsafe { *out0_present = false; }
                        }
                    }
                    crate::scripting::native::DropbearNativeError::Success.code()
                }
                crate::scripting::result::DropbearNativeResult::Err(e) => e.code(),
            }
        }
    } else {
        quote! {
            match #inner_name(#(#call_args),*) {
                crate::scripting::result::DropbearNativeResult::Ok(val) => {
                    unsafe { *out0 = val; }
                    crate::scripting::native::DropbearNativeError::Success.code()
                }
                crate::scripting::result::DropbearNativeResult::Err(e) => e.code(),
            }
        }
    };

    quote! {
        #[unsafe(no_mangle)]
        pub extern "C" fn #c_ident(#(#wrapper_inputs),* #out_params) -> i32 {
            #(#conversions)*
            #out_checks
            #result_match
        }
    }
}

fn build_kotlin_wrapper(
    original_name: &Ident,
    inner_name: &Ident,
    arg_specs: &[ArgSpec],
    result_inner: &Type,
    is_option: bool,
    option_inner: Option<&Type>,
    kotlin_args: KotlinArgs,
) -> proc_macro2::TokenStream {
    let class_name = kotlin_args.class.replace('.', "_");
    let jni_fn_name = format!("Java_{}_{}", class_name, kotlin_args.func);
    let jni_ident = Ident::new(&jni_fn_name, original_name.span());
    let jni_path: syn::Path = kotlin_args.jni_path.unwrap_or_else(|| parse_quote!(::jni));

    let mut wrapper_inputs = vec![
        quote! { mut env: #jni_path::JNIEnv },
        quote! { _: #jni_path::objects::JClass },
    ];
    let mut conversions = Vec::new();
    let mut call_args = Vec::new();

    for arg in arg_specs {
        let name = &arg.name;
        if arg.is_entity {
            wrapper_inputs.push(quote! { #name: #jni_path::sys::jlong });
            conversions.push(quote! {
                let #name = match ::hecs::Entity::from_bits(#name as u64) {
                    Some(v) => v,
                    None => {
                        let _ = env.throw_new("java/lang/RuntimeException", "Invalid entity id");
                        return crate::ffi_error_return!();
                    }
                };
            });
            call_args.push(quote! { #name });
            continue;
        }

        if let Some(_define_ty) = &arg.define_ty {
            wrapper_inputs.push(quote! { #name: #jni_path::sys::jlong });
            let (target_ty, is_mut_ref) = match &arg.ty {
                Type::Reference(reference) => (&*reference.elem, reference.mutability.is_some()),
                _ => {
                    return syn::Error::new(arg.ty.span(), "define(...) requires a reference argument")
                        .to_compile_error();
                }
            };
            let convert = if is_mut_ref {
                quote! { let #name = crate::convert_ptr!(mut #name => #target_ty); }
            } else {
                quote! { let #name = crate::convert_ptr!(#name => #target_ty); }
            };
            conversions.push(convert);
            call_args.push(quote! { #name });
            continue;
        }

        if is_string_type(&arg.ty) {
            wrapper_inputs.push(quote! { #name: #jni_path::objects::JString });
            conversions.push(quote! {
                let #name = match env.get_string(&#name) {
                    Ok(v) => match v.to_str() {
                        Ok(v) => v.to_string(),
                        Err(e) => {
                            let _ = env.throw_new(
                                "java/lang/RuntimeException",
                                format!("Failed to convert string to utf8: {:?}", e)
                            );
                            return crate::ffi_error_return!();
                        }
                    },
                    Err(e) => {
                        let _ = env.throw_new(
                            "java/lang/RuntimeException",
                            format!("Failed to get string from jni: {:?}", e)
                        );
                        return crate::ffi_error_return!();
                    }
                };
            });
            call_args.push(quote! { #name });
            continue;
        }

        if is_str_ref(&arg.ty) {
            return syn::Error::new(arg.ty.span(), "&str is not supported by export; use String")
                .to_compile_error();
        }

        if !is_primitive_type(&arg.ty) {
            wrapper_inputs.push(quote! { #name: #jni_path::objects::JObject });
            let (target_ty, is_mut_ref) = match &arg.ty {
                Type::Reference(reference) => (&*reference.elem, reference.mutability.is_some()),
                _ => (&arg.ty, false),
            };

            let value_name = Ident::new(&format!("{}_value", name), name.span());
            conversions.push(quote! {
                let #value_name = match crate::scripting::jni::utils::FromJObject::from_jobject(&mut env, &#name) {
                    Ok(v) => v,
                    Err(e) => {
                        let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to convert object: {:?}", e));
                        return crate::ffi_error_return!();
                    }
                };
            });

            if matches!(&arg.ty, Type::Reference(_)) {
                if is_mut_ref {
                    conversions.push(quote! { let #name = &mut #value_name; });
                } else {
                    conversions.push(quote! { let #name = &#value_name; });
                }
                call_args.push(quote! { #name });
            } else {
                conversions.push(quote! { let #name: #target_ty = #value_name; });
                call_args.push(quote! { #name });
            }
            continue;
        }

        let jni_ty = jni_param_type(&arg.ty, &jni_path);
        wrapper_inputs.push(quote! { #name: #jni_ty });
        let cast = jni_arg_cast(&arg.ty, &quote! { #name });
        call_args.push(cast);
    }

    let (jni_return_ty, result_match) = build_jni_return(
        inner_name,
        &call_args,
        result_inner,
        is_option,
        option_inner,
        &jni_path,
    );

    quote! {
        #[unsafe(no_mangle)]
        #[allow(non_snake_case)]
        pub extern "system" fn #jni_ident(#(#wrapper_inputs),*) -> #jni_return_ty {
            #(#conversions)*
            #result_match
        }
    }
}

fn build_jni_return(
    inner_name: &Ident,
    call_args: &[proc_macro2::TokenStream],
    result_inner: &Type,
    is_option: bool,
    option_inner: Option<&Type>,
    jni_path: &syn::Path,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    if is_unit_type(result_inner) {
        let body = quote! {
            match #inner_name(#(#call_args),*) {
                crate::scripting::result::DropbearNativeResult::Ok(()) => (),
                crate::scripting::result::DropbearNativeResult::Err(e) => {
                    let _ = env.throw_new("java/lang/RuntimeException", format!("JNI call failed: {:?}", e));
                }
            }
        };
        return (quote! { () }, body);
    }

    if is_option {
        let inner = option_inner.expect("option inner");
        if is_primitive_type(inner) {
            let (sig, wrapper, jvalue_expr) = jni_boxing_info(inner, jni_path);
            let body = quote! {
                match #inner_name(#(#call_args),*) {
                    crate::scripting::result::DropbearNativeResult::Ok(val) => {
                        crate::return_boxed!(&mut env, val.map(|v| #jvalue_expr), #sig, #wrapper)
                    }
                    crate::scripting::result::DropbearNativeResult::Err(e) => {
                        let _ = env.throw_new("java/lang/RuntimeException", format!("JNI call failed: {:?}", e));
                        std::ptr::null_mut()
                    }
                }
            };
            return (quote! { #jni_path::sys::jobject }, body);
        }

        if is_string_type(inner) {
            let body = quote! {
                match #inner_name(#(#call_args),*) {
                    crate::scripting::result::DropbearNativeResult::Ok(val) => match val {
                        Some(v) => match env.new_string(v) {
                            Ok(s) => s.into_raw(),
                            Err(e) => {
                                let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to create jstring: {:?}", e));
                                std::ptr::null_mut()
                            }
                        },
                        None => std::ptr::null_mut(),
                    },
                    crate::scripting::result::DropbearNativeResult::Err(e) => {
                        let _ = env.throw_new("java/lang/RuntimeException", format!("JNI call failed: {:?}", e));
                        std::ptr::null_mut()
                    }
                }
            };
            return (quote! { #jni_path::sys::jobject }, body);
        }

        let body = quote! {
            match #inner_name(#(#call_args),*) {
                crate::scripting::result::DropbearNativeResult::Ok(val) => match val {
                    Some(v) => match crate::scripting::jni::utils::ToJObject::to_jobject(&v, &mut env) {
                        Ok(obj) => obj.into_raw(),
                        Err(e) => {
                            let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to convert object: {:?}", e));
                            std::ptr::null_mut()
                        }
                    },
                    None => std::ptr::null_mut(),
                },
                crate::scripting::result::DropbearNativeResult::Err(e) => {
                    let _ = env.throw_new("java/lang/RuntimeException", format!("JNI call failed: {:?}", e));
                    std::ptr::null_mut()
                }
            }
        };
        return (quote! { #jni_path::sys::jobject }, body);
    }

    if is_string_type(result_inner) {
        let body = quote! {
            match #inner_name(#(#call_args),*) {
                crate::scripting::result::DropbearNativeResult::Ok(val) => match env.new_string(val) {
                    Ok(s) => s.into_raw(),
                    Err(e) => {
                        let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to create jstring: {:?}", e));
                        crate::ffi_error_return!()
                    }
                },
                crate::scripting::result::DropbearNativeResult::Err(e) => {
                    let _ = env.throw_new("java/lang/RuntimeException", format!("JNI call failed: {:?}", e));
                    crate::ffi_error_return!()
                }
            }
        };
        return (quote! { #jni_path::sys::jstring }, body);
    }

    if is_primitive_type(result_inner) {
        let jni_ret = jni_param_type(result_inner, jni_path);
        let cast = jni_value_cast(result_inner, quote! { val }, jni_path);
        let body = quote! {
            match #inner_name(#(#call_args),*) {
                crate::scripting::result::DropbearNativeResult::Ok(val) => #cast,
                crate::scripting::result::DropbearNativeResult::Err(e) => {
                    let _ = env.throw_new("java/lang/RuntimeException", format!("JNI call failed: {:?}", e));
                    crate::ffi_error_return!()
                }
            }
        };
        return (jni_ret, body);
    }

    let body = quote! {
        match #inner_name(#(#call_args),*) {
            crate::scripting::result::DropbearNativeResult::Ok(val) => match crate::scripting::jni::utils::ToJObject::to_jobject(&val, &mut env) {
                Ok(obj) => obj.into_raw(),
                Err(e) => {
                    let _ = env.throw_new("java/lang/RuntimeException", format!("Failed to convert object: {:?}", e));
                    std::ptr::null_mut()
                }
            },
            crate::scripting::result::DropbearNativeResult::Err(e) => {
                let _ = env.throw_new("java/lang/RuntimeException", format!("JNI call failed: {:?}", e));
                std::ptr::null_mut()
            }
        }
    };
    (quote! { #jni_path::sys::jobject }, body)
}

fn jni_param_type(ty: &Type, jni_path: &syn::Path) -> proc_macro2::TokenStream {
    if is_bool_type(ty) {
        return quote! { #jni_path::sys::jboolean };
    }
    if is_float_type(ty) {
        return quote! { #jni_path::sys::jfloat };
    }
    if is_double_type(ty) {
        return quote! { #jni_path::sys::jdouble };
    }
    if is_i8_type(ty) || is_u8_type(ty) {
        return quote! { #jni_path::sys::jbyte };
    }
    if is_i16_type(ty) || is_u16_type(ty) {
        return quote! { #jni_path::sys::jshort };
    }
    if is_i32_type(ty) || is_u32_type(ty) {
        return quote! { #jni_path::sys::jint };
    }
    if is_i64_type(ty) || is_u64_type(ty) || is_isize_type(ty) || is_usize_type(ty) {
        return quote! { #jni_path::sys::jlong };
    }

    quote! { #ty }
}

fn jni_arg_cast(ty: &Type, name: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    if is_bool_type(ty) {
        return quote! { #name != 0 };
    }
    if is_i8_type(ty) {
        return quote! { #name as i8 };
    }
    if is_u8_type(ty) {
        return quote! { #name as u8 };
    }
    if is_i16_type(ty) {
        return quote! { #name as i16 };
    }
    if is_u16_type(ty) {
        return quote! { #name as u16 };
    }
    if is_i32_type(ty) {
        return quote! { #name as i32 };
    }
    if is_u32_type(ty) {
        return quote! { #name as u32 };
    }
    if is_i64_type(ty) {
        return quote! { #name as i64 };
    }
    if is_u64_type(ty) {
        return quote! { #name as u64 };
    }
    if is_isize_type(ty) {
        return quote! { #name as isize };
    }
    if is_usize_type(ty) {
        return quote! { #name as usize };
    }
    if is_float_type(ty) {
        return quote! { #name as f32 };
    }
    if is_double_type(ty) {
        return quote! { #name as f64 };
    }

    quote! { #name }
}

fn jni_value_cast(ty: &Type, name: proc_macro2::TokenStream, jni_path: &syn::Path) -> proc_macro2::TokenStream {
    if is_bool_type(ty) {
        return quote! { if #name { 1 } else { 0 } };
    }
    if is_i8_type(ty) || is_u8_type(ty) {
        return quote! { #name as #jni_path::sys::jbyte };
    }
    if is_i16_type(ty) || is_u16_type(ty) {
        return quote! { #name as #jni_path::sys::jshort };
    }
    if is_i32_type(ty) || is_u32_type(ty) {
        return quote! { #name as #jni_path::sys::jint };
    }
    if is_i64_type(ty) || is_u64_type(ty) || is_isize_type(ty) || is_usize_type(ty) {
        return quote! { #name as #jni_path::sys::jlong };
    }
    if is_float_type(ty) {
        return quote! { #name as #jni_path::sys::jfloat };
    }
    if is_double_type(ty) {
        return quote! { #name as #jni_path::sys::jdouble };
    }

    quote! { #name }
}

fn jni_boxing_info(
    ty: &Type,
    jni_path: &syn::Path,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream, proc_macro2::TokenStream) {
    if is_i32_type(ty) || is_u32_type(ty) {
        return (
            quote! { "(I)Ljava/lang/Integer;" },
            quote! { "java/lang/Integer" },
            quote! { #jni_path::objects::JValue::Int(v as #jni_path::sys::jint) },
        );
    }
    if is_i64_type(ty) || is_u64_type(ty) || is_isize_type(ty) || is_usize_type(ty) {
        return (
            quote! { "(J)Ljava/lang/Long;" },
            quote! { "java/lang/Long" },
            quote! { #jni_path::objects::JValue::Long(v as #jni_path::sys::jlong) },
        );
    }
    if is_i16_type(ty) || is_u16_type(ty) {
        return (
            quote! { "(S)Ljava/lang/Short;" },
            quote! { "java/lang/Short" },
            quote! { #jni_path::objects::JValue::Short(v as #jni_path::sys::jshort) },
        );
    }
    if is_i8_type(ty) || is_u8_type(ty) {
        return (
            quote! { "(B)Ljava/lang/Byte;" },
            quote! { "java/lang/Byte" },
            quote! { #jni_path::objects::JValue::Byte(v as #jni_path::sys::jbyte) },
        );
    }
    if is_bool_type(ty) {
        return (
            quote! { "(Z)Ljava/lang/Boolean;" },
            quote! { "java/lang/Boolean" },
            quote! { #jni_path::objects::JValue::Bool(if v { 1 } else { 0 }) },
        );
    }
    if is_float_type(ty) {
        return (
            quote! { "(F)Ljava/lang/Float;" },
            quote! { "java/lang/Float" },
            quote! { #jni_path::objects::JValue::Float(v as #jni_path::sys::jfloat) },
        );
    }
    if is_double_type(ty) {
        return (
            quote! { "(D)Ljava/lang/Double;" },
            quote! { "java/lang/Double" },
            quote! { #jni_path::objects::JValue::Double(v as #jni_path::sys::jdouble) },
        );
    }

    (
        quote! { "(J)Ljava/lang/Long;" },
        quote! { "java/lang/Long" },
        quote! { #jni_path::objects::JValue::Long(v as #jni_path::sys::jlong) },
    )
}

fn default_c_name(original_name: &Ident) -> String {
    format!("dropbear_{}", original_name)
}

fn is_string_type(ty: &Type) -> bool {
    matches!(ty, Type::Path(path) if path.path.segments.last().map(|s| s.ident == "String").unwrap_or(false))
}

fn is_str_ref(ty: &Type) -> bool {
    match ty {
        Type::Reference(reference) => {
            matches!(&*reference.elem, Type::Path(path) if path.path.segments.last().map(|s| s.ident == "str").unwrap_or(false))
        }
        _ => false,
    }
}

fn is_primitive_type(ty: &Type) -> bool {
    is_bool_type(ty)
        || is_i8_type(ty)
        || is_i16_type(ty)
        || is_i32_type(ty)
        || is_i64_type(ty)
        || is_isize_type(ty)
        || is_u8_type(ty)
        || is_u16_type(ty)
        || is_u32_type(ty)
        || is_u64_type(ty)
        || is_usize_type(ty)
        || is_float_type(ty)
        || is_double_type(ty)
}

fn is_bool_type(ty: &Type) -> bool {
    matches!(ty, Type::Path(path) if path.path.segments.last().map(|s| s.ident == "bool").unwrap_or(false))
}

fn is_i8_type(ty: &Type) -> bool {
    matches!(ty, Type::Path(path) if path.path.segments.last().map(|s| s.ident == "i8").unwrap_or(false))
}

fn is_i16_type(ty: &Type) -> bool {
    matches!(ty, Type::Path(path) if path.path.segments.last().map(|s| s.ident == "i16").unwrap_or(false))
}

fn is_i32_type(ty: &Type) -> bool {
    matches!(ty, Type::Path(path) if path.path.segments.last().map(|s| s.ident == "i32").unwrap_or(false))
}

fn is_i64_type(ty: &Type) -> bool {
    matches!(ty, Type::Path(path) if path.path.segments.last().map(|s| s.ident == "i64").unwrap_or(false))
}

fn is_isize_type(ty: &Type) -> bool {
    matches!(ty, Type::Path(path) if path.path.segments.last().map(|s| s.ident == "isize").unwrap_or(false))
}

fn is_u8_type(ty: &Type) -> bool {
    matches!(ty, Type::Path(path) if path.path.segments.last().map(|s| s.ident == "u8").unwrap_or(false))
}

fn is_u16_type(ty: &Type) -> bool {
    matches!(ty, Type::Path(path) if path.path.segments.last().map(|s| s.ident == "u16").unwrap_or(false))
}

fn is_u32_type(ty: &Type) -> bool {
    matches!(ty, Type::Path(path) if path.path.segments.last().map(|s| s.ident == "u32").unwrap_or(false))
}

fn is_u64_type(ty: &Type) -> bool {
    matches!(ty, Type::Path(path) if path.path.segments.last().map(|s| s.ident == "u64").unwrap_or(false))
}

fn is_usize_type(ty: &Type) -> bool {
    matches!(ty, Type::Path(path) if path.path.segments.last().map(|s| s.ident == "usize").unwrap_or(false))
}

fn is_float_type(ty: &Type) -> bool {
    matches!(ty, Type::Path(path) if path.path.segments.last().map(|s| s.ident == "f32").unwrap_or(false))
}

fn is_double_type(ty: &Type) -> bool {
    matches!(ty, Type::Path(path) if path.path.segments.last().map(|s| s.ident == "f64").unwrap_or(false))
}

fn transform_function(mut func: ItemFn) -> ItemFn {
    let inputs = func.sig.inputs.clone();
    let output = func.sig.output.clone();
    let block = func.block;

    let inner_type = extract_inner_type(&output);
    let is_void = is_unit_type(&inner_type);

    let mut new_inputs = inputs.clone();

    if !is_void {
        let out_ptr_type: Type = parse_quote! { *mut #inner_type };

        new_inputs.push(FnArg::Typed(syn::PatType {
            attrs: vec![],
            pat: Box::new(parse_quote! { out_result }),
            colon_token: Default::default(),
            ty: Box::new(out_ptr_type),
        }));
    }
    
    let pointer_check = if !is_void {
        quote! {
            if out_result.is_null() {
                return crate::scripting::native::DropbearNativeError::NullPointer.code();
            }
        }
    } else {
        quote! {}
    };

    let success_handling = if !is_void {
        quote! {
            unsafe { *out_result = val; }
            crate::scripting::native::DropbearNativeError::Success.code()
        }
    } else {
        quote! {
            crate::scripting::native::DropbearNativeError::Success.code()
        }
    };

    let new_body = quote! {
        {
            #pointer_check
            
            let logic = || #output {
                #block
            };
            
            match logic() {
                DropbearNativeResult::Ok(val) => {
                    #success_handling
                }
                DropbearNativeResult::Err(e) => {
                    e.code()
                }
            }
        }
    };

    func.sig.inputs = new_inputs;
    func.sig.output = parse_quote! { -> i32 };
    func.sig.abi = Some(parse_quote! { extern "C" });
    func.sig.unsafety = Some(parse_quote! { unsafe });

    func.attrs.push(parse_quote! { #[unsafe(no_mangle)] });

    func.block = Box::new(syn::parse2(new_body).expect("Failed to parse new body"));

    func
}

/// Helper to dig into Result<T, E> and get T
fn extract_inner_type(output: &ReturnType) -> Type {
    match output {
        ReturnType::Type(_, ty) => {
            if let Type::Path(type_path) = &**ty {
                if let Some(segment) = type_path.path.segments.last() {
                    if let PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(GenericArgument::Type(inner)) = args.args.first() {
                            return inner.clone();
                        }
                    }
                }
            }
            parse_quote! { () }
        }
        ReturnType::Default => parse_quote! { () },
    }
}

/// Helper to check if type is ()
fn is_unit_type(ty: &Type) -> bool {
    if let Type::Tuple(tuple) = ty {
        return tuple.elems.is_empty();
    }
    false
}
