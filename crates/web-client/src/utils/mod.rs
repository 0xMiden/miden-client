use miden_client::SliceReader;
use miden_client::utils::{Deserializable, Serializable};
use miden_objects::utils::{DeserializationError, SliceReader};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Uint8Array;

pub mod assembler_utils;

#[cfg(feature = "testing")]
pub mod test_utils;

/// Error type for deserialization that includes type context.
#[derive(Debug)]
pub struct TypedDeserializationError {
    pub type_name: &'static str,
    pub source: DeserializationError,
}

impl std::fmt::Display for TypedDeserializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to deserialize {}: {}", self.type_name, self.source)
    }
}

impl std::error::Error for TypedDeserializationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

impl From<TypedDeserializationError> for JsValue {
    fn from(err: TypedDeserializationError) -> Self {
        JsValue::from(err.to_string())
    }
}

/// Serializes any value that implements `Serializable` into a `Uint8Array`.
pub fn serialize_to_uint8array<T: Serializable>(value: &T) -> Uint8Array {
    let mut buffer = Vec::new();
    // Call the trait method to write into the buffer.
    value.write_into(&mut buffer);
    Uint8Array::from(&buffer[..])
}

/// Deserializes a `Uint8Array` into any type that implements `Deserializable`.
/// Returns a `TypedDeserializationError` that includes the type name for better error context.
pub fn deserialize_from_uint8array<T: Deserializable>(
    bytes: &Uint8Array,
) -> Result<T, TypedDeserializationError> {
    let vec = bytes.to_vec();
    let mut reader = SliceReader::new(&vec);
    T::read_from(&mut reader).map_err(|source| TypedDeserializationError {
        type_name: std::any::type_name::<T>(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use miden_objects::utils::{ByteReader, ByteWriter};

    use super::*;

    // Mock types for testing
    #[derive(Debug, PartialEq)]
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
        let error_string = error.to_string();
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
        let error_string = error.to_string();
        assert!(error_string.contains("MockInsufficientDataType"));
        assert!(error_string.contains("failed to deserialize"));
    }

    #[test]
    fn test_typed_deserialization_error_display() {
        let source_error = DeserializationError::InvalidValue("test error message".to_string());
        let typed_error = TypedDeserializationError {
            type_name: "TestType",
            source: source_error,
        };

        let display_string = typed_error.to_string();
        assert_eq!(display_string, "failed to deserialize TestType: test error message");
    }

    #[test]
    fn test_typed_deserialization_error_source() {
        let source_error = DeserializationError::InvalidValue("test error".to_string());
        let typed_error = TypedDeserializationError {
            type_name: "TestType",
            source: source_error,
        };

        // Verify that the source error is accessible
        assert!(typed_error.source().is_some());
    }

    #[test]
    fn test_typed_deserialization_error_to_js_value() {
        let source_error = DeserializationError::InvalidValue("test error".to_string());
        let typed_error = TypedDeserializationError {
            type_name: "TestType",
            source: source_error,
        };

        let js_value: JsValue = typed_error.into();
        let js_string = js_value.as_string().unwrap();
        assert_eq!(js_string, "failed to deserialize TestType: test error");
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
        let error_string = error.to_string();
        assert!(error_string.contains("MockSuccessType"));
        assert!(error_string.contains("failed to deserialize"));
    }
}
