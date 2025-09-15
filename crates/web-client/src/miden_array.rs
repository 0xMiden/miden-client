use thiserror::Error;
#[derive(Debug, Error)]
pub enum ArrayError {
    #[error("out of bounds access -- tried to access at index: {index} with length {length}")]
    OutOfBounds { index: usize, length: usize },
}
macro_rules! export_js_miden_array {
    ($(($miden_type_name:path) -> $miden_type_array_name:ident),+ $(,)?) => {
        $(
            #[wasm_bindgen(inspectable)]
            #[derive(Clone)]
            pub struct $miden_type_array_name {
                __inner: Vec<$miden_type_name>,
                length: usize,
            }

            #[wasm_bindgen]
            impl $miden_type_array_name {
                #[wasm_bindgen(constructor)]
                pub fn new(elements: Option<Vec<$miden_type_name>>) -> Self {
                    let elements = elements.unwrap_or_else(|| vec![]);
                    let length = elements.len();
                    Self { __inner: elements, length }
                }

                /// Get element at index, will always return a clone to avoid aliasing issues.
                pub fn at(&self, index: usize) -> Result<$miden_type_name, wasm_bindgen::JsValue> {
                    match self.__inner.get(index) {
                        Some(value) => Ok(value.clone()),
                        None => {
                            let err = crate::miden_array::ArrayError::OutOfBounds {
                                index,
                                length: self.length,
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
                            crate::miden_array::ArrayError::OutOfBounds { index, length: self.length };
                        Err(js_error_with_context(
                            err,
                            &format!("array type is: {}", stringify!($miden_type_name)),
                        ))
                    }
                }

                pub fn push(&mut self, element: $miden_type_name) {
                    self.__inner.push(element);
                }

                pub fn length(&self) -> usize {
                    self.__inner.len()
                }
            }

            impl From<$miden_type_array_name> for Vec<$miden_type_name> {
                fn from(array: $miden_type_array_name) -> Self {
                    return array.__inner;
                }
            }

            impl From<Vec<$miden_type_name>> for $miden_type_array_name {
                fn from(vec: Vec<$miden_type_name>) -> Self {
                    Self::new(Some(vec))
                }
            }
        )+
    };
}
