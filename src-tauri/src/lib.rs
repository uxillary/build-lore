use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};
use tauri::Manager;

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
            scan_sources
        ])
        .run(tauri::generate_context!())
        .expect("error while running BuildLore");
}

#[cfg(test)]
mod tests {
    use super::*;
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
