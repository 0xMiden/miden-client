//! Unified attribute macro for wasm-bindgen and napi-derive bindings.
//!
//! This macro simplifies dual-platform support by expanding a single `#[bindings]`
//! attribute into the appropriate `#[cfg_attr(...)]` annotations for both wasm and napi.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Item};

/// Unified bindings attribute for wasm-bindgen and napi-derive.
///
/// # On Structs/Enums
/// ```rust
/// #[bindings]
/// pub struct MyType(NativeType);
/// ```
/// Expands to:
/// ```rust
/// #[cfg_attr(feature = "wasm", wasm_bindgen)]
/// #[cfg_attr(feature = "napi", napi_derive::napi)]
/// pub struct MyType(NativeType);
/// ```
///
/// # On Impl Blocks
/// ```rust
/// #[bindings]
/// impl MyType {
///     #[bindings(constructor)]
///     pub fn new() -> Self { ... }
///
///     #[bindings(factory)]
///     pub fn from_str(s: String) -> Self { ... }
///
///     #[bindings(getter)]
///     pub fn value(&self) -> u32 { ... }
///
///     #[bindings(js_name = "customName")]
///     pub fn some_method(&self) -> String { ... }
///
///     pub fn regular_method(&self) -> bool { ... }
/// }
/// ```
#[proc_macro_attribute]
pub fn bindings(attr: TokenStream, item: TokenStream) -> TokenStream {
    let _args_str = attr.to_string();
    let item = parse_macro_input!(item as Item);

    match item {
        Item::Struct(item_struct) => {
            let attrs = quote! {
                #[cfg_attr(feature = "wasm", wasm_bindgen)]
                #[cfg_attr(feature = "napi", napi_derive::napi)]
            };

            quote! {
                #attrs
                #item_struct
            }.into()
        }
        Item::Impl(mut item_impl) => {
            let impl_attrs = quote! {
                #[cfg_attr(feature = "wasm", wasm_bindgen)]
                #[cfg_attr(feature = "napi", napi_derive::napi)]
            };

            // Process each method in the impl block
            for item in &mut item_impl.items {
                if let syn::ImplItem::Fn(method) = item {
                    // Remove any existing #[bindings(...)] attributes and process them
                    let mut method_args = String::new();
                    method.attrs.retain(|attr| {
                        if attr.path().is_ident("bindings") {
                            method_args = attr.meta.require_list()
                                .map(|list| list.tokens.to_string())
                                .unwrap_or_default();
                            false  // Remove this attribute
                        } else {
                            true  // Keep other attributes
                        }
                    });

                    // Add appropriate cfg_attr based on the bindings arguments
                    add_method_attrs(&method_args, method);
                }
            }

            quote! {
                #impl_attrs
                #item_impl
            }.into()
        }
        Item::Enum(item_enum) => {
            let attrs = quote! {
                #[cfg_attr(feature = "wasm", wasm_bindgen)]
                #[cfg_attr(feature = "napi", napi_derive::napi)]
            };

            quote! {
                #attrs
                #item_enum
            }.into()
        }
        _ => {
            let attrs = quote! {
                #[cfg_attr(feature = "wasm", wasm_bindgen)]
                #[cfg_attr(feature = "napi", napi_derive::napi)]
            };

            quote! {
                #attrs
                #item
            }.into()
        }
    }
}

fn add_method_attrs(args_str: &str, method: &mut syn::ImplItemFn) {
    let mut wasm_parts = Vec::new();
    let mut napi_parts = Vec::new();

    // Parse common attributes
    if args_str.contains("constructor") {
        wasm_parts.push("constructor");
        napi_parts.push("constructor");
    }

    if args_str.contains("factory") {
        // factory only for napi
        napi_parts.push("factory");
    }

    if args_str.contains("getter") {
        wasm_parts.push("getter");
        napi_parts.push("getter");
    }

    if args_str.contains("setter") {
        wasm_parts.push("setter");
        napi_parts.push("setter");
    }

    // Extract js_name if present
    let js_name = if let Some(start) = args_str.find("js_name") {
        if let Some(eq_pos) = args_str[start..].find('=') {
            let after_eq = &args_str[start + eq_pos + 1..].trim_start();
            if let Some(quote_start) = after_eq.find('"') {
                if let Some(quote_end) = after_eq[quote_start + 1..].find('"') {
                    Some(after_eq[quote_start + 1..quote_start + 1 + quote_end].to_string())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // Build attribute strings
    let wasm_attr_str = if let Some(js_name_val) = &js_name {
        if wasm_parts.is_empty() {
            format!("js_name = \"{}\"", js_name_val)
        } else {
            format!("{}, js_name = \"{}\"", wasm_parts.join(", "), js_name_val)
        }
    } else if !wasm_parts.is_empty() {
        wasm_parts.join(", ")
    } else {
        String::new()
    };

    let napi_attr_str = if let Some(js_name_val) = &js_name {
        if napi_parts.is_empty() {
            format!("js_name = \"{}\"", js_name_val)
        } else {
            format!("{}, js_name = \"{}\"", napi_parts.join(", "), js_name_val)
        }
    } else if !napi_parts.is_empty() {
        napi_parts.join(", ")
    } else {
        String::new()
    };

    // Add cfg_attr attributes
    if !wasm_attr_str.is_empty() {
        let attr_tokens: proc_macro2::TokenStream = format!(
            "#[cfg_attr(feature = \"wasm\", wasm_bindgen({}))]",
            wasm_attr_str
        ).parse().unwrap();
        method.attrs.push(syn::parse_quote! { #attr_tokens });
    }

    if !napi_attr_str.is_empty() {
        let attr_tokens: proc_macro2::TokenStream = format!(
            "#[cfg_attr(feature = \"napi\", napi({}))]",
            napi_attr_str
        ).parse().unwrap();
        method.attrs.push(syn::parse_quote! { #attr_tokens });
    }
}
