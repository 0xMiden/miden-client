use miden_client::SliceReader;
use miden_client::utils::{Deserializable, Serializable};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

use crate::js_error_with_context;

pub mod assembler_utils;

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
    use miden_client::utils::{ByteReader, ByteWriter, DeserializationError};

    use super::*;

    // Mock types for testing
    #[derive(Debug, PartialEq, Eq)]
    struct MockSuccessType {
        value: u32,
    }

    impl Serializable for MockSuccessType {
        fn write_into<W: ByteWriter>(&self, target: &mut W) {
            target.write_u32(self.value);
        }
    }

    impl Deserializable for MockSuccessType {
        fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
            let value = source.read_u32()?;
            Ok(MockSuccessType { value })
        }
    }

    #[derive(Debug)]
    struct MockFailureType;

    impl Deserializable for MockFailureType {
        fn read_from<R: ByteReader>(_source: &mut R) -> Result<Self, DeserializationError> {
            Err(DeserializationError::InvalidValue("mock error".to_string()))
        }
    }

    #[derive(Debug)]
    struct MockInsufficientDataType;

    impl Deserializable for MockInsufficientDataType {
        fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
            // Try to read more data than available
            source.read_u64()?;
            source.read_u64()?;
            source.read_u64()?;
            Ok(MockInsufficientDataType)
        }
    }

    #[test]
    fn test_serialize_to_uint8array() {
        let mock_data = MockSuccessType { value: 42 };
        let uint8_array = serialize_to_uint8array(&mock_data);

        // Verify the array has the expected length (4 bytes for u32)
        assert_eq!(uint8_array.length(), 4);

        // Verify the content
        let vec = uint8_array.to_vec();
        assert_eq!(vec, vec![42, 0, 0, 0]); // Little-endian representation of 42
    }

    #[test]
    fn test_deserialize_from_uint8array_success() {
        // Create valid data for MockSuccessType
        let mock_data = MockSuccessType { value: 123 };
        let uint8_array = serialize_to_uint8array(&mock_data);

        // Deserialize it back
        let result = deserialize_from_uint8array::<MockSuccessType>(&uint8_array);

        assert!(result.is_ok());
        let deserialized = result.unwrap();
        assert_eq!(deserialized.value, 123);
    }

    #[test]
    fn test_deserialize_from_uint8array_failure_with_type_context() {
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

    #[test]
    fn test_deserialize_from_uint8array_insufficient_data() {
        // Create insufficient data (only 4 bytes, but MockInsufficientDataType needs 24)
        let uint8_array = Uint8Array::new_with_length(4);

        let result = deserialize_from_uint8array::<MockInsufficientDataType>(&uint8_array);

        assert!(result.is_err());
        let error = result.unwrap_err();

        // Verify the error contains the type name
        let error_string = error.as_string().unwrap();
        assert!(error_string.contains("MockInsufficientDataType"));
        assert!(error_string.contains("failed to deserialize"));
    }

    #[test]
    fn test_round_trip_serialization() {
        let original = MockSuccessType { value: 999 };

        // Serialize
        let uint8_array = serialize_to_uint8array(&original);

        // Deserialize
        let result = deserialize_from_uint8array::<MockSuccessType>(&uint8_array);

        assert!(result.is_ok());
        let deserialized = result.unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_empty_uint8array() {
        let empty_array = Uint8Array::new_with_length(0);

        let result = deserialize_from_uint8array::<MockSuccessType>(&empty_array);

        assert!(result.is_err());
        let error = result.unwrap_err();
        let error_string = error.as_string().unwrap();
        assert!(error_string.contains("MockSuccessType"));
        assert!(error_string.contains("failed to deserialize"));
    }
}
