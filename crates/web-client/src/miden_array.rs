use thiserror::Error;

#[derive(Debug, Error)]
pub enum ArrayError {
    #[error("out of bounds access -- tried to access at index: {index} with length {length}")]
    OutOfBounds { index: usize, length: usize },
}
/// Generates JS-exportable arrays that will run within the WASM
/// memory space. Also, since we're always cloning and not exposing
/// the inner vec as public, this new wrapper array should avoid
/// potential borrowing issues when interacting between JS and WASM.
macro_rules! declare_js_miden_arrays {
    ($(($miden_type_name:path) -> $miden_type_array_name:ident),+ $(,)?) => {
    pub mod miden_arrays {
        use alloc::format;
        use alloc::vec;
        use alloc::vec::Vec;
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
