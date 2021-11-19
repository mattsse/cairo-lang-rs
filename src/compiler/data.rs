use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Contract {
    pub abi: Vec<Abi>,
    pub entry_points_by_type: EntryPointsByType,
    pub program: Programm,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Abi {
    pub inputs: Vec<Item>,
    pub name: String,
    pub outputs: Vec<Item>,
    #[serde(rename = "type")]
    pub abi_type: AbiType,
    #[serde(rename = "stateMutability")]
    pub state_mutability: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    #[serde(rename = "type")]
    pub put_type: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntryPointsByType {
    #[serde(rename = "CONSTRUCTOR")]
    pub constructor: Vec<Option<serde_json::Value>>,
    #[serde(rename = "EXTERNAL")]
    pub external: Vec<External>,
    #[serde(rename = "L1_HANDLER")]
    pub l1_handler: Vec<Option<serde_json::Value>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct External {
    pub offset: String,
    pub selector: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Programm {
    pub builtins: Vec<String>,
    pub data: Vec<String>,
    pub debug_info: DebugInfo,
    pub hints: BTreeMap<String, Vec<ProgramHint>>,
    pub identifiers: BTreeMap<String, serde_json::Value>,
    pub main_scope: String,
    pub prime: String,
    pub reference_manager: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DebugInfo {
    pub file_contents: BTreeMap<String, String>,
    pub instruction_locations: HashMap<String, InstructionLocation>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstructionLocation {
    pub accessible_scopes: Vec<String>,
    pub flow_tracking_data: FlowTrackingData,
    pub hints: Vec<InstructionLocationHint>,
    pub inst: Inst,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FlowTrackingData {
    pub ap_tracking: ApTracking,
    pub reference_ids: HashMap<String, i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApTracking {
    pub group: i64,
    pub offset: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstructionLocationHint {
    pub location: Inst,
    pub n_prefix_newlines: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Inst {
    pub end_col: i64,
    pub end_line: i64,
    pub input_file: InputFile,
    pub start_col: i64,
    pub start_line: i64,
    pub parent_location: Option<Vec<ParentLocation>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputFile {
    pub filename: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProgramHint {
    pub accessible_scopes: Vec<String>,
    pub code: String,
    pub flow_tracking_data: FlowTrackingData,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Reference {
    pub ap_tracking_data: ApTracking,
    pub pc: i64,
    pub value: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParentLocation {
    Inst(Inst),
    String(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AbiType {
    #[serde(rename = "function")]
    Function,
}
