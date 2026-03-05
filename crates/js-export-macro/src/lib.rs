//! Proc macro that generates dual `wasm_bindgen`/`napi` annotations from a single attribute.
//!
//! # Usage
//!
//! ```ignore
//! #[js_export]                          // bare — applies to struct/enum/impl
//! #[js_export(constructor)]             // forwarded to both wasm_bindgen & napi
//! #[js_export(js_name = "foo")]         // forwarded to both
//! #[js_export(getter, js_name = "x")]   // forwarded to both
//! ```
//!
//! When a method signature contains [`JsU64`], the macro automatically splits it
//! into browser (`u64`) and Node.js (`f64`) variants.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::visit_mut::VisitMut;
use syn::{parse_macro_input, Ident, ImplItem, ImplItemFn, Item, ItemEnum, ItemImpl, ItemStruct};

// ================================================================================================
// Entry point
// ================================================================================================

#[proc_macro_attribute]
pub fn js_export(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr: TokenStream2 = attr.into();
    let item = parse_macro_input!(item as Item);

    let output = match item {
        Item::Struct(s) => handle_struct(&attr, s),
        Item::Enum(e) => handle_enum(&attr, e),
        Item::Impl(i) => handle_impl(&attr, i),
        other => {
            return syn::Error::new_spanned(other, "#[js_export] only supports struct, enum, impl")
                .to_compile_error()
                .into();
        }
    };

    output.into()
}

// ================================================================================================
// Struct / Enum handlers
// ================================================================================================

fn handle_struct(attr: &TokenStream2, item: ItemStruct) -> TokenStream2 {
    let wasm = wasm_attr(attr);
    let napi = napi_attr(attr);

    // item already contains its own attributes (like #[derive(Clone)]),
    // so we just prepend the platform attrs.
    quote! {
        #wasm
        #napi
        #item
    }
}

fn handle_enum(attr: &TokenStream2, item: ItemEnum) -> TokenStream2 {
    let wasm = wasm_attr(attr);
    let napi = napi_attr(attr);

    quote! {
        #wasm
        #napi
        #item
    }
}

// ================================================================================================
// Impl block handler
// ================================================================================================

