pub mod account;
pub mod account_header;
pub mod account_id;
pub mod account_storage_mode;
pub mod address;
pub mod auth;
pub mod felt;
pub mod input_note_record;
pub mod note_filter;
pub mod note_id;
pub mod output_note_record;
pub mod sync_summary;
pub mod transaction_id;
pub mod word;

// NAPI WRAPPER MACRO
// ================================================================================================

/// Generates a napi-compatible newtype wrapper and bidirectional `From` impls.
///
/// This eliminates the repetitive boilerplate that every model needs:
/// the struct definition, `#[napi]`/`#[derive]` attributes, and 2-4 `From` impls
/// for converting between the wrapper and native types.
///
/// # Variants
///
/// - `copy`: For `Copy` types (e.g. `Felt`, `AccountId`). Generates `Copy + Clone` derives and uses
///   `*` dereference in ref conversions.
/// - `clone`: For non-`Copy` types (e.g. `Account`, `Address`). Generates `Clone` derive and uses
///   `.clone()` in ref conversions.
///
/// # Usage
///
/// ```ignore
/// napi_wrap!(copy  Felt      wraps NativeFelt);
/// napi_wrap!(clone Account   wraps NativeAccount);
/// ```
macro_rules! napi_wrap {
    (copy $Name:ident wraps $Native:ty) => {
        #[napi]
        #[derive(Clone, Copy)]
        pub struct $Name(pub(crate) $Native);

        impl From<$Native> for $Name {
            fn from(native: $Native) -> Self {
                $Name(native)
            }
        }
        impl From<&$Native> for $Name {
            fn from(native: &$Native) -> Self {
                $Name(*native)
            }
        }
        impl From<$Name> for $Native {
            fn from(w: $Name) -> Self {
                w.0
            }
        }
        impl From<&$Name> for $Native {
            fn from(w: &$Name) -> Self {
                w.0
            }
        }
    };
    (clone $Name:ident wraps $Native:ty) => {
        #[napi]
        #[derive(Clone)]
        pub struct $Name(pub(crate) $Native);

        impl From<$Native> for $Name {
            fn from(native: $Native) -> Self {
                $Name(native)
            }
        }
        impl From<&$Native> for $Name {
            fn from(native: &$Native) -> Self {
                $Name(native.clone())
            }
        }
        impl From<$Name> for $Native {
            fn from(w: $Name) -> Self {
                w.0
            }
        }
        impl From<&$Name> for $Native {
            fn from(w: &$Name) -> Self {
                w.0.clone()
            }
        }
    };
    // One-way only (Clone): wrap native → wrapper, unwrap wrapper → native (owned).
    (clone $Name:ident wraps $Native:ty,one_way) => {
        #[napi]
        #[derive(Clone)]
        pub struct $Name(pub(crate) $Native);

        impl From<$Native> for $Name {
            fn from(native: $Native) -> Self {
                $Name(native)
            }
        }
        impl From<$Name> for $Native {
            fn from(w: $Name) -> Self {
                w.0
            }
        }
    };
    // One-way only (no Clone): for native types that don't implement Clone.
    (owned $Name:ident wraps $Native:ty) => {
        #[napi]
        pub struct $Name(pub(crate) $Native);

        impl From<$Native> for $Name {
            fn from(native: $Native) -> Self {
                $Name(native)
            }
        }
    };
}

pub(crate) use napi_wrap;
