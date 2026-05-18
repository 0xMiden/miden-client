//! Encodes CLI tokens into felts using debug type info.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use miden_mast_package::debug_info::{
    DebugPrimitiveType,
    DebugTypeIdx,
    DebugTypeInfo,
    DebugTypesSection,
};
use miden_protocol::account::AccountId;
use miden_protocol::{Felt, Word};

use super::PackageTypesError;
use super::introspect::is_account_id_type;

pub(super) fn encode_for_type<I: Iterator<Item = String>>(
    tokens: &mut I,
    types: &DebugTypesSection,
    idx: DebugTypeIdx,
) -> Result<Vec<Felt>, PackageTypesError> {
    let ty = types
        .types
        .get(idx.as_u32() as usize)
        .ok_or(PackageTypesError::MissingType(idx.as_u32()))?;
    match ty {
        DebugTypeInfo::Primitive(p) => encode_primitive(next_token(tokens)?, *p),
        DebugTypeInfo::Struct { name_idx, fields, .. } => {
            let name = types.strings.get(*name_idx as usize).map_or("", AsRef::as_ref);
            if is_account_id_type(name) {
                return encode_account_id(&next_token(tokens)?);
            }
            let mut felts = Vec::new();
            for f in fields {
                felts.extend(encode_for_type(tokens, types, f.type_idx)?);
            }
            Ok(felts)
        },
        DebugTypeInfo::Array { element_type_idx, count: Some(n) } => {
            let mut felts = Vec::new();
            for _ in 0..*n {
                felts.extend(encode_for_type(tokens, types, *element_type_idx)?);
            }
            Ok(felts)
        },
        DebugTypeInfo::Array { count: None, .. } => {
            Err(PackageTypesError::UnsupportedType("array"))
        },
        // No defined encoding for pointer params from the CLI.
        DebugTypeInfo::Pointer { .. } => Err(PackageTypesError::UnsupportedType("pointer")),
        DebugTypeInfo::Function { .. } => Err(PackageTypesError::UnsupportedType("function")),
        DebugTypeInfo::Unknown => Err(PackageTypesError::UnsupportedType("unknown")),
    }
}

fn encode_primitive(token: String, p: DebugPrimitiveType) -> Result<Vec<Felt>, PackageTypesError> {
    match p {
        DebugPrimitiveType::Word => {
            let w = Word::try_from(token.as_str())
                .map_err(|e| PackageTypesError::InvalidWord { token: token.clone(), source: e })?;
            Ok(w.to_vec())
        },
        DebugPrimitiveType::Bool => {
            let v = match token.to_ascii_lowercase().as_str() {
                "true" | "1" => 1u64,
                "false" | "0" => 0,
                _ => return Err(PackageTypesError::InvalidBool(token)),
            };
            Felt::try_from(v)
                .map(|f| alloc::vec![f])
                .map_err(|_| PackageTypesError::FeltOutOfRange(token))
        },
        DebugPrimitiveType::Void => Ok(Vec::new()),
        _ => Ok(alloc::vec![parse_felt_token(&token)?]),
    }
}

/// Decimal or `0x..` hex CLI token to a `Felt`. Shared between the typed and raw arg parsers.
pub fn parse_felt_token(s: &str) -> Result<Felt, PackageTypesError> {
    let v: u64 = if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u64::from_str_radix(hex, 16).map_err(|_| PackageTypesError::InvalidHex(s.to_string()))?
    } else {
        s.parse::<u64>().map_err(|_| PackageTypesError::InvalidU64(s.to_string()))?
    };
    Felt::try_from(v).map_err(|_| PackageTypesError::FeltOutOfRange(s.to_string()))
}

fn next_token<I: Iterator<Item = String>>(tokens: &mut I) -> Result<String, PackageTypesError> {
    tokens.next().ok_or(PackageTypesError::NotEnoughArgs)
}

fn encode_account_id(token: &str) -> Result<Vec<Felt>, PackageTypesError> {
    let id = AccountId::from_hex(token)
        .map_err(|e| PackageTypesError::InvalidAccountId { token: token.to_string(), source: e })?;
    let [prefix, suffix]: [Felt; 2] = id.into();
    Ok(alloc::vec![prefix, suffix])
}
