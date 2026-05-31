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

use super::PackageDebugInfoError;
use super::introspect::{is_account_id_type, is_word_type, type_name_raw};

/// Encodes the CLI tokens for a value of type `idx` into its stack felts.
/// Consumes exactly `arg_token_count(idx)` tokens and produces `stack_felt_count(idx)` felts.
pub(super) fn encode_tokens<I: Iterator<Item = String>>(
    tokens: &mut I,
    types: &DebugTypesSection,
    idx: DebugTypeIdx,
) -> Result<Vec<Felt>, PackageDebugInfoError> {
    let ty = types
        .types
        .get(idx.as_u32() as usize)
        .ok_or(PackageDebugInfoError::MissingType(idx.as_u32()))?;
    match ty {
        DebugTypeInfo::Primitive(DebugPrimitiveType::Void) => Ok(Vec::new()),
        DebugTypeInfo::Primitive(p) => encode_primitive(next_token(tokens)?, *p),
        DebugTypeInfo::Struct { name_idx, fields, .. } => {
            let name = type_name_raw(types, *name_idx);
            if is_account_id_type(name) {
                return encode_account_id(&next_token(tokens)?);
            }
            if is_word_type(name) {
                return encode_primitive(next_token(tokens)?, DebugPrimitiveType::Word);
            }
            let mut felts = Vec::new();
            for f in fields {
                felts.extend(encode_tokens(tokens, types, f.type_idx)?);
            }
            Ok(felts)
        },
        DebugTypeInfo::Array { element_type_idx, count: Some(n) } => {
            let mut felts = Vec::new();
            for _ in 0..*n {
                felts.extend(encode_tokens(tokens, types, *element_type_idx)?);
            }
            Ok(felts)
        },
        DebugTypeInfo::Array { count: None, .. } => {
            Err(PackageDebugInfoError::UnsupportedType("array"))
        },
        // No defined encoding for bellow types params from the CLI.
        DebugTypeInfo::Pointer { .. } => Err(PackageDebugInfoError::UnsupportedType("pointer")),
        DebugTypeInfo::Function { .. } => Err(PackageDebugInfoError::UnsupportedType("function")),
        DebugTypeInfo::Unknown => Err(PackageDebugInfoError::UnsupportedType("unknown")),
    }
}

/// Number of CLI tokens `encode_tokens` reads for `idx`; must match it exactly. One token per
/// primitive, `void` none, an `account-id` or `word` struct one, other structs
/// and fixed arrays the sum of their leaves. `None` when the count isn't static (dynamic array,
/// pointer, function, unknown); the caller then skips the upfront count check.
pub(super) fn arg_token_count(types: &DebugTypesSection, idx: DebugTypeIdx) -> Option<usize> {
    let ty = types.types.get(idx.as_u32() as usize)?;
    match ty {
        DebugTypeInfo::Primitive(DebugPrimitiveType::Void) => Some(0),
        DebugTypeInfo::Primitive(_) => Some(1),
        DebugTypeInfo::Struct { name_idx, fields, .. } => {
            let name = type_name_raw(types, *name_idx);
            if is_account_id_type(name) || is_word_type(name) {
                return Some(1);
            }
            let mut total = 0;
            for f in fields {
                total += arg_token_count(types, f.type_idx)?;
            }
            Some(total)
        },
        DebugTypeInfo::Array { element_type_idx, count: Some(n) } => {
            Some(arg_token_count(types, *element_type_idx)? * *n as usize)
        },
        DebugTypeInfo::Array { count: None, .. }
        | DebugTypeInfo::Pointer { .. }
        | DebugTypeInfo::Function { .. }
        | DebugTypeInfo::Unknown => None,
    }
}

fn encode_account_id(token: &str) -> Result<Vec<Felt>, PackageDebugInfoError> {
    let id = AccountId::from_hex(token).map_err(|e| PackageDebugInfoError::InvalidAccountId {
        token: token.to_string(),
        source: e,
    })?;
    let [prefix, suffix]: [Felt; 2] = id.into();
    Ok(alloc::vec![prefix, suffix])
}

fn encode_primitive(
    token: String,
    p: DebugPrimitiveType,
) -> Result<Vec<Felt>, PackageDebugInfoError> {
    match p {
        // Compiler-built packages emit `word` as a struct (see `is_word_type`); this arm fires
        // only for a core `Word`. The struct path reuses it.
        DebugPrimitiveType::Word => {
            let w = Word::try_from(token.as_str()).map_err(|e| {
                PackageDebugInfoError::InvalidWord { token: token.clone(), source: e }
            })?;
            Ok(w.to_vec())
        },
        DebugPrimitiveType::Bool => {
            let v = match token.to_ascii_lowercase().as_str() {
                "true" | "1" => 1u64,
                "false" | "0" => 0,
                _ => return Err(PackageDebugInfoError::InvalidBool(token)),
            };
            Felt::try_from(v)
                .map(|f| alloc::vec![f])
                .map_err(|_| PackageDebugInfoError::FeltOutOfRange(token))
        },
        DebugPrimitiveType::Void => Ok(Vec::new()),
        _ => Ok(alloc::vec![parse_felt_token(&token)?]),
    }
}

/// Decimal or `0x..` hex CLI token to a `Felt`. Shared between the typed and raw arg parsers.
pub fn parse_felt_token(s: &str) -> Result<Felt, PackageDebugInfoError> {
    let v: u64 = if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u64::from_str_radix(hex, 16)
            .map_err(|_| PackageDebugInfoError::InvalidHex(s.to_string()))?
    } else {
        s.parse::<u64>().map_err(|_| PackageDebugInfoError::InvalidU64(s.to_string()))?
    };
    Felt::try_from(v).map_err(|_| PackageDebugInfoError::FeltOutOfRange(s.to_string()))
}

fn next_token<I: Iterator<Item = String>>(tokens: &mut I) -> Result<String, PackageDebugInfoError> {
    tokens.next().ok_or(PackageDebugInfoError::NotEnoughArgs)
}
