use alloc::string::String;

use miden_objects::note::NoteScript;

// NOTE SCRIPT RECORD
// ================================================================================================

/// Represents a `NoteScript` which the Store can keep track and retrieve.
#[derive(Clone, Debug, PartialEq)]
pub struct NoteScriptRecord {
    script_root: String,
    script: NoteScript,
}

impl NoteScriptRecord {
    pub fn new(script_root: String, script: NoteScript) -> NoteScriptRecord {
        NoteScriptRecord { script_root, script }
    }

    pub fn script_root(&self) -> &str {
        &self.script_root
    }

    pub fn script(&self) -> &NoteScript {
        &self.script
    }
}

impl From<NoteScript> for NoteScriptRecord {
    fn from(script: NoteScript) -> Self {
        let script_root = script.root().to_hex();
        NoteScriptRecord { script_root, script }
    }
}

impl From<NoteScriptRecord> for NoteScript {
    fn from(record: NoteScriptRecord) -> Self {
        record.script
    }
}
