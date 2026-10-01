use crate::{
    context, credentials,
    domain::*,
    error::{KeelError, Result},
    storage::Store,
};
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};
use uuid::Uuid;
use walkdir::WalkDir;

pub struct Service {
    pub data: AppData,
    store: Store,
}

impl Service {
    pub fn open(directory: std::path::PathBuf) -> Result<Self> {
        let (store, data) = Store::open(directory)?;
        let mut service = Self { data, store };
        if service.provision_sixtwelve_engines() {
            service.save()?;
        }
        Ok(service)
    }

    fn provision_sixtwelve_engines(&mut self) -> bool {
        let Some(root) = discover_model_training_root() else {
            return false;
        };
        let root_text = root.display().to_string();
        let profiles = [
            ("SIXTWELVE Writer", "writer", 16_384),
            ("SIXTWELVE Code 7B", "code-7b", 32_768),
        ];
        let mut changed = false;
        for (name, model, max_context_tokens) in profiles {
            if self
                .data
                .engines
                .iter()
                .any(|engine| engine.kind == EngineKind::SixtwelveMlx && engine.model == model)
            {
                continue;
            }
            let engine = Engine {
                id: Uuid::new_v4(),
                name: name.into(),
                kind: EngineKind::SixtwelveMlx,
                endpoint: root_text.clone(),
                model: model.into(),
                remote: false,
                enabled: true,
                max_context_tokens,
            };
            if model == "writer" && self.data.active_engine_id.is_none() {
                self.data.active_engine_id = Some(engine.id);
            }
            self.data.engines.push(engine);
            changed = true;
        }
        changed
    }

    fn save(&self) -> Result<()> {
        self.store.save(&self.data)
    }
    pub fn snapshot(&self) -> AppSnapshot {
        AppSnapshot {
            data: self.data.clone(),
            data_directory: self.store.directory().display().to_string(),
        }
    }

    pub fn finish_onboarding(&mut self) -> Result<()> {
        self.data.onboarding_complete = true;
        self.save()
    }

    pub fn create_project(&mut self, input: CreateProjectInput) -> Result<Project> {
        if input.name.trim().is_empty() {
            return Err(KeelError::Validation("Project name is required".into()));
        }
        let now = Utc::now();
        let document = Document {
            id: Uuid::new_v4(),
            title: "Untitled".into(),
            body: String::new(),
            version: 1,
            updated_at: now,
        };
        let project = Project {
            id: Uuid::new_v4(),
            name: input.name.trim().into(),
            space: input.space,
            objective: input.objective.trim().into(),
            constraints: vec![],
            memories: vec![],
            documents: vec![document],
            created_at: now,
            updated_at: now,
        };
        self.data.projects.push(project.clone());
        self.save()?;
        Ok(project)
    }

    pub fn save_document(&mut self, input: SaveDocumentInput) -> Result<Document> {
        let project = self
            .data
            .projects
            .iter_mut()
            .find(|p| p.id == input.project_id)
            .ok_or_else(|| KeelError::NotFound("Project".into()))?;
        let now = Utc::now();
        let document = if let Some(id) = input.document_id {
            let document = project
                .documents
                .iter_mut()
                .find(|d| d.id == id)
                .ok_or_else(|| KeelError::NotFound("Document".into()))?;
            if document.body != input.body {
                document.version += 1;
            }
            document.title = input.title.trim().into();
            document.body = input.body;
            document.updated_at = now;
            document.clone()
        } else {
            let document = Document {
                id: Uuid::new_v4(),
                title: input.title.trim().into(),
                body: input.body,
                version: 1,
                updated_at: now,
            };
            project.documents.push(document.clone());
            document
        };
        project.updated_at = now;
        self.save()?;
        Ok(document)
    }

    pub fn add_memory(
        &mut self,
        project_id: Uuid,
        kind: String,
        text: String,
    ) -> Result<MemoryItem> {
        if !matches!(kind.as_str(), "decision" | "thread") {
            return Err(KeelError::Validation(
                "Memory kind must be decision or thread".into(),
            ));
        }
        let project = self
            .data
            .projects
            .iter_mut()
            .find(|p| p.id == project_id)
            .ok_or_else(|| KeelError::NotFound("Project".into()))?;
        let item = MemoryItem {
            id: Uuid::new_v4(),
            text: text.trim().into(),
            kind,
            created_at: Utc::now(),
        };
        project.memories.push(item.clone());
        project.updated_at = Utc::now();
        self.save()?;
        Ok(item)
    }

    pub fn delete_memory(&mut self, project_id: Uuid, memory_id: Uuid) -> Result<()> {
        let project = self
            .data
            .projects
            .iter_mut()
            .find(|p| p.id == project_id)
            .ok_or_else(|| KeelError::NotFound("Project".into()))?;
        project.memories.retain(|m| m.id != memory_id);
        self.save()
    }

