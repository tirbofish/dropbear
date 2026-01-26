use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input, ItemMod, Item, Type, parse_quote, ItemFn, FnArg, ReturnType, PathArguments, GenericArgument};

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
