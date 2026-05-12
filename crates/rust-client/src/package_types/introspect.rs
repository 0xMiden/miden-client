//! Read-only helpers over a `Package`'s debug sections: section loading, function lookup,
//! and shared type-name predicates.

use alloc::format;

use miden_mast_package::debug_info::{DebugFunctionInfo, DebugFunctionsSection, DebugTypesSection};
use miden_mast_package::{Package, Section, SectionId};
use miden_protocol::utils::serde::Deserializable;

/// Reads both debug sections from `package`. `None` if either is missing or fails to decode.
pub(super) fn read_debug_sections(
    package: &Package,
) -> Option<(DebugFunctionsSection, DebugTypesSection)> {
    let mut funcs_section: Option<&Section> = None;
    let mut types_section: Option<&Section> = None;
    for s in &package.sections {
        if s.id == SectionId::DEBUG_FUNCTIONS {
            funcs_section = Some(s);
        } else if s.id == SectionId::DEBUG_TYPES {
            types_section = Some(s);
        }
    }
    let funcs = DebugFunctionsSection::read_from_bytes(&funcs_section?.data).ok()?;
    let types = DebugTypesSection::read_from_bytes(&types_section?.data).ok()?;
    Some((funcs, types))
}

/// When multiple entries share the procedure name, prefer the one with a `type_idx`.
pub(super) fn find_debug_fn<'a>(
    funcs: &'a DebugFunctionsSection,
    procedure_name: &str,
) -> Option<&'a DebugFunctionInfo> {
    let kebab = procedure_name.replace('_', "-");
    let hash_name = format!("#{procedure_name}");
    let hash_kebab = format!("#{kebab}");
    let mut first_any: Option<&DebugFunctionInfo> = None;
    for f in &funcs.functions {
        let Some(s) = funcs.strings.get(f.name_idx as usize) else {
            continue;
        };
        let s = s.as_ref();
        let matches = s == procedure_name
            || s == kebab.as_str()
            || s.ends_with(&hash_name)
            || s.ends_with(&hash_kebab);
        if !matches {
            continue;
        }
        if f.type_idx.is_some() {
            return Some(f);
        }
        if first_any.is_none() {
            first_any = Some(f);
        }
    }
    first_any
}

/// Last `/`-separated segment of a WIT path (`account-id` from
/// `miden:base/core-types@1.0.0/account-id`). Empty when `name` is empty or ends in `/`.
pub(super) fn wit_short_name(name: &str) -> &str {
    name.rsplit('/').next().filter(|s| !s.is_empty()).unwrap_or("")
}

pub(super) fn is_account_id_type(name: &str) -> bool {
    wit_short_name(name) == "account-id"
}
