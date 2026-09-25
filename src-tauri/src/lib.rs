use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};
use tauri::Manager;
use base64::Engine;

const SCHEMA_VERSION: u32 = 1;
const MAX_FILES: usize = 10_000;
const MAX_DEPTH: usize = 6;
const SKIP_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "dist",
    "build",
    ".next",
    "coverage",
];
const EVIDENCE_SCHEMA_VERSION: u32 = 1;
const VISION_MODEL: &str = "gpt-5-mini";
const CREDENTIAL_SERVICE: &str = "BuildLore";
const CREDENTIAL_ACCOUNT: &str = "openai-api-key";
const ANALYSIS_PROMPT: &str = "Describe only observable screenshot evidence. Identify screen/tool if observable; project area; concise visible state; useful visible text; notable elements; possible purposes and story relevance as topics only. Do not infer history, chronology, causes, developer intentions, or unseen implementation. Avoid tiny labels. Use unknown when uncertain. Return JSON matching the supplied schema.";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScreenshotEvidence {
    schema_version: u32, id: String, source_id: String, file_path: String,
    file_size: u64, modified_at: Option<String>, analysed_at: String,
    description: String, content_type: String, project_area: Option<String>,
    visible_text: Vec<String>, notable_elements: Vec<String>, possible_purpose: Vec<String>,
    possible_story_relevance: Vec<String>, confidence: f32,
    model: ModelInfo,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ModelInfo { provider: String, model: String }
#[derive(Debug, Serialize, Deserialize)]
struct EvidenceStore { schema_version: u32, records: Vec<ScreenshotEvidence> }
#[derive(Debug, Deserialize)]
struct ProviderEvidence { description: String, content_type: String, project_area: Option<String>, visible_text: Vec<String>, notable_elements: Vec<String>, possible_purpose: Vec<String>, possible_story_relevance: Vec<String>, confidence: f32 }

fn evidence_path(app: &tauri::AppHandle, workspace_id: &str) -> Result<PathBuf, String> {
    Ok(app.path().app_data_dir().map_err(|e| e.to_string())?.join("evidence").join(format!("{workspace_id}.json")))
}
fn stored_credential() -> Result<Option<String>, String> {
    match keyring::Entry::new(CREDENTIAL_SERVICE, CREDENTIAL_ACCOUNT).map_err(|_| "Could not access Windows Credential Manager.".to_string())?.get_password() {
        Ok(value) if !value.trim().is_empty() => Ok(Some(value)),
        Ok(_) => Ok(None),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(_) => Err("Could not access Windows Credential Manager.".into()),
    }
}
fn credential_override(env_key: Option<String>, stored: Option<String>) -> Option<String> {
    env_key.filter(|value| !value.trim().is_empty()).or(stored)
}
fn resolve_credential() -> Result<Option<String>, String> {
    if let Ok(env_key) = std::env::var("BUILDLORE_OPENAI_API_KEY") {
        if !env_key.trim().is_empty() { return Ok(credential_override(Some(env_key), None)); }
    }
    stored_credential().map(|stored| credential_override(None, stored))
}

#[tauri::command]
fn provider_status() -> Result<String, String> {
    Ok(if resolve_credential()?.is_some() { "configured" } else { "not_configured" }.into())
}
#[tauri::command]
fn save_openai_key(api_key: String) -> Result<(), String> {
    let key = api_key.trim();
    if key.is_empty() { return Err("Enter an API key before saving.".into()); }
    keyring::Entry::new(CREDENTIAL_SERVICE, CREDENTIAL_ACCOUNT).map_err(|_| "Could not access Windows Credential Manager.".to_string())?
        .set_password(key).map_err(|_| "Could not save the key in Windows Credential Manager.".to_string())
}
#[tauri::command]
fn remove_openai_key() -> Result<(), String> {
    match keyring::Entry::new(CREDENTIAL_SERVICE, CREDENTIAL_ACCOUNT).map_err(|_| "Could not access Windows Credential Manager.".to_string())?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(_) => Err("Could not remove the key from Windows Credential Manager.".into()),
    }
}