    pub fn import_path(&mut self, project_id: Uuid, raw_path: String) -> Result<Vec<Source>> {
        let project = self
            .data
            .projects
            .iter()
            .find(|p| p.id == project_id)
            .cloned()
            .ok_or_else(|| KeelError::NotFound("Project".into()))?;
        let path = Path::new(&raw_path).canonicalize()?;
        let candidates: Vec<_> = if path.is_dir() {
            WalkDir::new(&path)
                .follow_links(false)
                .max_depth(12)
                .into_iter()
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.file_type().is_file())
                .map(|entry| entry.path().to_path_buf())
                .collect()
        } else {
            vec![path]
        };
        let mut imported = vec![];
        for file in candidates.into_iter().take(10_000) {
            let extension = file
                .extension()
                .and_then(|v| v.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if !matches!(
                extension.as_str(),
                "txt"
                    | "md"
                    | "markdown"
                    | "rs"
                    | "py"
                    | "js"
                    | "ts"
                    | "tsx"
                    | "jsx"
                    | "json"
                    | "toml"
                    | "yaml"
                    | "yml"
                    | "html"
                    | "css"
                    | "swift"
                    | "c"
                    | "h"
                    | "cpp"
                    | "hpp"
            ) {
                continue;
            }
            let bytes = fs::read(&file)?;
            if bytes.len() > 5_000_000 {
                continue;
            }
            let Ok(content) = String::from_utf8(bytes) else {
                continue;
            };
            let digest = format!("{:x}", Sha256::digest(content.as_bytes()));
            if self
                .data
                .sources
                .iter()
                .any(|source| source.project_id == project_id && source.sha256 == digest)
            {
                continue;
            }
            let now = Utc::now();
            let source = Source {
                id: Uuid::new_v4(),
                project_id,
                space: project.space.clone(),
                name: file
                    .file_name()
                    .and_then(|v| v.to_str())
                    .unwrap_or("Source")
                    .into(),
                path: file.display().to_string(),
                content,
                status: SourceStatus::NeedsReview,
                canonical: false,
                sha256: digest,
                created_at: now,
                updated_at: now,
            };
            self.data.sources.push(source.clone());
            imported.push(source);
        }
        self.save()?;
        Ok(imported)
    }

    pub fn update_source(
        &mut self,
        source_id: Uuid,
        status: SourceStatus,
        canonical: bool,
    ) -> Result<Source> {
        let source = self
            .data
            .sources
            .iter_mut()
            .find(|source| source.id == source_id)
            .ok_or_else(|| KeelError::NotFound("Source".into()))?;
        source.status = status;
        source.canonical = canonical;
        source.updated_at = Utc::now();
        let result = source.clone();
        self.save()?;
        Ok(result)
    }

    pub fn assemble_context(&self, project_id: Uuid, prompt: String) -> Result<WorkingContext> {
        let project = self
            .data
            .projects
            .iter()
            .find(|p| p.id == project_id)
            .ok_or_else(|| KeelError::NotFound("Project".into()))?;
        let max = self
            .active_engine()
            .map(|e| e.max_context_tokens)
            .unwrap_or(8192);
        Ok(context::assemble(project, &self.data.sources, &prompt, max))
    }

    pub fn validate_context(&self, project_id: Uuid, context: &WorkingContext) -> Result<()> {
        if context.project_id != project_id {
            return Err(KeelError::Policy(
                "Working Context belongs to a different project".into(),
            ));
        }
        for item in &context.included {
            let source = self
                .data
                .sources
                .iter()
                .find(|source| source.id == item.source_id && source.project_id == project_id)
                .ok_or_else(|| {
                    KeelError::Policy("Working Context contains an unknown source".into())
                })?;
            if matches!(
                source.status,
                SourceStatus::Excluded | SourceStatus::Quarantined
            ) {
                return Err(KeelError::Policy(format!(
                    "{} is not permitted in Working Context",
                    source.name
                )));
            }
            if !source.content.starts_with(&item.excerpt) {
                return Err(KeelError::Policy(format!(
                    "{} has changed since Working Context was assembled",
                    source.name
                )));
            }
        }
        Ok(())
    }

