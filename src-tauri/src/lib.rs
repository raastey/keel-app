mod context;
mod credentials;
mod developer;
mod domain;
mod engine;
mod error;
mod service;
mod storage;
mod training;

use crate::{
    domain::*,
    error::{KeelError, Result},
    service::Service,
};
use chrono::Utc;
use std::sync::Mutex;
use tauri::{Manager, State};
use uuid::Uuid;

struct AppState(Mutex<Service>);

fn locked<'a>(state: &'a State<'a, AppState>) -> Result<std::sync::MutexGuard<'a, Service>> {
    state
        .0
        .lock()
        .map_err(|_| KeelError::Storage(std::io::Error::other("Keel state lock was poisoned")))
}

#[tauri::command]
fn snapshot(state: State<'_, AppState>) -> Result<AppSnapshot> {
    Ok(locked(&state)?.snapshot())
}

#[tauri::command]
fn finish_onboarding(state: State<'_, AppState>) -> Result<()> {
    locked(&state)?.finish_onboarding()
}

#[tauri::command]
fn create_project(state: State<'_, AppState>, input: CreateProjectInput) -> Result<Project> {
    locked(&state)?.create_project(input)
}

#[tauri::command]
fn save_document(state: State<'_, AppState>, input: SaveDocumentInput) -> Result<Document> {
    locked(&state)?.save_document(input)
}

#[tauri::command]
fn add_memory(
    state: State<'_, AppState>,
    project_id: Uuid,
    kind: String,
    text: String,
) -> Result<MemoryItem> {
    locked(&state)?.add_memory(project_id, kind, text)
}

#[tauri::command]
fn delete_memory(state: State<'_, AppState>, project_id: Uuid, memory_id: Uuid) -> Result<()> {
    locked(&state)?.delete_memory(project_id, memory_id)
}

#[tauri::command]
fn import_path(state: State<'_, AppState>, project_id: Uuid, path: String) -> Result<Vec<Source>> {
    locked(&state)?.import_path(project_id, path)
}

#[tauri::command]
fn update_source(
    state: State<'_, AppState>,
    source_id: Uuid,
    status: SourceStatus,
    canonical: bool,
) -> Result<Source> {
    locked(&state)?.update_source(source_id, status, canonical)
}

#[tauri::command]
fn assemble_context(
    state: State<'_, AppState>,
    project_id: Uuid,
    prompt: String,
) -> Result<WorkingContext> {
    locked(&state)?.assemble_context(project_id, prompt)
}

#[tauri::command]
fn add_engine(state: State<'_, AppState>, input: AddEngineInput) -> Result<Engine> {
    locked(&state)?.add_engine(input)
}

#[tauri::command]
fn set_active_engine(state: State<'_, AppState>, engine_id: Uuid) -> Result<()> {
    locked(&state)?.set_active_engine(engine_id)
}

#[tauri::command]
async fn check_engine(state: State<'_, AppState>, engine_id: Uuid) -> Result<String> {
    let selected = {
        locked(&state)?
            .data
            .engines
            .iter()
            .find(|engine| engine.id == engine_id)
            .cloned()
            .ok_or_else(|| KeelError::NotFound("Engine".into()))?
    };
    engine::check(&selected).await
}

#[tauri::command]
async fn generate(state: State<'_, AppState>, input: GenerateInput) -> Result<GenerationResult> {
    let selected = {
        let service = locked(&state)?;
        service.validate_context(input.project_id, &input.context)?;
        let engine = service
            .engine_for_project(input.project_id)
            .cloned()
            .ok_or_else(|| {
                KeelError::Policy("Choose an Engine before asking Keel to answer".into())
            })?;
        if engine.remote && !input.remote_confirmed {
            return Err(KeelError::Policy(
                "Review the itemised send gate before using a remote provider".into(),
            ));
        }
        engine
    };
    let text = engine::generate(&selected, &input.context, &input.prompt).await?;
    let receipt = Receipt {
        id: Uuid::new_v4(),
        project_id: input.project_id,
        engine: selected.name.clone(),
        engine_id: selected.id,
        personal_model: None,
        context_id: input.context.id,
        source_names: input
            .context
            .included
            .iter()
            .map(|item| item.name.clone())
            .collect(),
        remote: selected.remote,
        provider: selected.remote.then_some(selected.name),
        created_at: Utc::now(),
    };
    locked(&state)?.record_receipt(receipt.clone())?;
    Ok(GenerationResult { text, receipt })
}

