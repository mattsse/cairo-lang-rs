use crate::compiler::data::{DebugInfo, ProgramHint};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Program {
    pub builtins: Vec<String>,
    pub data: Vec<String>,
    pub debug_info: DebugInfo,
    pub hints: BTreeMap<String, Vec<ProgramHint>>,
    pub identifiers: BTreeMap<String, serde_json::Value>,
    pub main_scope: String,
    pub prime: String,
    pub reference_manager: serde_json::Value,
}
