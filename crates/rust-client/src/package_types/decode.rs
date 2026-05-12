//! Decodes felts into a structured string using debug type info.

use alloc::format;
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

use super::introspect::{is_account_id_type, wit_short_name};

/// `None` if `idx` has no static size (e.g. dynamic arrays, `Unknown`).
pub(super) fn felts_for_type(types: &DebugTypesSection, idx: DebugTypeIdx) -> Option<usize> {
    let ty = types.types.get(idx.as_u32() as usize)?;
    match ty {
        DebugTypeInfo::Primitive(p) => Some(match p {
            DebugPrimitiveType::Word => 4,
            DebugPrimitiveType::Void => 0,
            _ => 1,
        }),
        DebugTypeInfo::Array { element_type_idx, count } => {
            let elem = felts_for_type(types, *element_type_idx)?;
            count.map(|n| elem * n as usize)
        },
        DebugTypeInfo::Struct { fields, .. } => {
            let mut total = 0;
            for f in fields {
                total += felts_for_type(types, f.type_idx)?;
            }
            Some(total)
        },
        DebugTypeInfo::Pointer { .. } | DebugTypeInfo::Function { .. } | DebugTypeInfo::Unknown => {
            None
        },
    }
}

/// Returns `(body, leftover)`. The body for primitives omits the outer type tag.
pub(super) fn decode_value<'a>(
    felts: &'a [Felt],
    types: &DebugTypesSection,
    idx: DebugTypeIdx,
) -> Option<(String, &'a [Felt])> {
    let ty = types.types.get(idx.as_u32() as usize)?;
    match ty {
        DebugTypeInfo::Primitive(p) => decode_primitive(felts, *p),
        DebugTypeInfo::Struct { name_idx, fields, .. } => {
            let full = types.strings.get(*name_idx as usize).map_or("", AsRef::as_ref);
            let short = wit_short_name(full);
            if is_account_id_type(full)
                && let Some((rendered, rest)) = decode_account_id(felts)
            {
                return Some((rendered, rest));
            }
            if let [only] = fields.as_slice() {
                let (inner, rest) = decode_value(felts, types, only.type_idx)?;
                return Some((wrap_struct(short, &inner), rest));
            }
            let mut cursor = felts;
            let mut rendered = Vec::with_capacity(fields.len());
            for f in fields {
                let fname = types
                    .strings
                    .get(f.name_idx as usize)
                    .map_or_else(|| format!("f{}", f.name_idx), |s| s.as_ref().to_string());
                let (fv, rest) = decode_value(cursor, types, f.type_idx)?;
                rendered.push(format!("{fname}={fv}"));
                cursor = rest;
            }
            Some((wrap_struct(short, &rendered.join(", ")), cursor))
        },
        DebugTypeInfo::Array { element_type_idx, count: Some(n) } => {
            let mut cursor = felts;
            let mut rendered = Vec::with_capacity(*n as usize);
            for _ in 0..*n {
                let (v, rest) = decode_value(cursor, types, *element_type_idx)?;
                rendered.push(v);
                cursor = rest;
            }
            Some((format!("[{}]", rendered.join(", ")), cursor))
        },
        DebugTypeInfo::Array { count: None, .. }
        | DebugTypeInfo::Pointer { .. }
        | DebugTypeInfo::Function { .. }
        | DebugTypeInfo::Unknown => None,
    }
}

fn decode_primitive(felts: &[Felt], p: DebugPrimitiveType) -> Option<(String, &[Felt])> {
    match p {
        DebugPrimitiveType::Void => Some((String::from("()"), felts)),
        DebugPrimitiveType::Word => {
            if felts.len() < 4 {
                return None;
            }
            let (chunk, rest) = felts.split_at(4);
            let word = Word::from([chunk[0], chunk[1], chunk[2], chunk[3]]);
            Some((word.to_hex(), rest))
        },
        DebugPrimitiveType::Felt => {
            let (head, rest) = felts.split_first()?;
            Some((format!("{head}"), rest))
        },
        DebugPrimitiveType::Bool => {
            let (head, rest) = felts.split_first()?;
            let v = head.as_canonical_u64();
            Some((if v == 0 { "false".into() } else { "true".into() }, rest))
        },
        DebugPrimitiveType::I8
        | DebugPrimitiveType::I16
        | DebugPrimitiveType::I32
        | DebugPrimitiveType::I64
        | DebugPrimitiveType::U8
        | DebugPrimitiveType::U16
        | DebugPrimitiveType::U32
        | DebugPrimitiveType::U64 => {
            let (head, rest) = felts.split_first()?;
            Some((format!("{}", head.as_canonical_u64()), rest))
        },
        DebugPrimitiveType::I128
        | DebugPrimitiveType::U128
        | DebugPrimitiveType::F32
        | DebugPrimitiveType::F64 => {
            let (head, rest) = felts.split_first()?;
            Some((format!("{} (as {p:?})", head.as_canonical_u64()), rest))
        },
    }
}

fn decode_account_id(felts: &[Felt]) -> Option<(String, &[Felt])> {
    if felts.len() < 2 {
        return None;
    }
    let (chunk, rest) = felts.split_at(2);
    let id = AccountId::try_from_elements(chunk[1], chunk[0]).ok()?;
    Some((format!("account-id({})", id.to_hex()), rest))
}

/// `name(body)` for named structs; `{body}` for anonymous or unnamed ones.
fn wrap_struct(short: &str, body: &str) -> String {
    if short.is_empty() || short == "<anonymous>" {
        format!("{{{body}}}")
    } else {
        format!("{short}({body})")
    }
}
