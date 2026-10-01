use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Space {
    Writer,
    Developer,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceStatus {
    New,
    NeedsReview,
    Approved,
    Gold,
    ReferenceOnly,
    Excluded,
    Quarantined,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub id: Uuid,
    pub project_id: Uuid,
    pub space: Space,
    pub name: String,
    pub path: String,
    pub content: String,
    pub status: SourceStatus,
    pub canonical: bool,
    pub sha256: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    pub id: Uuid,
    pub text: String,
    pub kind: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub version: u32,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub space: Space,
    pub objective: String,
    pub constraints: Vec<String>,
    pub memories: Vec<MemoryItem>,
    pub documents: Vec<Document>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EngineKind {
    SixtwelveMlx,
    Ollama,
    OpenAiCompatible,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Engine {
    pub id: Uuid,
    pub name: String,
    pub kind: EngineKind,
    pub endpoint: String,
    pub model: String,
    pub remote: bool,
    pub enabled: bool,
    pub max_context_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModelStatus {
    Training,
    NeedsReview,
    Candidate,
    Active,
    Rejected,
    Incompatible,
    Interrupted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalModel {
    pub id: Uuid,
    pub name: String,
    pub space: Space,
    pub base_engine_id: Uuid,
    pub adapter_path: String,
    pub status: ModelStatus,
    pub quality_checks_passed: u8,
    pub quality_checks_total: u8,
    pub blind_comparison_passed: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum DeveloperGate {
    Scope,
    FileList,
    Diff,
    ApplyChoice,
    TestCommand,
    ResultSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperProposal {
    pub id: Uuid,
    pub project_id: Uuid,
    pub repository_root: String,
    pub scope: String,
    pub files: Vec<String>,
    pub diff: String,
    pub gate: DeveloperGate,
    pub apply_authorized: bool,
    pub applied: bool,
    pub test_command: Option<Vec<String>>,
    pub test_authorized: bool,
    pub test_output: Option<String>,
    pub result_summary: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrainingBridge {
    pub cli_path: String,
    pub root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextItem {
    pub source_id: Uuid,
    pub name: String,
    pub excerpt: String,
    pub reason: String,
    pub score: f32,
    pub tokens: usize,
    pub canonical: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcludedContextItem {
    pub source_id: Uuid,
    pub name: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingContext {
    pub id: Uuid,
    pub project_id: Uuid,
    pub objective: String,
    pub constraints: Vec<String>,
    pub decisions: Vec<String>,
    pub included: Vec<ContextItem>,
    pub not_included: Vec<ExcludedContextItem>,
    pub open_threads: Vec<String>,
    pub used_tokens: usize,
    pub max_tokens: usize,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub id: Uuid,
    pub project_id: Uuid,
    pub engine: String,
    pub engine_id: Uuid,
    pub personal_model: Option<String>,
    pub context_id: Uuid,
    pub source_names: Vec<String>,
    pub remote: bool,
    pub provider: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResult {
    pub text: String,
    pub receipt: Receipt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppData {
    pub schema_version: u32,
    pub onboarding_complete: bool,
    pub projects: Vec<Project>,
    pub sources: Vec<Source>,
    pub engines: Vec<Engine>,
    pub active_engine_id: Option<Uuid>,
    pub personal_models: Vec<PersonalModel>,
    pub receipts: Vec<Receipt>,
    pub developer_proposals: Vec<DeveloperProposal>,
    pub training_bridge: TrainingBridge,
    pub theme: String,
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            onboarding_complete: false,
            projects: vec![],
            sources: vec![],
            engines: vec![],
            active_engine_id: None,
            personal_models: vec![],
            receipts: vec![],
            developer_proposals: vec![],
            training_bridge: TrainingBridge::default(),
            theme: "system".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSnapshot {
    pub data: AppData,
    pub data_directory: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateProjectInput {
    pub name: String,
    pub space: Space,
    pub objective: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SaveDocumentInput {
    pub project_id: Uuid,
    pub document_id: Option<Uuid>,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AddEngineInput {
    pub name: String,
    pub kind: EngineKind,
    pub endpoint: String,
    pub model: String,
    pub remote: bool,
    pub api_key: Option<String>,
    pub max_context_tokens: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GenerateInput {
    pub project_id: Uuid,
    pub prompt: String,
    pub context: WorkingContext,
    pub remote_confirmed: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateProposalInput {
    pub project_id: Uuid,
    pub repository_root: String,
    pub scope: String,
    pub files: Vec<String>,
    pub diff: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigureTrainingInput {
    pub cli_path: String,
    pub root: String,
}