fn provider_error(status: reqwest::StatusCode) -> String {
    match status.as_u16() {
        401 | 403 => "OpenAI rejected this API key. Check it in Settings or replace it.".into(),
        429 => "OpenAI could not process the request because of quota, billing, or rate limits. Check your OpenAI API billing and limits.".into(),
        400 | 404 => "OpenAI could not process this screenshot request. Check model access and try again.".into(),
        _ if status.is_server_error() => "OpenAI is temporarily unavailable. Try again later.".into(),
        _ => "OpenAI could not process the screenshot request. Try again.".into(),
    }
}
fn extract_provider_text(response: &serde_json::Value) -> Result<&str, String> {
    response["output"].as_array().and_then(|a| a.iter().flat_map(|v| v["content"].as_array().into_iter().flatten()).find_map(|c| c["text"].as_str()))
        .ok_or_else(|| "OpenAI returned an unsupported response. Retry this screenshot.".into())
}
fn request_provider(client: &reqwest::blocking::Client, endpoint: &str, key: &str, body: &serde_json::Value) -> Result<serde_json::Value, String> {
    let response = client.post(endpoint).bearer_auth(key).json(body).send()
        .map_err(|_| "Could not reach OpenAI. Check your internet connection and try again.".to_string())?;
    if !response.status().is_success() { return Err(provider_error(response.status())); }
    response.json().map_err(|_| "OpenAI returned an invalid response. Retry this screenshot.".to_string())
}
fn load_evidence_at(path: &Path) -> Result<Vec<ScreenshotEvidence>, String> {
    if !path.exists() { return Ok(vec![]); }
    let raw = fs::read(path).map_err(|e| format!("Cannot read screenshot evidence: {e}"))?;
    let store: EvidenceStore = serde_json::from_slice(&raw).map_err(|e| format!("Screenshot evidence is malformed: {e}"))?;
    if store.schema_version != EVIDENCE_SCHEMA_VERSION { return Err("Screenshot evidence has an unsupported schema version".into()); }
    for record in &store.records { validate_evidence(record)?; }
    Ok(store.records)
}
fn valid_content_type(value: &str) -> bool {
    ["application_ui", "website", "code", "terminal", "development_tool", "design", "error", "documentation", "mixed", "unknown"].contains(&value)
}
fn validate_evidence(e: &ScreenshotEvidence) -> Result<(), String> {
    if e.schema_version != EVIDENCE_SCHEMA_VERSION || e.id.is_empty() || e.source_id.is_empty() || e.file_path.is_empty() || e.description.trim().is_empty() || !valid_content_type(&e.content_type) || !(0.0..=1.0).contains(&e.confidence) || e.model.provider != "OpenAI" || e.model.model.is_empty() {
        return Err("The visual analysis response did not match the screenshot evidence schema. Retry this screenshot.".into());
    }
    for values in [&e.visible_text, &e.notable_elements, &e.possible_purpose, &e.possible_story_relevance] {
        if values.len() > 30 || values.iter().any(|v| v.trim().is_empty() || v.len() > 500) { return Err("The visual analysis response contained invalid list items. Retry this screenshot.".into()); }
    }
    Ok(())
}
#[cfg(test)]
fn evidence_matches_identity(e: &ScreenshotEvidence, size: u64, modified_at: &Option<String>) -> bool {
    e.file_size == size && &e.modified_at == modified_at
}
fn file_identity(path: &Path) -> Result<(u64, Option<String>), String> {
    let m = fs::metadata(path).map_err(|e| format!("Cannot read screenshot metadata: {e}"))?;
    Ok((m.len(), modified(path)))
}

