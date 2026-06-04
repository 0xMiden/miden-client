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

use self::decode::{decode_value, stack_felt_count};
pub use self::encode::parse_felt_token;
use self::encode::{arg_token_count, encode_tokens};
pub use self::errors::PackageDebugInfoError;
use self::format::format_type;
use self::introspect::{find_debug_fn, proc_display_name, read_debug_sections};

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
        let name = func_display_name(func, &funcs, procedure_name);
        let (return_type_idx, fallback_param_types) = extract_signature_types(func, &types);
        let params = build_params(func, &funcs, &fallback_param_types);
        Some(Self { types, name, return_type_idx, params })
    }

    /// `get-count() -> felt`.
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

    /// Number of argument tokens the procedure expects, summed over its parameters (`word` and
    /// `account-id` take one token each; other structs take one per leaf field). `None` if any
    /// parameter has no statically-known token count.
    pub fn expected_arg_count(&self) -> Option<usize> {
        self.params.iter().map(|p| arg_token_count(&self.types, p.type_idx)).sum()
    }

    /// Encodes `tokens` as a flat felt vector matching the procedure's parameter types. `word`
    /// and `account-id` each parse from a single hex token; other structs expect one token per
    /// leaf field.
    pub fn encode_args(&self, tokens: &[String]) -> Result<Vec<Felt>, PackageDebugInfoError> {
        if let Some(expected) = self.expected_arg_count()
            && tokens.len() != expected
        {
            return Err(PackageDebugInfoError::WrongArgCount { expected, got: tokens.len() });
        }

        let mut iter = tokens.iter().cloned();
        let mut felts = Vec::new();
        for p in &self.params {
            felts.extend(encode_tokens(&mut iter, &self.types, p.type_idx)?);
        }
        if iter.next().is_some() {
            return Err(PackageDebugInfoError::TooManyArgs);
        }
        Ok(felts)
    }

    /// Number of felts the procedure pushes on the stack for its return value. `Some(0)` if
    /// there is no return type; `None` if the return type has no statically-known felt size.
    pub fn return_value_felt_count(&self) -> Option<usize> {
        match self.return_type_idx {
            Some(idx) => stack_felt_count(&self.types, idx),
            None => Some(0),
        }
    }

    /// Formats the procedure's return value as a string, reading its felts from the start of
    /// `stack`. Returns `None` if there is no return type, its felt size can't be determined,
    /// or `stack` is too short to hold it.
    pub fn decode_result(&self, stack: &[Felt]) -> Option<String> {
        let return_idx = self.return_type_idx?;
        let n = stack_felt_count(&self.types, return_idx)?;
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

fn func_display_name(
    func: &DebugFunctionInfo,
    funcs: &DebugFunctionsSection,
    fallback: &str,
) -> String {
    funcs
        .strings
        .get(func.name_idx as usize)
        .map_or_else(|| fallback.to_string(), |s| proc_display_name(s.as_ref()).to_string())
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
    fn bool_roundtrip() {
        let mut types = DebugTypesSection::new();
        let bool_idx = types.add_type(DebugTypeInfo::Primitive(DebugPrimitiveType::Bool));
        let proc = make_proc(
            types,
            "take-bool",
            Some(bool_idx),
            vec![TypedParam {
                name: "b".to_string(),
                type_idx: bool_idx,
            }],
        );

        assert_eq!(proc.encode_args(&["true".to_string()]).unwrap(), vec![felt(1)]);
        assert_eq!(proc.encode_args(&["false".to_string()]).unwrap(), vec![felt(0)]);

        assert_eq!(proc.decode_result(&[felt(1)]).as_deref(), Some("bool(true)"));
        assert_eq!(proc.decode_result(&[felt(0)]).as_deref(), Some("bool(false)"));
        assert_eq!(proc.format_signature(), "take-bool(b: Bool) -> Bool",);
    }

    #[test]
    fn u32_roundtrip() {
        let mut types = DebugTypesSection::new();
        let u32_idx = types.add_type(DebugTypeInfo::Primitive(DebugPrimitiveType::U32));
        let proc = make_proc(
            types,
            "take-u32",
            Some(u32_idx),
            vec![TypedParam { name: "n".to_string(), type_idx: u32_idx }],
        );

        let encoded = proc.encode_args(&["4294967295".to_string()]).unwrap();
        assert_eq!(encoded, vec![felt(4_294_967_295)]);

        assert_eq!(proc.decode_result(&[felt(4_294_967_295)]).as_deref(), Some("u32(4294967295)"));
        assert_eq!(proc.format_signature(), "take-u32(n: U32) -> U32",);
    }

    #[test]
    fn u64_roundtrip() {
        let mut types = DebugTypesSection::new();
        let u64_idx = types.add_type(DebugTypeInfo::Primitive(DebugPrimitiveType::U64));
        let proc = make_proc(
            types,
            "take-u64",
            Some(u64_idx),
            vec![TypedParam { name: "n".to_string(), type_idx: u64_idx }],
        );

        let encoded = proc.encode_args(&["1234567890123".to_string()]).unwrap();
        assert_eq!(encoded, vec![felt(1_234_567_890_123)]);

        assert_eq!(
            proc.decode_result(&[felt(1_234_567_890_123)]).as_deref(),
            Some("u64(1234567890123)")
        );
        assert_eq!(proc.format_signature(), "take-u64(n: U64) -> U64",);
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
    fn word_struct_one_hex_token_roundtrips() {
        let mut types = DebugTypesSection::new();
        let felt_t = types.add_type(DebugTypeInfo::Primitive(DebugPrimitiveType::Felt));
        let f_n = types.add_string(Arc::from("f"));
        let word_name = types.add_string(Arc::from("miden:base/core-types@1.0.0/word"));
        let word_t = types.add_type(DebugTypeInfo::Struct {
            name_idx: word_name,
            size: 32,
            fields: vec![
                DebugFieldInfo {
                    name_idx: f_n,
                    type_idx: felt_t,
                    offset: 0,
                },
                DebugFieldInfo {
                    name_idx: f_n,
                    type_idx: felt_t,
                    offset: 8,
                },
                DebugFieldInfo {
                    name_idx: f_n,
                    type_idx: felt_t,
                    offset: 16,
                },
                DebugFieldInfo {
                    name_idx: f_n,
                    type_idx: felt_t,
                    offset: 24,
                },
            ],
        });
        let proc = make_proc(
            types,
            "take-word",
            Some(word_t),
            vec![TypedParam { name: "w".to_string(), type_idx: word_t }],
        );

        let hex = "0x0100000000000000020000000000000003000000000000000400000000000000";
        let encoded = proc.encode_args(&[hex.to_string()]).unwrap();
        assert_eq!(encoded.len(), 4);
        assert_eq!(encoded[0].as_canonical_u64(), 1);
        assert_eq!(encoded[3].as_canonical_u64(), 4);

        assert_eq!(proc.decode_result(&encoded), Some(format!("word({hex})")));
        assert_eq!(proc.format_signature(), "take-word(w: word) -> word");
    }

    /// Feeds exactly `arg_token_count` tokens, then checks `encode_tokens` reads all of them and
    /// produces `stack_felt_count` felts. Keeps the three functions in sync: a special case added
    /// to one (e.g. `void`, `account-id`) but not the others fails here.
    fn assert_counts_consistent(types: &DebugTypesSection, idx: DebugTypeIdx, tokens: &[&str]) {
        let want_tokens = arg_token_count(types, idx).expect("static token count");
        assert_eq!(want_tokens, tokens.len(), "test must feed exactly the expected token count");

        let mut iter = tokens.iter().map(ToString::to_string);
        let felts = encode_tokens(&mut iter, types, idx).unwrap();
        assert!(iter.next().is_none(), "encode_tokens must consume every token");
        assert_eq!(
            Some(felts.len()),
            stack_felt_count(types, idx),
            "stack_felt_count must equal the felts encode_tokens produces"
        );
    }

    #[test]
    fn token_and_felt_counts_match_actual_consumption() {
        let mut types = DebugTypesSection::new();
        let felt_t = types.add_type(DebugTypeInfo::Primitive(DebugPrimitiveType::Felt));
        let word_t = types.add_type(DebugTypeInfo::Primitive(DebugPrimitiveType::Word));
        let void_t = types.add_type(DebugTypeInfo::Primitive(DebugPrimitiveType::Void));
        let f_n = types.add_string(Arc::from("f"));

        // `account-id` is special-cased to 1 token / 2 felts in all three functions.
        let aid_name = types.add_string(Arc::from("miden:base/core-types@1.0.0/account-id"));
        let aid_t = types.add_type(DebugTypeInfo::Struct {
            name_idx: aid_name,
            size: 16,
            fields: vec![
                DebugFieldInfo {
                    name_idx: f_n,
                    type_idx: felt_t,
                    offset: 0,
                },
                DebugFieldInfo {
                    name_idx: f_n,
                    type_idx: felt_t,
                    offset: 8,
                },
            ],
        });

        // The WIT `word` struct is special-cased to 1 token / 4 felts, like the `Word` primitive.
        let word_name = types.add_string(Arc::from("miden:base/core-types@1.0.0/word"));
        let word_struct_t = types.add_type(DebugTypeInfo::Struct {
            name_idx: word_name,
            size: 32,
            fields: vec![
                DebugFieldInfo {
                    name_idx: f_n,
                    type_idx: felt_t,
                    offset: 0,
                },
                DebugFieldInfo {
                    name_idx: f_n,
                    type_idx: felt_t,
                    offset: 8,
                },
                DebugFieldInfo {
                    name_idx: f_n,
                    type_idx: felt_t,
                    offset: 16,
                },
                DebugFieldInfo {
                    name_idx: f_n,
                    type_idx: felt_t,
                    offset: 24,
                },
            ],
        });

        // A struct mixing a zero-token `void` field with a `felt` field.
        let mixed_name = types.add_string(Arc::from("pkg/mixed"));
        let mixed_t = types.add_type(DebugTypeInfo::Struct {
            name_idx: mixed_name,
            size: 8,
            fields: vec![
                DebugFieldInfo {
                    name_idx: f_n,
                    type_idx: void_t,
                    offset: 0,
                },
                DebugFieldInfo {
                    name_idx: f_n,
                    type_idx: felt_t,
                    offset: 0,
                },
            ],
        });

        let hex_word = "0x0100000000000000020000000000000003000000000000000400000000000000";
        let hex_aid = "0xa591009a3022e800788f9ed177dcdb";

        assert_counts_consistent(&types, felt_t, &["7"]);
        assert_counts_consistent(&types, word_t, &[hex_word]);
        assert_counts_consistent(&types, void_t, &[]);
        assert_counts_consistent(&types, aid_t, &[hex_aid]);
        assert_counts_consistent(&types, word_struct_t, &[hex_word]);
        assert_counts_consistent(&types, mixed_t, &["7"]);
    }

    #[test]
    fn anonymous_struct_falls_back_to_field_shape() {
        let mut types = DebugTypesSection::new();
        let felt_t = types.add_type(DebugTypeInfo::Primitive(DebugPrimitiveType::Felt));
        let anon = types.add_string(Arc::from("<anon>"));
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

        assert!(matches!(
            proc.encode_args(&[]),
            Err(PackageDebugInfoError::WrongArgCount { expected: 1, got: 0 })
        ));
        assert!(matches!(
            proc.encode_args(&["1".to_string(), "2".to_string()]),
            Err(PackageDebugInfoError::WrongArgCount { expected: 1, got: 2 })
        ));
    }
}