fn handle_impl(outer_attr: &TokenStream2, mut item: ItemImpl) -> TokenStream2 {
    let self_ty = &item.self_ty;
    let generics = &item.generics;

    // Partition methods: those with JsU64 in signature vs those without.
    let mut shared_methods: Vec<ImplItemFn> = Vec::new();
    let mut jsu64_methods: Vec<ImplItemFn> = Vec::new();
    let mut other_items: Vec<ImplItem> = Vec::new(); // const, type, etc.

    for member in item.items.drain(..) {
        match member {
            ImplItem::Fn(mut method) => {
                // Strip #[js_export(...)] from the method — we'll re-emit it.
                let method_attr = extract_js_export_attr(&mut method);
                let method_attr_tokens =
                    method_attr.map(|a| a.into()).unwrap_or_else(TokenStream2::new);

                if has_jsu64(&method) {
                    // Tag the method with its js_export args for later.
                    method
                        .attrs
                        .push(syn::parse_quote!(#[__js_export_args(#method_attr_tokens)]));
                    jsu64_methods.push(method);
                } else {
                    // Annotate the method inline with dual cfg_attr.
                    let wasm = wasm_attr(&method_attr_tokens);
                    let napi = napi_attr(&method_attr_tokens);
                    method.attrs.push(syn::parse_quote!(#wasm));
                    method.attrs.push(syn::parse_quote!(#napi));
                    shared_methods.push(method);
                }
            }
            other => other_items.push(other),
        }
    }

    let mut output = TokenStream2::new();

    // --- Shared impl block (methods without JsU64) ---
    if !shared_methods.is_empty() || !other_items.is_empty() {
        let wasm_outer = wasm_attr(outer_attr);
        let napi_outer = napi_attr(outer_attr);
        output.extend(quote! {
            #wasm_outer
            #napi_outer
            impl #generics #self_ty {
                #(#other_items)*
                #(#shared_methods)*
            }
        });
    }

    // --- Platform-specific impl blocks (methods with JsU64) ---
    if !jsu64_methods.is_empty() {
        let browser_methods: Vec<ImplItemFn> =
            jsu64_methods.iter().map(|m| make_platform_method(m, Platform::Browser)).collect();
        let nodejs_methods: Vec<ImplItemFn> =
            jsu64_methods.iter().map(|m| make_platform_method(m, Platform::Nodejs)).collect();

        output.extend(quote! {
            #[cfg(feature = "browser")]
            #[::wasm_bindgen::prelude::wasm_bindgen]
            impl #generics #self_ty {
                #(#browser_methods)*
            }

            #[cfg(feature = "nodejs")]
            #[::napi_derive::napi]
            impl #generics #self_ty {
                #(#nodejs_methods)*
            }
        });
    }

    output
}

// ================================================================================================
// Platform-specific method generation
// ================================================================================================

#[derive(Clone, Copy)]
enum Platform {
    Browser,
    Nodejs,
}

fn make_platform_method(method: &ImplItemFn, platform: Platform) -> ImplItemFn {
    let mut method = method.clone();

    // Extract the stashed __js_export_args and apply platform-specific attribute.
    let args = extract_stashed_args(&mut method);
    match platform {
        Platform::Browser => {
            if args.is_empty() {
                method.attrs.push(syn::parse_quote!(#[::wasm_bindgen::prelude::wasm_bindgen]));
            } else {
                method
                    .attrs
                    .push(syn::parse_quote!(#[::wasm_bindgen::prelude::wasm_bindgen(#args)]));
            }
        }
        Platform::Nodejs => {
            if args.is_empty() {
                method.attrs.push(syn::parse_quote!(#[::napi_derive::napi]));
            } else {
                method.attrs.push(syn::parse_quote!(#[::napi_derive::napi(#args)]));
            }
        }
    }

    // Replace JsU64 in signature.
    let replacement = match platform {
        Platform::Browser => "u64",
        Platform::Nodejs => "f64",
    };
    let mut replacer = JsU64Replacer { replacement };
    replacer.visit_impl_item_fn_mut(&mut method);

    method
}

/// Extract the `#[__js_export_args(...)]` marker we stashed earlier.
fn extract_stashed_args(method: &mut ImplItemFn) -> TokenStream2 {
    let mut args = TokenStream2::new();
    method.attrs.retain(|attr| {
        if attr.path().is_ident("__js_export_args") {
            if let syn::Meta::List(list) = &attr.meta {
                args = list.tokens.clone();
            }
            false
        } else {
            true
        }
    });
    args
}

// ================================================================================================
// JsU64 detection
// ================================================================================================

fn has_jsu64(method: &ImplItemFn) -> bool {
    let mut detector = JsU64Detector { found: false };
    // Check params.
    for arg in &method.sig.inputs {
        if let syn::FnArg::Typed(pat_type) = arg {
            let mut ty = (*pat_type.ty).clone();
            detector.visit_type_mut(&mut ty);
        }
    }
    // Check return type.
    if let syn::ReturnType::Type(_, ty) = &method.sig.output {
        let mut ty = (**ty).clone();
        detector.visit_type_mut(&mut ty);
    }
    detector.found
}

struct JsU64Detector {
    found: bool,
}

impl VisitMut for JsU64Detector {
    fn visit_type_path_mut(&mut self, tp: &mut syn::TypePath) {
        if tp.path.is_ident("JsU64") {
            self.found = true;
        }
        syn::visit_mut::visit_type_path_mut(self, tp);
    }
}

// ================================================================================================
// JsU64 replacement (only in method signature, NOT in body)
// ================================================================================================

struct JsU64Replacer {
    replacement: &'static str,
}

impl VisitMut for JsU64Replacer {
    fn visit_impl_item_fn_mut(&mut self, method: &mut ImplItemFn) {
        // Only visit the signature, NOT the body.
        for arg in &mut method.sig.inputs {
            self.visit_fn_arg_mut(arg);
        }
        self.visit_return_type_mut(&mut method.sig.output);
        // Deliberately skip: self.visit_block_mut(&mut method.block);
    }

    fn visit_type_path_mut(&mut self, tp: &mut syn::TypePath) {
        if tp.path.is_ident("JsU64") {
            let ident = Ident::new(self.replacement, tp.path.segments[0].ident.span());
            tp.path = ident.into();
        }
        syn::visit_mut::visit_type_path_mut(self, tp);
    }
}

// ================================================================================================
// Attribute helpers
// ================================================================================================

/// Build `#[cfg_attr(feature = "browser", ::wasm_bindgen::prelude::wasm_bindgen(...))]`
fn wasm_attr(args: &TokenStream2) -> TokenStream2 {
    if args.is_empty() {
        quote! { #[cfg_attr(feature = "browser", ::wasm_bindgen::prelude::wasm_bindgen)] }
    } else {
        quote! { #[cfg_attr(feature = "browser", ::wasm_bindgen::prelude::wasm_bindgen(#args))] }
    }
}

/// Build `#[cfg_attr(feature = "nodejs", ::napi_derive::napi(...))]`
fn napi_attr(args: &TokenStream2) -> TokenStream2 {
    if args.is_empty() {
        quote! { #[cfg_attr(feature = "nodejs", ::napi_derive::napi)] }
    } else {
        quote! { #[cfg_attr(feature = "nodejs", ::napi_derive::napi(#args))] }
    }
}

/// Extract and remove a `#[js_export(...)]` attribute from a method, returning its args.
fn extract_js_export_attr(method: &mut ImplItemFn) -> Option<TokenStream2> {
    let mut extracted = None;
    method.attrs.retain(|attr| {
        if attr.path().is_ident("js_export") {
            extracted = Some(match &attr.meta {
                syn::Meta::Path(_) => TokenStream2::new(),
                syn::Meta::List(list) => list.tokens.clone(),
                syn::Meta::NameValue(_) => TokenStream2::new(),
            });
            false
        } else {
            true
        }
    });
    extracted
}

