//! Typed view over a `Package`'s debug sections: signature rendering, argument encoding,
//! and result decoding for a single procedure.

mod decode;
mod encode;
mod errors;
mod format;
mod introspect;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use miden_mast_package::Package;
use miden_mast_package::debug_info::{
    DebugFunctionInfo,
    DebugFunctionsSection,
    DebugTypeIdx,
    DebugTypeInfo,
    DebugTypesSection,
};
use miden_protocol::Felt;

use self::decode::{decode_value, felts_for_type};
use self::encode::encode_for_type;
pub use self::encode::parse_felt_token;
pub use self::errors::PackageTypesError;
use self::format::format_type;
use self::introspect::{find_debug_fn, read_debug_sections};

// PUBLIC API
// ================================================================================================

pub struct TypedProcInfo {
    types: DebugTypesSection,
    name: String,
    return_type_idx: Option<DebugTypeIdx>,
    params: Vec<TypedParam>,
}

#[derive(Clone)]
struct TypedParam {
    name: String,
    type_idx: DebugTypeIdx,
}

impl TypedProcInfo {
    /// `None` if the package has no debug info or no entry for `procedure_name`.
    pub fn resolve(package: &Package, procedure_name: &str) -> Option<Self> {
        let (funcs, types) = read_debug_sections(package)?;
        let func = find_debug_fn(&funcs, procedure_name)?;
        let name = display_name(func, &funcs, procedure_name);
        let (return_type_idx, fallback_param_types) = extract_signature_types(func, &types);
        let params = build_params(func, &funcs, &fallback_param_types);
        Some(Self { types, name, return_type_idx, params })
    }

    /// `take-account-id(id: account-id) -> account-id`.
    pub fn format_signature(&self) -> String {
        let params = self
            .params
            .iter()
            .map(|p| format!("{}: {}", p.name, format_type(&self.types, p.type_idx, 0)))
            .collect::<Vec<_>>()
            .join(", ");
        let ret = self
            .return_type_idx
            .map(|i| format!(" -> {}", format_type(&self.types, i, 0)))
            .unwrap_or_default();
        format!("{}({}){}", self.name, params, ret)
    }

    /// Encodes `tokens` as a flat felt vector matching the procedure's parameter types. `word`
    /// and `account-id` each parse from a single hex token; other structs expect one token per
    /// leaf field.
    pub fn encode_args(&self, tokens: &[String]) -> Result<Vec<Felt>, PackageTypesError> {
        let mut iter = tokens.iter().cloned();
        let mut felts = Vec::new();
        for p in &self.params {
            felts.extend(encode_for_type(&mut iter, &self.types, p.type_idx)?);
        }
        if iter.next().is_some() {
            return Err(PackageTypesError::TooManyArgs);
        }
        Ok(felts)
    }

    /// Number of felts the procedure pushes on the stack for its return value. `Some(0)` if
    /// there is no return type; `None` if the return type has no statically-known felt size.
    pub fn result_felt_count(&self) -> Option<usize> {
        match self.return_type_idx {
            Some(idx) => felts_for_type(&self.types, idx),
            None => Some(0),
        }
    }

    /// Formats the procedure's return value as a string, reading its felts from the start of
    /// `stack`. Returns `None` if there is no return type, its felt size can't be determined,
    /// or `stack` is too short to hold it.
    pub fn decode_result(&self, stack: &[Felt]) -> Option<String> {
        let return_idx = self.return_type_idx?;
        let n = felts_for_type(&self.types, return_idx)?;
        if n == 0 || n > stack.len() {
            return None;
        }
        let slice = &stack[..n];
        let return_ty = self.types.types.get(return_idx.as_u32() as usize);
        let (rendered, rest) = decode_value(slice, &self.types, return_idx)?;
        if !rest.is_empty() {
            return None;
        }
        Some(match return_ty {
            Some(DebugTypeInfo::Primitive(p)) => {
                format!("{}({rendered})", format!("{p:?}").to_lowercase())
            },
            _ => rendered,
        })
    }
}