    pub fn add_engine(&mut self, input: AddEngineInput) -> Result<Engine> {
        if input.name.trim().is_empty() || input.model.trim().is_empty() {
            return Err(KeelError::Validation(
                "Engine name and model are required".into(),
            ));
        }
        if input.remote != (input.kind == EngineKind::OpenAiCompatible) {
            return Err(KeelError::Validation(
                "Remote routing must match the Engine kind".into(),
            ));
        }
        let engine = Engine {
            id: Uuid::new_v4(),
            name: input.name.trim().into(),
            kind: input.kind,
            endpoint: input.endpoint.trim_end_matches('/').into(),
            model: input.model.trim().into(),
            remote: input.remote,
            enabled: true,
            max_context_tokens: input
                .max_context_tokens
                .unwrap_or(8192)
                .clamp(1024, 1_000_000),
        };
        if let Some(secret) = input.api_key.filter(|v| !v.is_empty()) {
            credentials::set(&engine.id.to_string(), &secret)?;
        }
        self.data.engines.push(engine.clone());
        self.data.active_engine_id = Some(engine.id);
        self.reconcile_models();
        self.save()?;
        Ok(engine)
    }

    pub fn set_active_engine(&mut self, engine_id: Uuid) -> Result<()> {
        if !self
            .data
            .engines
            .iter()
            .any(|engine| engine.id == engine_id && engine.enabled)
        {
            return Err(KeelError::NotFound("Enabled Engine".into()));
        }
        self.data.active_engine_id = Some(engine_id);
        self.reconcile_models();
        self.save()
    }

    fn reconcile_models(&mut self) {
        if let Some(active) = self.data.active_engine_id {
            for model in &mut self.data.personal_models {
                if model.base_engine_id != active && model.status == ModelStatus::Active {
                    model.status = ModelStatus::Incompatible;
                }
            }
        }
    }

    pub fn active_engine(&self) -> Option<&Engine> {
        self.data.active_engine_id.and_then(|id| {
            self.data
                .engines
                .iter()
                .find(|engine| engine.id == id && engine.enabled)
        })
    }

    pub fn engine_for_project(&self, project_id: Uuid) -> Option<&Engine> {
        let active = self.active_engine()?;
        if active.kind != EngineKind::SixtwelveMlx {
            return Some(active);
        }
        let wanted = match self
            .data
            .projects
            .iter()
            .find(|project| project.id == project_id)?
            .space
        {
            Space::Writer => "writer",
            Space::Developer => "code-7b",
        };
        self.data
            .engines
            .iter()
            .find(|engine| {
                engine.enabled && engine.kind == EngineKind::SixtwelveMlx && engine.model == wanted
            })
            .or(Some(active))
    }

    pub fn record_receipt(&mut self, receipt: Receipt) -> Result<()> {
        self.data.receipts.push(receipt);
        if self.data.receipts.len() > 1000 {
            self.data.receipts.drain(0..100);
        }
        self.save()
    }

    pub fn set_theme(&mut self, theme: String) -> Result<()> {
        if !matches!(theme.as_str(), "system" | "light" | "dark") {
            return Err(KeelError::Validation(
                "Theme must be system, light, or dark".into(),
            ));
        }
        self.data.theme = theme;
        self.save()
    }

    pub fn create_proposal(&mut self, input: CreateProposalInput) -> Result<DeveloperProposal> {
        let project = self
            .data
            .projects
            .iter()
            .find(|project| project.id == input.project_id)
            .ok_or_else(|| KeelError::NotFound("Project".into()))?;
        if project.space != Space::Developer {
            return Err(KeelError::Policy(
                "Repository proposals belong to Developer projects only".into(),
            ));
        }
        let root = Path::new(&input.repository_root).canonicalize()?;
        let proposal = DeveloperProposal {
            id: Uuid::new_v4(),
            project_id: input.project_id,
            repository_root: root.display().to_string(),
            scope: input.scope.trim().into(),
            files: input.files,
            diff: input.diff,
            gate: DeveloperGate::Diff,
            apply_authorized: false,
            applied: false,
            test_command: None,
            test_authorized: false,
            test_output: None,
            result_summary: None,
            created_at: Utc::now(),
        };
        crate::developer::validate(&proposal)?;
        self.data.developer_proposals.push(proposal.clone());
        self.save()?;
        Ok(proposal)
    }