#[tauri::command]
fn set_theme(state: State<'_, AppState>, theme: String) -> Result<()> {
    locked(&state)?.set_theme(theme)
}

#[tauri::command]
fn create_proposal(
    state: State<'_, AppState>,
    input: CreateProposalInput,
) -> Result<DeveloperProposal> {
    locked(&state)?.create_proposal(input)
}

#[tauri::command]
async fn apply_proposal(
    state: State<'_, AppState>,
    proposal_id: Uuid,
) -> Result<DeveloperProposal> {
    let proposal = { locked(&state)?.authorize_apply(proposal_id)? };
    match developer::check_and_apply(&proposal).await {
        Ok(summary) => locked(&state)?.record_applied(proposal_id, summary),
        Err(error) => {
            locked(&state)?.record_apply_failure(proposal_id)?;
            Err(error)
        }
    }
}

#[tauri::command]
async fn run_proposal_test(
    state: State<'_, AppState>,
    proposal_id: Uuid,
    command: Vec<String>,
) -> Result<DeveloperProposal> {
    let proposal = { locked(&state)?.authorize_test(proposal_id, command)? };
    match developer::run_test(&proposal).await {
        Ok(output) => locked(&state)?.record_test(proposal_id, output),
        Err(error) => {
            locked(&state)?.record_test_failure(proposal_id)?;
            Err(error)
        }
    }
}

#[tauri::command]
fn configure_training(state: State<'_, AppState>, input: ConfigureTrainingInput) -> Result<()> {
    locked(&state)?.configure_training(input)
}

#[tauri::command]
async fn training_doctor(state: State<'_, AppState>) -> Result<String> {
    let bridge = { locked(&state)?.data.training_bridge.clone() };
    if bridge.cli_path.is_empty() {
        return Err(KeelError::Validation(
            "Configure the training bridge first".into(),
        ));
    }
    training::doctor(&bridge.cli_path, &bridge.root).await
}

#[tauri::command]
async fn start_training(
    state: State<'_, AppState>,
    profile: String,
    run_id: String,
) -> Result<PersonalModel> {
    let (bridge, engine_id) = {
        let service = locked(&state)?;
        (
            service.data.training_bridge.clone(),
            service.data.active_engine_id.ok_or_else(|| {
                KeelError::Policy("Choose the base Engine before training".into())
            })?,
        )
    };
    if bridge.cli_path.is_empty() {
        return Err(KeelError::Validation(
            "Configure the training bridge first".into(),
        ));
    }
    let adapter = training::train(&bridge.cli_path, &bridge.root, &profile, &run_id).await?;
    locked(&state)?.add_candidate(run_id, adapter, engine_id)
}

#[tauri::command]
fn review_model(
    state: State<'_, AppState>,
    model_id: Uuid,
    passed: u8,
    blind_comparison_passed: bool,
    promote: bool,
) -> Result<PersonalModel> {
    locked(&state)?.review_model(model_id, passed, blind_comparison_passed, promote)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let directory = app.path().app_data_dir()?;
            let service = Service::open(directory)
                .map_err(|error| Box::<dyn std::error::Error>::from(error.to_string()))?;
            app.manage(AppState(Mutex::new(service)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            snapshot,
            finish_onboarding,
            create_project,
            save_document,
            add_memory,
            delete_memory,
            import_path,
            update_source,
            assemble_context,
            add_engine,
            set_active_engine,
            check_engine,
            generate,
            set_theme,
            create_proposal,
            apply_proposal,
            run_proposal_test,
            configure_training,
            training_doctor,
            start_training,
            review_model
        ])
        .run(tauri::generate_context!())
        .expect("Keel failed to start");
}
