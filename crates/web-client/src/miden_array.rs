use thiserror::Error;
#[derive(Debug, Error)]
pub enum ArrayError {
    #[error("out of bounds access -- tried to access at index: {index} with length {length}")]
    OutOfBounds { index: usize, length: usize },
}

/// Implements `FromNapiValue` for napi class types that have `FromNapiRef` + `Clone`.
///
/// napi-rs v3 generates `FromNapiRef` (borrow) for `#[napi]` class types but NOT
/// `FromNapiValue` (by-value). This macro bridges the gap so these types can be used
/// as by-value parameters and inside `Vec<T>` parameters.
#[cfg(feature = "nodejs")]
macro_rules! impl_napi_from_value {
    ($t:ty) => {
        impl napi::bindgen_prelude::FromNapiValue for $t {
            unsafe fn from_napi_value(
                env: napi::bindgen_prelude::sys::napi_env,
                napi_val: napi::bindgen_prelude::sys::napi_value,
            ) -> napi::Result<Self> {
                let ref_val =
                    unsafe { <$t as napi::bindgen_prelude::FromNapiRef>::from_napi_ref(env, napi_val)? };
                Ok(ref_val.clone())
            }
        }
    };
}

/// No-op for browser builds.
#[cfg(feature = "browser")]
macro_rules! impl_napi_from_value {
    ($t:ty) => {};
}

/// Browser variant: Generates JS-exportable array wrapper types using wasm_bindgen.
#[cfg(feature = "browser")]
macro_rules! declare_js_miden_arrays {
    ($(($miden_type_name:path) -> $miden_type_array_name:ident),+ $(,)?) => {
    pub mod miden_arrays {
        use crate::js_error_with_context;
        use wasm_bindgen::prelude::*;
        $(
            #[wasm_bindgen(inspectable)]
            #[derive(Clone)]
            pub struct $miden_type_array_name {
                pub (crate) __inner: Vec<$miden_type_name>,
            }

            #[wasm_bindgen]
            impl $miden_type_array_name {
                #[wasm_bindgen(constructor)]
                pub fn new(elements: Option<Vec<$miden_type_name>>) -> Self {
                    let elements = elements.unwrap_or_else(|| vec![]);
                    Self { __inner: elements }
                }

                /// Get element at index, will always return a clone to avoid aliasing issues.
                pub fn get(&self, index: usize) -> Result<$miden_type_name, wasm_bindgen::JsValue> {
                    match self.__inner.get(index) {
                        Some(value) => Ok(value.clone()),
                        None => {
                            let err = crate::miden_array::ArrayError::OutOfBounds {
                                index,
                                length: self.__inner.len(),
                            };
                            Err(js_error_with_context(
                                err,
                                &format!("array type is: {}", stringify!($miden_type_name)),
                            ))
                        },
                    }
                }

                #[wasm_bindgen(js_name = "replaceAt")]
                pub fn replace_at(
                    &mut self,
                    index: usize,
                    elem: $miden_type_name,
                ) -> Result<(), wasm_bindgen::JsValue> {
                    if let Some(value_at_index) = self.__inner.get_mut(index) {
                        *value_at_index = elem;
                        Ok(())
                    } else {
                        let err =
                            crate::miden_array::ArrayError::OutOfBounds { index, length: self.__inner.len() };
                        Err(js_error_with_context(
                            err,
                            &format!("array type is: {}", stringify!($miden_type_name)),
                        ))
                    }
                }

                pub fn push(&mut self, element: &$miden_type_name) {
                    self.__inner.push(element.clone());
                }

                pub fn length(&self) -> u32 {
                    u32::try_from(self.__inner.len()).expect("fatal: usize in wasm should be u32")
                }
            }

            impl From<$miden_type_array_name> for Vec<$miden_type_name> {
                fn from(array: $miden_type_array_name) -> Self {
                    return array.__inner;
                }
            }

            impl From<&$miden_type_array_name> for Vec<$miden_type_name> {
                fn from(array: &$miden_type_array_name) -> Self {
                    return array.__inner.clone();
                }
            }

            impl From<Vec<$miden_type_name>> for $miden_type_array_name {
                fn from(vec: Vec<$miden_type_name>) -> Self {
                    Self::new(Some(vec))
                }
            }
        )+
    }
    };
}