#[tauri::command]
fn load_screenshot_evidence(app: tauri::AppHandle, workspace_id: String) -> Result<Vec<ScreenshotEvidence>, String> {
    load_evidence_at(&evidence_path(&app, &workspace_id)?)
}

#[tauri::command]
fn screenshot_preview(path: String) -> Result<String, String> {
    let p = Path::new(&path);
    let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    let mime = match ext.as_str() { "png" => "image/png", "jpg"|"jpeg" => "image/jpeg", "webp" => "image/webp", _ => return Err("Unsupported screenshot format".into()) };
    let bytes = fs::read(p).map_err(|e| format!("Cannot open screenshot preview: {e}"))?;
    if bytes.len() > 20 * 1024 * 1024 { return Err("Screenshot is too large to preview (20 MB limit).".into()); }
    Ok(format!("data:{mime};base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes)))
}

#[tauri::command]
fn analyse_screenshot(app: tauri::AppHandle, workspace_id: String, source_id: String, path: String) -> Result<ScreenshotEvidence, String> {
    let key = resolve_credential()?.ok_or_else(|| "OpenAI isn't configured yet. Add an API key in Settings to analyse screenshots.".to_string())?;
    let p = Path::new(&path);
    let (file_size, modified_at) = file_identity(p)?;
    if file_size > 20 * 1024 * 1024 { return Err("Screenshot exceeds the 20 MB analysis limit.".into()); }
    let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    let mime = match ext.as_str() { "png" => "image/png", "jpg"|"jpeg" => "image/jpeg", "webp" => "image/webp", _ => return Err("Unsupported screenshot format".into()) };
    let bytes = fs::read(p).map_err(|e| format!("Cannot read screenshot: {e}"))?;
    let data = format!("data:{mime};base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes));
    let schema = serde_json::json!({"type":"object","additionalProperties":false,"properties":{
      "description":{"type":"string"},"content_type":{"type":"string","enum":["application_ui","website","code","terminal","development_tool","design","error","documentation","mixed","unknown"]},
      "project_area":{"type":["string","null"]},"visible_text":{"type":"array","items":{"type":"string"}},"notable_elements":{"type":"array","items":{"type":"string"}},"possible_purpose":{"type":"array","items":{"type":"string"}},"possible_story_relevance":{"type":"array","items":{"type":"string"}},"confidence":{"type":"number"}},
      "required":["description","content_type","project_area","visible_text","notable_elements","possible_purpose","possible_story_relevance","confidence"]});
    let body = serde_json::json!({"model":VISION_MODEL,"input":[{"role":"user","content":[{"type":"input_text","text":ANALYSIS_PROMPT},{"type":"input_image","image_url":data}]}],"text":{"format":{"type":"json_schema","name":"screenshot_evidence","strict":true,"schema":schema}}});
    let client = reqwest::blocking::Client::builder().timeout(std::time::Duration::from_secs(90)).build().map_err(|_| "Could not initialize the OpenAI connection.".to_string())?;
    let response = request_provider(&client, "https://api.openai.com/v1/responses", &key, &body)?;
    let text = extract_provider_text(&response)?;
    let parsed: ProviderEvidence = serde_json::from_str(text).map_err(|_| "OpenAI returned malformed or incomplete evidence. Retry this screenshot.".to_string())?;
    let evidence = ScreenshotEvidence { schema_version: EVIDENCE_SCHEMA_VERSION, id: id(), source_id, file_path: path,
      file_size, modified_at, analysed_at: now(), description: parsed.description, content_type: parsed.content_type, project_area: parsed.project_area,
      visible_text: parsed.visible_text, notable_elements: parsed.notable_elements, possible_purpose: parsed.possible_purpose,
      possible_story_relevance: parsed.possible_story_relevance, confidence: parsed.confidence, model: ModelInfo { provider: "OpenAI".into(), model: VISION_MODEL.into() } };
    validate_evidence(&evidence)?;
    let store_path = evidence_path(&app, &workspace_id)?;
    let mut records = load_evidence_at(&store_path)?;
    records.retain(|r| !(r.source_id == evidence.source_id && r.file_path == evidence.file_path));
    records.push(evidence.clone());
    write_json_atomic(&store_path, &EvidenceStore { schema_version: EVIDENCE_SCHEMA_VERSION, records })?;
    Ok(evidence)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum SourceType {
    GitRepository,
    Screenshots,
    Conversation,
    Notes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileEntry {
    path: String,
    name: String,
    modified_at: Option<String>,
    size_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SourceDefinition {
    id: String,
    source_type: SourceType,
    path: String,
    display_name: String,
    exists: bool,
    scanned_at: Option<String>,
    files: Vec<FileEntry>,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Workspace {
    schema_version: u32,
    id: String,
    name: String,
    project_path: String,
    created_at: String,
    updated_at: String,
    sources: Vec<SourceDefinition>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Store {
    schema_version: u32,
    recent_workspace_id: Option<String>,
}

fn now() -> String {
    Utc::now().to_rfc3339()
}
fn id() -> String {
    format!("{}-{}", Utc::now().timestamp_millis(), std::process::id())
}
fn display_name(path: &Path) -> String {
    path.file_name()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("Project")
        .to_string()
}

fn validate_workspace(w: &Workspace) -> Result<(), String> {
    if w.schema_version != SCHEMA_VERSION {
        return Err(format!(
            "Unsupported workspace schema version {}",
            w.schema_version
        ));
    }
    if w.id.is_empty() || w.name.is_empty() || w.project_path.is_empty() {
        return Err("Workspace is missing required fields".into());
    }
    let mut ids = std::collections::HashSet::new();
    for source in &w.sources {
        if source.id.is_empty() || source.path.is_empty() || !ids.insert(&source.id) {
            return Err("Workspace contains an invalid or duplicate source".into());
        }
    }
    Ok(())
}

fn parse_workspace(bytes: &[u8]) -> Result<Workspace, String> {
    let workspace: Workspace = serde_json::from_slice(bytes)
        .map_err(|e| format!("The recent workspace is malformed: {e}"))?;
    validate_workspace(&workspace)?;
    Ok(workspace)
}

fn canonical_key(path: &Path) -> String {
    let value = fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .to_string();
    #[cfg(windows)]
    {
        value.to_lowercase()
    }
    #[cfg(not(windows))]
    {
        value
    }
}

fn has_source(workspace: &Workspace, source_type: &SourceType, path: &Path) -> bool {
    let key = canonical_key(path);
    workspace
        .sources
        .iter()
        .any(|s| &s.source_type == source_type && canonical_key(Path::new(&s.path)) == key)
}

fn write_json_atomic(path: &Path, value: &impl Serialize) -> Result<(), String> {
    let parent = path.parent().ok_or("Invalid application data path")?;
    fs::create_dir_all(parent)
        .map_err(|e| format!("Cannot create application data directory: {e}"))?;
    let tmp = path.with_extension("json.tmp");
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    fs::write(&tmp, bytes).map_err(|e| format!("Cannot write workspace data: {e}"))?;
    if path.exists() {
        fs::remove_file(path).map_err(|e| format!("Cannot replace workspace data: {e}"))?;
    }
    fs::rename(&tmp, path).map_err(|e| format!("Cannot finish saving workspace: {e}"))
}

fn workspace_path(app: &tauri::AppHandle, id: &str) -> Result<PathBuf, String> {
    let root = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(root.join("workspaces").join(format!("{id}.json")))
}

fn read_store(app: &tauri::AppHandle) -> Result<Store, String> {
    let path = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("store.json");
    if !path.exists() {
        return Ok(Store {
            schema_version: SCHEMA_VERSION,
            recent_workspace_id: None,
        });
    }
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let store: Store = serde_json::from_slice(&bytes)
        .map_err(|e| format!("Stored workspace index is malformed: {e}"))?;
    if store.schema_version != SCHEMA_VERSION {
        return Err("Stored workspace index has an unsupported version".into());
    }
    Ok(store)
}

fn save_workspace(app: &tauri::AppHandle, w: &Workspace) -> Result<(), String> {
    validate_workspace(w)?;
    write_json_atomic(&workspace_path(app, &w.id)?, w)?;
    let store = Store {
        schema_version: SCHEMA_VERSION,
        recent_workspace_id: Some(w.id.clone()),
    };
    write_json_atomic(
        &app.path()
            .app_data_dir()
            .map_err(|e| e.to_string())?
            .join("store.json"),
        &store,
    )
}

#[tauri::command]
fn load_recent(app: tauri::AppHandle) -> Result<Option<Workspace>, String> {
    let store = read_store(&app)?;
    let Some(id) = store.recent_workspace_id else {
        return Ok(None);
    };
    let bytes = fs::read(workspace_path(&app, &id)?)
        .map_err(|e| format!("The recent workspace could not be read: {e}"))?;
    let w = parse_workspace(&bytes)?;
    if w.id != id {
        return Err("Workspace identity does not match its saved record".into());
    }
    Ok(Some(w))
}

#[tauri::command]
fn create_workspace(app: tauri::AppHandle, project_path: String) -> Result<Workspace, String> {
    let root = PathBuf::from(&project_path);
    if !root.is_dir() {
        return Err("Choose an accessible project folder.".into());
    }
    let mut w = Workspace {
        schema_version: SCHEMA_VERSION,
        id: id(),
        name: display_name(&root),
        project_path: root.to_string_lossy().to_string(),
        created_at: now(),
        updated_at: now(),
        sources: Vec::new(),
    };
    let git = root.join(".git");
    if git.exists() {
        w.sources.push(SourceDefinition {
            id: "git-repository".into(),
            source_type: SourceType::GitRepository,
            path: root.to_string_lossy().to_string(),
            display_name: "Git repository".into(),
            exists: true,
            scanned_at: Some(now()),
            files: vec![],
            error: None,
        });
    }
    save_workspace(&app, &w)?;
    Ok(w)
}

#[tauri::command]
fn add_source(
    app: tauri::AppHandle,
    mut workspace: Workspace,
    source_type: SourceType,
    path: String,
) -> Result<Workspace, String> {
    validate_workspace(&workspace)?;
    let p = PathBuf::from(&path);
    let valid = match &source_type {
        SourceType::Screenshots => p.is_dir(),
        SourceType::Conversation | SourceType::Notes => p.is_file(),
        SourceType::GitRepository => false,
    };
    if !valid {
        return Err(
            "The selected source is not an accessible folder or file of the expected type.".into(),
        );
    }
    if has_source(&workspace, &source_type, &p) {
        return Ok(workspace);
    }
    workspace.sources.push(SourceDefinition {
        id: id(),
        source_type,
        path: p.to_string_lossy().to_string(),
        display_name: display_name(&p),
        exists: true,
        scanned_at: None,
        files: vec![],
        error: None,
    });
    workspace.updated_at = now();
    save_workspace(&app, &workspace)?;
    Ok(workspace)
}

fn modified(path: &Path) -> Option<String> {
    let time = fs::metadata(path)
        .ok()?
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_secs();
    DateTime::from_timestamp(time as i64, 0).map(|d| d.to_rfc3339())
}
fn recognised(path: &Path, kind: &SourceType) -> bool {
    let ext = path
        .extension()
        .and_then(|x| x.to_str())
        .unwrap_or("")
        .to_lowercase();
    match kind {
        SourceType::Screenshots => ["png", "jpg", "jpeg", "webp"].contains(&ext.as_str()),
        SourceType::Conversation | SourceType::Notes => {
            ["md", "txt", "json"].contains(&ext.as_str())
        }
        SourceType::GitRepository => false,
    }
}
fn walk(
    path: &Path,
    kind: &SourceType,
    depth: usize,
    results: &mut Vec<FileEntry>,
) -> Result<(), String> {
    if results.len() >= MAX_FILES {
        return Ok(());
    }
    let entries = fs::read_dir(path).map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
    for entry in entries {
        if results.len() >= MAX_FILES {
            break;
        }
        let entry = entry.map_err(|e| e.to_string())?;
        let p = entry.path();
        let ty = entry.file_type().map_err(|e| e.to_string())?;
        if ty.is_dir() {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            if depth < MAX_DEPTH && !SKIP_DIRS.contains(&name.as_str()) {
                walk(&p, kind, depth + 1, results)?;
            }
        } else if ty.is_file() && recognised(&p, kind) {
            let meta = entry.metadata().ok();
            results.push(FileEntry {
                path: p.to_string_lossy().to_string(),
                name: entry.file_name().to_string_lossy().to_string(),
                modified_at: modified(&p),
                size_bytes: meta.map(|m| m.len()),
            });
        }
    }
    Ok(())
}

#[tauri::command]
fn scan_sources(app: tauri::AppHandle, mut workspace: Workspace) -> Result<Workspace, String> {
    validate_workspace(&workspace)?;
    let root = Path::new(&workspace.project_path);
    if !root.is_dir() {
        return Err("The selected project folder is missing or unavailable.".into());
    }
    workspace.name = display_name(root);
    let git_index = workspace
        .sources
        .iter()
        .position(|s| s.source_type == SourceType::GitRepository);
    let detected = root.join(".git").exists();
    if detected && git_index.is_none() {
        workspace.sources.push(SourceDefinition {
            id: "git-repository".into(),
            source_type: SourceType::GitRepository,
            path: workspace.project_path.clone(),
            display_name: "Git repository".into(),
            exists: true,
            scanned_at: None,
            files: vec![],
            error: None,
        });
    } else if !detected {
        if let Some(index) = git_index {
            workspace.sources.remove(index);
        }
    }
    for source in &mut workspace.sources {
        if source.source_type == SourceType::GitRepository {
            source.exists = detected;
            source.scanned_at = Some(now());
            continue;
        }
        source.files.clear();
        source.error = None;
        source.exists = Path::new(&source.path).exists();
        source.scanned_at = Some(now());
        if !source.exists {
            source.error = Some("Source path is missing or unavailable.".into());
            continue;
        }
        let path = PathBuf::from(&source.path);
        let outcome = match source.source_type {
            SourceType::Screenshots => walk(&path, &source.source_type, 0, &mut source.files),
            SourceType::Conversation | SourceType::Notes => {
                if recognised(&path, &source.source_type) {
                    let meta = fs::metadata(&path).ok();
                    source.files.push(FileEntry {
                        path: path.to_string_lossy().to_string(),
                        name: display_name(&path),
                        modified_at: modified(&path),
                        size_bytes: meta.map(|m| m.len()),
                    });
                    Ok(())
                } else {
                    Err("File type is not supported.".into())
                }
            }
            SourceType::GitRepository => Ok(()),
        };
        if let Err(e) = outcome {
            source.error = Some(e);
        }
    }
    workspace.updated_at = now();
    save_workspace(&app, &workspace)?;
    Ok(workspace)
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            load_recent,
            create_workspace,
            add_source,
            scan_sources,
            load_screenshot_evidence,
            screenshot_preview,
            analyse_screenshot,
            provider_status,
            save_openai_key,
            remove_openai_key
        ])
        .run(tauri::generate_context!())
        .expect("error while running BuildLore");
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample_evidence() -> ScreenshotEvidence {
        ScreenshotEvidence { schema_version: 1, id: "e1".into(), source_id: "s1".into(), file_path: "C:/shot.png".into(), file_size: 5, modified_at: Some("2026-01-01T00:00:00+00:00".into()), analysed_at: now(), description: "A settings screen".into(), content_type: "application_ui".into(), project_area: None, visible_text: vec!["Settings".into()], notable_elements: vec![], possible_purpose: vec![], possible_story_relevance: vec![], confidence: 0.9, model: ModelInfo { provider: "OpenAI".into(), model: VISION_MODEL.into() } }
    }
    #[test]
    fn evidence_schema_validation_accepts_valid_and_rejects_invalid() {
        let e = sample_evidence(); assert!(validate_evidence(&e).is_ok());
        let mut invalid = e; invalid.content_type = "event".into(); assert!(validate_evidence(&invalid).is_err());
    }
    #[test]
    fn provider_response_is_normalized_only_when_complete() {
        let response = r#"{"description":"Settings screen","content_type":"application_ui","project_area":null,"visible_text":["Settings"],"notable_elements":[],"possible_purpose":[],"possible_story_relevance":[],"confidence":0.8}"#;
        assert!(serde_json::from_str::<ProviderEvidence>(response).is_ok());
        assert!(serde_json::from_str::<ProviderEvidence>(r#"{"description":"missing required fields"}"#).is_err());
        assert!(extract_provider_text(&serde_json::json!({"output":[]})).is_err());
        assert!(extract_provider_text(&serde_json::json!({"output":[{"content":[{"type":"refusal"}]}]})).is_err());
    }
    #[test]
    fn credentials_resolve_override_then_store_then_absent() {
        assert_eq!(credential_override(None, None), None);
        assert_eq!(credential_override(None, Some("stored-key".into())), Some("stored-key".into()));
        assert_eq!(credential_override(Some("override-key".into()), Some("stored-key".into())), Some("override-key".into()));
        assert_eq!(credential_override(Some("  ".into()), Some("stored-key".into())), Some("stored-key".into()));
    }
    #[test]
    fn api_key_is_not_written_to_workspace_or_evidence_json() {
        let sentinel = "secret-test-key-never-persist";
        let workspace = Workspace { schema_version: SCHEMA_VERSION, id: "x".into(), name: "Project".into(), project_path: "C:/project".into(), created_at: now(), updated_at: now(), sources: vec![] };
        let evidence = EvidenceStore { schema_version: 1, records: vec![sample_evidence()] };
        assert!(!String::from_utf8(serde_json::to_vec(&workspace).unwrap()).unwrap().contains(sentinel));
        assert!(!String::from_utf8(serde_json::to_vec(&evidence).unwrap()).unwrap().contains(sentinel));
        assert!(!provider_error(reqwest::StatusCode::UNAUTHORIZED).contains(sentinel));
    }
    #[test]
    fn provider_http_errors_are_safe_and_actionable() {
        assert!(provider_error(reqwest::StatusCode::UNAUTHORIZED).contains("API key"));
        assert!(provider_error(reqwest::StatusCode::TOO_MANY_REQUESTS).contains("billing"));
        assert!(provider_error(reqwest::StatusCode::INTERNAL_SERVER_ERROR).contains("temporarily unavailable"));
    }
    #[test]
    fn provider_request_uses_a_mock_and_sanitizes_http_bodies() {
        use std::{io::{Read, Write}, net::TcpListener, thread};
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0; 4096];
            let _ = stream.read(&mut request);
            let body = r#"{"error":{"message":"secret-test-key-never-log"}}"#;
            write!(stream, "HTTP/1.1 401 Unauthorized\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body).unwrap();
        });
        let client = reqwest::blocking::Client::new();
        let result = request_provider(&client, &format!("http://{address}/v1/responses"), "secret-test-key-never-log", &serde_json::json!({"model": VISION_MODEL}));
        server.join().unwrap();
        let error = result.unwrap_err();
        assert!(error.contains("API key"));
        assert!(!error.contains("secret-test-key-never-log"));
    }
    #[test]
    fn evidence_identity_preserves_unchanged_and_stales_changed_file() {
        let e = sample_evidence();
        assert!(evidence_matches_identity(&e, 5, &e.modified_at));
        assert!(!evidence_matches_identity(&e, 6, &e.modified_at));
        assert!(!evidence_matches_identity(&e, 5, &Some("later".into())));
    }
    #[test]
    fn evidence_store_round_trips_and_rejects_malformed_data() {
        let path = std::env::temp_dir().join(format!("buildlore-evidence-{}.json", id()));
        let store = EvidenceStore { schema_version: 1, records: vec![sample_evidence()] };
        write_json_atomic(&path, &store).unwrap();
        assert_eq!(load_evidence_at(&path).unwrap().len(), 1);
        fs::write(&path, b"broken").unwrap();
        assert!(load_evidence_at(&path).unwrap_err().contains("malformed"));
        let _ = fs::remove_file(path);
    }
    #[test]
    fn extension_recognition_is_case_insensitive() {
        assert!(recognised(
            Path::new("screen.PNG"),
            &SourceType::Screenshots
        ));
        assert!(recognised(
            Path::new("chat.JSON"),
            &SourceType::Conversation
        ));
        assert!(!recognised(
            Path::new("movie.mp4"),
            &SourceType::Screenshots
        ));
    }
    #[test]
    fn workspace_validation_rejects_schema_and_duplicate_sources() {
        let mut w = Workspace {
            schema_version: 9,
            id: "x".into(),
            name: "x".into(),
            project_path: "x".into(),
            created_at: now(),
            updated_at: now(),
            sources: vec![],
        };
        assert!(validate_workspace(&w).is_err());
        w.schema_version = SCHEMA_VERSION;
        let source = SourceDefinition {
            id: "s".into(),
            source_type: SourceType::Notes,
            path: "p".into(),
            display_name: "p".into(),
            exists: true,
            scanned_at: None,
            files: vec![],
            error: None,
        };
        w.sources = vec![source.clone(), source];
        assert!(validate_workspace(&w).is_err());
    }
    #[test]
    fn serialization_round_trips() {
        let w = Workspace {
            schema_version: SCHEMA_VERSION,
            id: "x".into(),
            name: "Project".into(),
            project_path: "C:/project".into(),
            created_at: now(),
            updated_at: now(),
            sources: vec![],
        };
        let encoded = serde_json::to_vec(&w).unwrap();
        let decoded: Workspace = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(decoded.id, w.id);
        assert_eq!(decoded.schema_version, SCHEMA_VERSION);
    }
    #[test]
    fn malformed_saved_workspace_returns_a_recoverable_error() {
        assert!(parse_workspace(b"{not json")
            .unwrap_err()
            .contains("malformed"));
        let wrong_version = br#"{"schema_version":99,"id":"x","name":"x","project_path":"x","created_at":"x","updated_at":"x","sources":[]}"#;
        assert!(parse_workspace(wrong_version)
            .unwrap_err()
            .contains("Unsupported workspace schema"));
    }
    #[test]
    fn source_registration_deduplicates_by_type_and_path() {
        let path = std::env::temp_dir();
        let source = SourceDefinition {
            id: "s".into(),
            source_type: SourceType::Notes,
            path: path.to_string_lossy().to_string(),
            display_name: "tmp".into(),
            exists: true,
            scanned_at: None,
            files: vec![],
            error: None,
        };
        let w = Workspace {
            schema_version: SCHEMA_VERSION,
            id: "x".into(),
            name: "x".into(),
            project_path: path.to_string_lossy().to_string(),
            created_at: now(),
            updated_at: now(),
            sources: vec![source],
        };
        assert!(has_source(&w, &SourceType::Notes, &path));
        assert!(!has_source(&w, &SourceType::Screenshots, &path));
    }
}