    pub fn authorize_apply(&mut self, id: Uuid) -> Result<DeveloperProposal> {
        let p = self
            .data
            .developer_proposals
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or_else(|| KeelError::NotFound("Developer proposal".into()))?;
        if p.gate != DeveloperGate::Diff {
            return Err(KeelError::Policy(
                "The proposal is not awaiting diff review".into(),
            ));
        }
        p.gate = DeveloperGate::ApplyChoice;
        p.apply_authorized = true;
        let out = p.clone();
        self.save()?;
        Ok(out)
    }
    pub fn record_applied(&mut self, id: Uuid, summary: String) -> Result<DeveloperProposal> {
        let p = self
            .data
            .developer_proposals
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or_else(|| KeelError::NotFound("Developer proposal".into()))?;
        p.applied = true;
        p.apply_authorized = false;
        p.gate = DeveloperGate::TestCommand;
        p.result_summary = Some(summary);
        let out = p.clone();
        self.save()?;
        Ok(out)
    }
    pub fn record_apply_failure(&mut self, id: Uuid) -> Result<()> {
        let proposal = self
            .data
            .developer_proposals
            .iter_mut()
            .find(|proposal| proposal.id == id)
            .ok_or_else(|| KeelError::NotFound("Developer proposal".into()))?;
        proposal.gate = DeveloperGate::Diff;
        proposal.apply_authorized = false;
        self.save()
    }
    pub fn authorize_test(&mut self, id: Uuid, command: Vec<String>) -> Result<DeveloperProposal> {
        let p = self
            .data
            .developer_proposals
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or_else(|| KeelError::NotFound("Developer proposal".into()))?;
        if p.gate != DeveloperGate::TestCommand || !p.applied {
            return Err(KeelError::Policy(
                "Apply the reviewed diff before authorizing tests".into(),
            ));
        }
        p.test_command = Some(command);
        p.test_authorized = true;
        let out = p.clone();
        self.save()?;
        Ok(out)
    }
    pub fn record_test(&mut self, id: Uuid, output: String) -> Result<DeveloperProposal> {
        let p = self
            .data
            .developer_proposals
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or_else(|| KeelError::NotFound("Developer proposal".into()))?;
        p.test_output = Some(output);
        p.test_authorized = false;
        p.gate = DeveloperGate::ResultSummary;
        let out = p.clone();
        self.save()?;
        Ok(out)
    }
    pub fn record_test_failure(&mut self, id: Uuid) -> Result<()> {
        let proposal = self
            .data
            .developer_proposals
            .iter_mut()
            .find(|proposal| proposal.id == id)
            .ok_or_else(|| KeelError::NotFound("Developer proposal".into()))?;
        proposal.test_authorized = false;
        self.save()
    }

    pub fn configure_training(&mut self, input: ConfigureTrainingInput) -> Result<()> {
        let cli = Path::new(&input.cli_path).canonicalize()?;
        let root = Path::new(&input.root).canonicalize()?;
        if !cli.is_file() || !root.is_dir() {
            return Err(KeelError::Validation(
                "Training CLI and root must exist".into(),
            ));
        }
        self.data.training_bridge = TrainingBridge {
            cli_path: cli.display().to_string(),
            root: root.display().to_string(),
        };
        self.save()
    }
    pub fn add_candidate(
        &mut self,
        name: String,
        adapter_path: String,
        engine_id: Uuid,
    ) -> Result<PersonalModel> {
        let model = PersonalModel {
            id: Uuid::new_v4(),
            name,
            space: Space::Writer,
            base_engine_id: engine_id,
            adapter_path,
            status: ModelStatus::NeedsReview,
            quality_checks_passed: 0,
            quality_checks_total: 4,
            blind_comparison_passed: false,
            created_at: Utc::now(),
        };
        self.data.personal_models.push(model.clone());
        self.save()?;
        Ok(model)
    }
    pub fn review_model(
        &mut self,
        id: Uuid,
        passed: u8,
        blind: bool,
        promote: bool,
    ) -> Result<PersonalModel> {
        let model = self
            .data
            .personal_models
            .iter_mut()
            .find(|m| m.id == id)
            .ok_or_else(|| KeelError::NotFound("Personal Model".into()))?;
        if passed > model.quality_checks_total {
            return Err(KeelError::Validation(
                "Passed checks exceed total checks".into(),
            ));
        }
        model.quality_checks_passed = passed;
        model.blind_comparison_passed = blind;
        if promote {
            if passed != model.quality_checks_total || !blind {
                return Err(KeelError::Policy(
                    "All Quality Checks and a blind comparison are required before promotion"
                        .into(),
                ));
            }
            if Some(model.base_engine_id) != self.data.active_engine_id {
                return Err(KeelError::Policy(
                    "This Personal Model is incompatible with the active Engine".into(),
                ));
            }
            model.status = ModelStatus::Active
        } else if passed == model.quality_checks_total && blind {
            model.status = ModelStatus::Candidate
        }
        let out = model.clone();
        self.save()?;
        Ok(out)
    }
}

fn discover_model_training_root() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(path) = std::env::var_os("KEEL_MODEL_TRAINING_ROOT") {
        candidates.push(PathBuf::from(path));
    }
    candidates.push(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()?
            .parent()?
            .join("model-training"),
    );
    if let Some(home) = std::env::var_os("HOME") {
        candidates.push(PathBuf::from(home).join("Downloads/raastey-work/model-training"));
    }
    candidates.into_iter().find(|path| {
        path.join(".venv/bin/sixai").is_file()
            && path
                .join("models/bases/writer/qwen2.5-3b-instruct-4bit/config.json")
                .is_file()
            && path
                .join("models/bases/code/qwen2.5-coder-7b-instruct-4bit/config.json")
                .is_file()
    })
}