/// Node.js variant: Creates newtype wrapper structs around `Vec<T>` that implement
/// `FromNapiRef` and `FromNapiValue`, allowing them to be used as `&ArrayType` parameters
/// in `#[napi]` functions. The wrappers deref to `Vec<T>` for ergonomic access.
#[cfg(feature = "nodejs")]
macro_rules! declare_js_miden_arrays {
    ($(($miden_type_name:path) -> $miden_type_array_name:ident),+ $(,)?) => {
    pub mod miden_arrays {
        $(
            #[derive(Clone)]
            pub struct $miden_type_array_name(pub(crate) Vec<$miden_type_name>);

            impl std::ops::Deref for $miden_type_array_name {
                type Target = Vec<$miden_type_name>;
                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }

            impl std::ops::DerefMut for $miden_type_array_name {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.0
                }
            }

            impl From<$miden_type_array_name> for Vec<$miden_type_name> {
                fn from(array: $miden_type_array_name) -> Self {
                    array.0
                }
            }

            impl From<&$miden_type_array_name> for Vec<$miden_type_name> {
                fn from(array: &$miden_type_array_name) -> Self {
                    array.0.clone()
                }
            }

            impl From<Vec<$miden_type_name>> for $miden_type_array_name {
                fn from(vec: Vec<$miden_type_name>) -> Self {
                    Self(vec)
                }
            }

            impl napi::bindgen_prelude::TypeName for $miden_type_array_name {
                fn type_name() -> &'static str {
                    concat!("Array<", stringify!($miden_type_name), ">")
                }

                fn value_type() -> napi::ValueType {
                    napi::ValueType::Object
                }
            }

            impl napi::bindgen_prelude::TypeName for &$miden_type_array_name {
                fn type_name() -> &'static str {
                    concat!("Array<", stringify!($miden_type_name), ">")
                }

                fn value_type() -> napi::ValueType {
                    napi::ValueType::Object
                }
            }

            impl napi::bindgen_prelude::TypeName for &mut $miden_type_array_name {
                fn type_name() -> &'static str {
                    concat!("Array<", stringify!($miden_type_name), ">")
                }

                fn value_type() -> napi::ValueType {
                    napi::ValueType::Object
                }
            }

            impl napi::bindgen_prelude::ValidateNapiValue for $miden_type_array_name {
                unsafe fn validate(
                    env: napi::bindgen_prelude::sys::napi_env,
                    napi_val: napi::bindgen_prelude::sys::napi_value,
                ) -> napi::Result<napi::bindgen_prelude::sys::napi_value> {
                    unsafe { <Vec<$miden_type_name> as napi::bindgen_prelude::ValidateNapiValue>::validate(env, napi_val) }
                }
            }

            impl napi::bindgen_prelude::FromNapiValue for $miden_type_array_name {
                unsafe fn from_napi_value(
                    env: napi::bindgen_prelude::sys::napi_env,
                    napi_val: napi::bindgen_prelude::sys::napi_value,
                ) -> napi::Result<Self> {
                    let vec = unsafe { <Vec<$miden_type_name> as napi::bindgen_prelude::FromNapiValue>::from_napi_value(env, napi_val)? };
                    Ok(Self(vec))
                }
            }

            impl napi::bindgen_prelude::FromNapiRef for $miden_type_array_name {
                unsafe fn from_napi_ref(
                    env: napi::bindgen_prelude::sys::napi_env,
                    napi_val: napi::bindgen_prelude::sys::napi_value,
                ) -> napi::Result<&'static Self> {
                    // Deserialize the JS array into a Vec, wrap it, leak it so we can return &'static Self.
                    // This mirrors how napi class types handle FromNapiRef (they unwrap a pointer).
                    let vec = unsafe { <Vec<$miden_type_name> as napi::bindgen_prelude::FromNapiValue>::from_napi_value(env, napi_val)? };
                    let boxed = Box::new(Self(vec));
                    Ok(Box::leak(boxed))
                }
            }

            impl napi::bindgen_prelude::ToNapiValue for $miden_type_array_name {
                unsafe fn to_napi_value(
                    env: napi::bindgen_prelude::sys::napi_env,
                    val: Self,
                ) -> napi::Result<napi::bindgen_prelude::sys::napi_value> {
                    unsafe { <Vec<$miden_type_name> as napi::bindgen_prelude::ToNapiValue>::to_napi_value(env, val.0) }
                }
            }

            impl napi::bindgen_prelude::ToNapiValue for &$miden_type_array_name {
                unsafe fn to_napi_value(
                    env: napi::bindgen_prelude::sys::napi_env,
                    val: Self,
                ) -> napi::Result<napi::bindgen_prelude::sys::napi_value> {
                    unsafe { <Vec<$miden_type_name> as napi::bindgen_prelude::ToNapiValue>::to_napi_value(env, val.0.clone()) }
                }
            }
        )+
    }
    };
}
