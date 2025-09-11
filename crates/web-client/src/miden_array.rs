use thiserror::Error;
#[derive(Debug, Error)]
pub enum ArrayError {
    #[error("out of bounds access -- tried to access at index: {index} with length {length}")]
    OutOfBounds { index: usize, length: usize },
}

macro_rules! define_array {
    ($miden_type_name:ident -> $miden_type_array_name:ident) => {
        #[derive(Clone, Debug)]
        #[wasm_bindgen]
        pub struct $miden_type_array_name {
            __inner: Vec<$miden_type_name>,
            #[wasm_bindgen(readonly)]
            pub length: usize,
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

            /// Replace element at index
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

            /// Iterate over the Array and apply the given closure.
            /// The closure must not modify the elements inside the array.
            // FIXME: Revisit this
            // pub fn each(&self, closure: &wasm_bindgen_futures::js_sys::Function) {
            //     let this = JsValue::undefined();
            //     for elem in self.__inner.iter() {
            //         closure.call1(&this, &JsValue::from(elem));
            //     }
            // }

            /// Only for use from the rust library. Return the inner vec
            /// for this Array type. Take ownership to avoid aliasing issues.
            pub(crate) fn as_vec(self) -> Vec<$miden_type_name> {
                self.__inner
            }

            // pub(crate) fn from_vec(elements: Vec<$miden_type>) -> Self {
            //    Self {
            //        __inner: elements;
            //    }
            // }
        }
    };
}
