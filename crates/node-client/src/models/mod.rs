pub mod account;
pub mod account_header;
pub mod account_id;
pub mod account_storage_mode;
pub mod address;
pub mod asset_vault;
pub mod auth;
pub mod felt;
pub mod input_note_record;
pub mod note;
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

// NAPI DELEGATE MACRO
// ================================================================================================

/// Generates `#[napi]` delegate methods on a newtype wrapper, eliminating
/// per-method boilerplate (`#[napi]`, `pub fn`, body).
///
/// napi-rs auto-converts snake_case to camelCase, so `is_faucet` becomes `isFaucet` in JS.
///
/// # Supported patterns
///
/// - `delegate method -> ReturnType;` — calls `self.0.method()` and converts via `.into()`. Works
///   for any return type (for `bool`/`u32`/etc., `.into()` is a no-op).
///
/// - `collect field -> Vec<ReturnType>;` — iterates `self.0.field` and converts each via `.into()`.
///
/// Methods needing custom logic, arguments, or non-standard JS names
/// should be written in a separate `#[napi] impl` block.
///
/// # Usage
///
/// ```ignore
/// napi_delegate!(impl Account {
///     /// Returns the account identifier.
///     delegate id -> AccountId;
///     /// Returns true if the account is a faucet.
///     delegate is_faucet -> bool;
/// });
/// ```
macro_rules! napi_delegate {
    // Entry: start accumulating methods.
    (impl $Wrapper:ident { $($body:tt)* }) => {
        napi_delegate!(@acc $Wrapper [] $($body)*);
    };

    // Done — emit the impl block with all accumulated methods.
    (@acc $Wrapper:ident [ $($methods:tt)* ]) => {
        #[napi]
        impl $Wrapper { $($methods)* }
    };

    // delegate: call same-named method on inner, convert via .into().
    (@acc $Wrapper:ident [ $($methods:tt)* ]
        $(#[$meta:meta])* delegate $method:ident -> $ret:ty;
        $($rest:tt)*
    ) => {
        napi_delegate!(@acc $Wrapper [
            $($methods)*
            $(#[$meta])*
            #[napi]
            pub fn $method(&self) -> $ret { self.0.$method().into() }
        ] $($rest)*);
    };

    // collect: iterate a field on inner, convert each element via .into().
    (@acc $Wrapper:ident [ $($methods:tt)* ]
        $(#[$meta:meta])* collect $field:ident -> $ret:ty;
        $($rest:tt)*
    ) => {
        napi_delegate!(@acc $Wrapper [
            $($methods)*
            $(#[$meta])*
            #[napi]
            pub fn $field(&self) -> $ret { self.0.$field.iter().map(Into::into).collect() }
        ] $($rest)*);
    };
}

pub(crate) use napi_delegate;
