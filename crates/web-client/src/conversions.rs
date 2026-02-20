//! Common conversion patterns for wrapper types.

/// Implements standard From trait conversions for a newtype wrapper.
///
/// This macro generates implementations for:
/// - `From<NativeType> for WrapperType` - owned conversion
/// - `From<&NativeType> for WrapperType` - reference conversion (clones)
///
/// # Usage
///
/// ```rust
/// impl_wrapper_conversions!(Word, NativeWord);
/// ```
///
/// This expands to:
/// ```rust
/// impl From<NativeWord> for Word {
///     fn from(native: NativeWord) -> Self {
///         Word(native)
///     }
/// }
///
/// impl From<&NativeWord> for Word {
///     fn from(native: &NativeWord) -> Self {
///         Word(native.clone())
///     }
/// }
/// ```
#[macro_export]
macro_rules! impl_wrapper_conversions {
    ($wrapper:ty, $native:ty) => {
        impl From<$native> for $wrapper {
            fn from(native: $native) -> Self {
                Self(native)
            }
        }

        impl From<&$native> for $wrapper {
            fn from(native: &$native) -> Self {
                Self(native.clone())
            }
        }
    };
}
