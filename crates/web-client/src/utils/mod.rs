use alloc::vec::Vec;

use miden_client::SliceReader;
use miden_client::utils::{Deserializable, Serializable};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

use crate::js_error_with_context;

#[cfg(feature = "testing")]
pub mod test_utils;

/// Serializes any value that implements `Serializable` into a `Uint8Array`.
pub fn serialize_to_uint8array<T: Serializable>(value: &T) -> Uint8Array {
    let mut buffer = Vec::new();
    // Call the trait method to write into the buffer.
    value.write_into(&mut buffer);
    Uint8Array::from(&buffer[..])
}

/// Deserializes a `Uint8Array` into any type that implements `Deserializable`.
pub fn deserialize_from_uint8array<T: Deserializable>(bytes: &Uint8Array) -> Result<T, JsValue> {
    let vec = bytes.to_vec();
    let mut reader = SliceReader::new(&vec);
    let context = alloc::format!("failed to deserialize {}", core::any::type_name::<T>());
    T::read_from(&mut reader).map_err(|e| js_error_with_context(e, &context))
}

#[cfg(test)]
mod tests {
    use miden_client::utils::{ByteReader, DeserializationError};

    use super::*;

    // Mock types for testing
    #[derive(Debug)]
    struct MockFailureType;

    impl Deserializable for MockFailureType {
        fn read_from<R: ByteReader>(_source: &mut R) -> Result<Self, DeserializationError> {
            Err(DeserializationError::InvalidValue("mock error".to_string()))
        }
    }

    #[test]
    fn deserialize_from_uint8array_failure_with_type_context() {
        // Create some invalid bytes
        let uint8_array = Uint8Array::new_with_length(10);

        // Try to deserialize with a type that always fails
        let result = deserialize_from_uint8array::<MockFailureType>(&uint8_array);

        assert!(result.is_err());
        let error = result.unwrap_err();

        // Verify the error contains the type name
        let error_string = error.as_string().unwrap();
        assert!(error_string.contains("MockFailureType"));
        assert!(error_string.contains("failed to deserialize"));
        assert!(error_string.contains("mock error"));
    }
}