// HELPERS
// ================================================================================================

fn display_name(func: &DebugFunctionInfo, funcs: &DebugFunctionsSection, fallback: &str) -> String {
    funcs
        .strings
        .get(func.name_idx as usize)
        .map_or_else(|| fallback.to_string(), |s| s.as_ref().to_string())
}

/// `(return_type, param_types)` from the `Function`-typed debug entry, or `(None, vec![])` if
/// the entry has no Function type.
fn extract_signature_types(
    func: &DebugFunctionInfo,
    types: &DebugTypesSection,
) -> (Option<DebugTypeIdx>, Vec<DebugTypeIdx>) {
    match func.type_idx.and_then(|i| types.types.get(i.as_u32() as usize)) {
        Some(DebugTypeInfo::Function { return_type_idx, param_type_indices }) => {
            (*return_type_idx, param_type_indices.clone())
        },
        _ => (None, Vec::new()),
    }
}

/// Prefers named variables (`arg_index > 0`); falls back to `arg1`, `arg2`, ... paired with
/// the function type's positional indices.
fn build_params(
    func: &DebugFunctionInfo,
    funcs: &DebugFunctionsSection,
    fallback_indices: &[DebugTypeIdx],
) -> Vec<TypedParam> {
    // `DebugVariableInfo` entries are not necessarily in `arg_index` order.
    let mut named: Vec<(u32, String, DebugTypeIdx)> = func
        .variables
        .iter()
        .filter(|v| v.arg_index > 0)
        .map(|v| {
            let n = funcs
                .strings
                .get(v.name_idx as usize)
                .map_or_else(|| format!("arg{}", v.arg_index), |s| s.as_ref().to_string());
            (v.arg_index, n, v.type_idx)
        })
        .collect();
    named.sort_by_key(|(i, ..)| *i);

    if named.is_empty() {
        fallback_indices
            .iter()
            .enumerate()
            .map(|(i, t)| TypedParam {
                name: format!("arg{}", i + 1),
                type_idx: *t,
            })
            .collect()
    } else {
        named
            .into_iter()
            .map(|(_, name, type_idx)| TypedParam { name, type_idx })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use alloc::sync::Arc;
    use alloc::vec;

    use miden_mast_package::debug_info::{DebugFieldInfo, DebugPrimitiveType, DebugTypeInfo};

    use super::*;

    fn make_proc(
        types: DebugTypesSection,
        name: &str,
        return_type_idx: Option<DebugTypeIdx>,
        params: Vec<TypedParam>,
    ) -> TypedProcInfo {
        TypedProcInfo {
            types,
            name: name.to_string(),
            return_type_idx,
            params,
        }
    }

    fn felt(v: u64) -> Felt {
        Felt::try_from(v).unwrap()
    }

    #[test]
    fn felt_roundtrip() {
        let mut types = DebugTypesSection::new();
        let felt_idx = types.add_type(DebugTypeInfo::Primitive(DebugPrimitiveType::Felt));
        let proc = make_proc(
            types,
            "take-felt",
            Some(felt_idx),
            vec![TypedParam {
                name: "f".to_string(),
                type_idx: felt_idx,
            }],
        );

        let encoded = proc.encode_args(&["42".to_string()]).unwrap();
        assert_eq!(encoded, vec![felt(42)]);

        assert_eq!(proc.decode_result(&[felt(42)]).as_deref(), Some("felt(42)"));
        assert_eq!(proc.format_signature(), "take-felt(f: Felt) -> Felt",);
    }

    #[test]
    fn word_one_token_expands_to_four_felts() {
        let mut types = DebugTypesSection::new();
        let word_idx = types.add_type(DebugTypeInfo::Primitive(DebugPrimitiveType::Word));
        let proc = make_proc(
            types,
            "take-word",
            Some(word_idx),
            vec![TypedParam {
                name: "w".to_string(),
                type_idx: word_idx,
            }],
        );

        let hex = "0x0100000000000000020000000000000003000000000000000400000000000000";
        let encoded = proc.encode_args(&[hex.to_string()]).unwrap();
        assert_eq!(encoded.len(), 4);
        assert_eq!(encoded[0].as_canonical_u64(), 1);
        assert_eq!(encoded[3].as_canonical_u64(), 4);

        let decoded = proc.decode_result(&encoded).unwrap();
        assert_eq!(decoded, format!("word({hex})"));
    }

    #[test]
    fn account_id_one_hex_token_expands_and_roundtrips() {
        let mut types = DebugTypesSection::new();
        let felt_t = types.add_type(DebugTypeInfo::Primitive(DebugPrimitiveType::Felt));
        let aid_name = types.add_string(Arc::from("miden:base/core-types@1.0.0/account-id"));
        let prefix_n = types.add_string(Arc::from("prefix"));
        let suffix_n = types.add_string(Arc::from("suffix"));
        let aid_idx = types.add_type(DebugTypeInfo::Struct {
            name_idx: aid_name,
            size: 16,
            fields: vec![
                DebugFieldInfo {
                    name_idx: prefix_n,
                    type_idx: felt_t,
                    offset: 0,
                },
                DebugFieldInfo {
                    name_idx: suffix_n,
                    type_idx: felt_t,
                    offset: 8,
                },
            ],
        });
        let proc = make_proc(
            types,
            "take-account-id",
            Some(aid_idx),
            vec![TypedParam {
                name: "id".to_string(),
                type_idx: aid_idx,
            }],
        );

        let hex = "0xa591009a3022e800788f9ed177dcdb";
        let encoded = proc.encode_args(&[hex.to_string()]).unwrap();
        assert_eq!(encoded.len(), 2);

        let decoded = proc.decode_result(&encoded).unwrap();
        assert_eq!(decoded, format!("account-id({hex})"));

        assert_eq!(proc.format_signature(), "take-account-id(id: account-id) -> account-id",);
    }

    #[test]
    fn anonymous_struct_falls_back_to_field_shape() {
        let mut types = DebugTypesSection::new();
        let felt_t = types.add_type(DebugTypeInfo::Primitive(DebugPrimitiveType::Felt));
        let anon = types.add_string(Arc::from("<anonymous>"));
        let x = types.add_string(Arc::from("x"));
        let y = types.add_string(Arc::from("y"));
        let point = types.add_type(DebugTypeInfo::Struct {
            name_idx: anon,
            size: 8,
            fields: vec![
                DebugFieldInfo { name_idx: x, type_idx: felt_t, offset: 0 },
                DebugFieldInfo { name_idx: y, type_idx: felt_t, offset: 4 },
            ],
        });
        let proc = make_proc(
            types,
            "take-point",
            Some(point),
            vec![TypedParam { name: "p".to_string(), type_idx: point }],
        );

        assert_eq!(
            proc.format_signature(),
            "take-point(p: {x: Felt, y: Felt}) -> {x: Felt, y: Felt}",
        );
        assert_eq!(proc.decode_result(&[felt(3), felt(4)]).as_deref(), Some("{x=3, y=4}"));
    }

    #[test]
    fn arg_count_mismatch_errors() {
        let mut types = DebugTypesSection::new();
        let felt_idx = types.add_type(DebugTypeInfo::Primitive(DebugPrimitiveType::Felt));
        let proc = make_proc(
            types,
            "take-felt",
            Some(felt_idx),
            vec![TypedParam {
                name: "f".to_string(),
                type_idx: felt_idx,
            }],
        );

        assert!(matches!(proc.encode_args(&[]), Err(PackageTypesError::NotEnoughArgs)));
        assert!(matches!(
            proc.encode_args(&["1".to_string(), "2".to_string()]),
            Err(PackageTypesError::TooManyArgs)
        ));
    }
}
