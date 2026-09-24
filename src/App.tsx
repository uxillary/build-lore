import { useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { BuildLoreWorkspace, ScreenshotEvidence, SourceType } from "./types";

const labels: Record<SourceType, string> = { git_repository: "Git repository", screenshots: "Screenshots", conversation: "Conversations", notes: "Notes" };
const sourceTypes: SourceType[] = ["git_repository", "screenshots", "conversation", "notes"];
const isDesktop = "__TAURI_INTERNALS__" in window;

export default function App() {
  const [workspace, setWorkspace] = useState<BuildLoreWorkspace | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [ready, setReady] = useState(false);
  const [view, setView] = useState<"workspace" | "screenshots">("workspace");
  const [evidence, setEvidence] = useState<ScreenshotEvidence[]>([]);
  const [selectedPath, setSelectedPath] = useState("");
  const [preview, setPreview] = useState("");
  const [progress, setProgress] = useState<{ done: number; total: number; current: string } | null>(null);
  const [analysisStates, setAnalysisStates] = useState<Record<string, "queued" | "analysing" | "failed">>({});
  const cancelQueue = useRef(false);

  useEffect(() => {
    if (!isDesktop) { setReady(true); return; }
    invoke<BuildLoreWorkspace | null>("load_recent").then(async (w) => { setWorkspace(w); if (w) setEvidence(await invoke("load_screenshot_evidence", { workspaceId: w.id })); }).catch((e) => setError(String(e))).finally(() => setReady(true));
  }, []);

  useEffect(() => { if (!workspace) return; invoke<ScreenshotEvidence[]>("load_screenshot_evidence", { workspaceId: workspace.id }).then(setEvidence).catch((e) => setError(String(e))); }, [workspace?.id]);
  useEffect(() => { if (!selectedPath) { setPreview(""); return; } invoke<string>("screenshot_preview", { path: selectedPath }).then(setPreview).catch(() => setPreview("")); }, [selectedPath]);

  const latestScan = useMemo(() => workspace?.sources.reduce<string | null>((latest, source) => source.scanned_at && (!latest || source.scanned_at > latest) ? source.scanned_at : latest, null) ?? null, [workspace]);
  async function pickProject() {
    const selected = await open({ directory: true, multiple: false, title: "Choose a project folder" });
    if (typeof selected !== "string") return;
    await action(async () => setWorkspace(await invoke("create_workspace", { projectPath: selected })));
  }
  async function action(work: () => Promise<void>) {
    setBusy(true); setError("");
    try { await work(); } catch (e) { setError(String(e)); } finally { setBusy(false); }
  }
  async function addSource(kind: SourceType) {
    if (!workspace) return;
    const selected = await open({ directory: kind === "screenshots", multiple: kind === "screenshots", title: `Add ${labels[kind].toLowerCase()}`,
      ...(kind === "screenshots" ? {} : { filters: [{ name: "Text files", extensions: ["md", "txt", "json"] }] }) });
    const paths = Array.isArray(selected) ? selected : selected ? [selected] : [];
    if (!paths.length) return;
    await action(async () => {
      let next = workspace;
      for (const path of paths) next = await invoke<BuildLoreWorkspace>("add_source", { workspace: next, sourceType: kind, path });
      setWorkspace(next);
    });
  }
  async function scan() {
    if (!workspace) return;
    await action(async () => setWorkspace(await invoke("scan_sources", { workspace })));
  }

  const screenshots = workspace?.sources.filter((s) => s.source_type === "screenshots").flatMap((s) => s.files.map((f) => ({ ...f, sourceId: s.id }))) ?? [];
  const selected = screenshots.find((f) => f.path === selectedPath);
  const evidenceFor = (file: typeof screenshots[number]) => evidence.find((e) => e.source_id === file.sourceId && e.file_path === file.path);
  const currentEvidence = selected ? evidenceFor(selected) : undefined;
  const stale = (file: typeof screenshots[number], item?: ScreenshotEvidence) => Boolean(item && (item.file_size !== file.size_bytes || item.modified_at !== file.modified_at));
  async function analyse(files: typeof screenshots) {
    if (!workspace || files.length === 0) return;
    const names = files.slice(0, 4).map((f) => f.name).join(", ");
    if (!window.confirm(`The selected screenshot${files.length === 1 ? "" : "s"} will be sent to OpenAI (${files.length > 4 ? `${names}, and ${files.length - 4} more` : names}) for visual analysis. Continue?`)) return;
    cancelQueue.current = false; setAnalysisStates((prev) => ({ ...prev, ...Object.fromEntries(files.map((f) => [f.path, "queued"])) })); setProgress({ done: 0, total: files.length, current: "" });
    for (let i = 0; i < files.length; i++) {
      if (cancelQueue.current) { setAnalysisStates((prev) => Object.fromEntries(Object.entries(prev).filter(([path]) => !files.slice(i).some((f) => f.path === path)))); break; }
      const file = files[i]; setProgress({ done: i, total: files.length, current: file.name });
      setAnalysisStates((prev) => ({ ...prev, [file.path]: "analysing" }));
      try {
        const item = await invoke<ScreenshotEvidence>("analyse_screenshot", { workspaceId: workspace.id, sourceId: file.sourceId, path: file.path });
        setEvidence((prev) => [...prev.filter((e) => !(e.source_id === item.source_id && e.file_path === item.file_path)), item]);
        setAnalysisStates((prev) => { const next = { ...prev }; delete next[file.path]; return next; });
      } catch (e) { setAnalysisStates((prev) => ({ ...prev, [file.path]: "failed" })); setError(`${file.name}: ${String(e)}`); }
      setProgress({ done: i + 1, total: files.length, current: "" });
    }
    setProgress(null);
  }

  if (!ready) return <main className="shell"><p className="muted">Opening your workspace…</p></main>;

  return <main className="shell">
    <header className="topbar"><a className="wordmark" href="#" aria-label="BuildLore home"><span className="mark">b</span> buildlore</a><span className="local"><i /> LOCAL WORKSPACE</span></header>
    {!workspace ? <section className="welcome">
      <p className="eyebrow">PROJECT ARCHAEOLOGY, AT YOUR PACE</p>
      <h1>Every project leaves<br /><em>a story behind.</em></h1>
      <p className="intro">Bring your project folder and the evidence around it together. BuildLore discovers what’s there and keeps it on this device.</p>
      <button className="primary" onClick={() => isDesktop ? void pickProject() : setError("Run BuildLore as a Tauri desktop app to choose a local folder.")}>Choose a project folder <span>↗</span></button>
      <p className="subtle">Your project stays untouched. Nothing is uploaded.</p>
    </section> : <>
      <section className="project-head">
        <div><p className="eyebrow">YOUR WORKSPACE</p><h1>{workspace.name}</h1><p className="path">{workspace.project_path}</p></div>
        <button className="quiet" disabled={busy} onClick={() => void pickProject()}>Switch project</button>
      </section>
      <nav className="view-nav" aria-label="Workspace views"><button aria-current={view === "workspace" ? "page" : undefined} onClick={() => setView("workspace")}>Workspace</button><button aria-current={view === "screenshots" ? "page" : undefined} onClick={() => setView("screenshots")}>Screenshots <span>{screenshots.length}</span></button></nav>
      {view === "screenshots" ? <section className="library">
        <div className="section-heading"><div><p className="eyebrow">VISUAL EVIDENCE</p><h2>Screenshots</h2></div><button className="quiet" disabled={!!progress || !screenshots.some((f) => !evidenceFor(f) || stale(f, evidenceFor(f)))} onClick={() => void analyse(screenshots.filter((f) => !evidenceFor(f) || stale(f, evidenceFor(f))))}>Analyse unanalysed</button></div>
        <p className="privacy-note">Discovery is local; nothing is uploaded. AI analysis sends only the selected screenshot to OpenAI after confirmation.</p>
        {progress && <div className="queue-status" role="status"><span>Analysing screenshots · {progress.done} / {progress.total} complete</span><span>Current: {progress.current || "Saving result"}</span><button className="text-action" onClick={() => { cancelQueue.current = true; }}>Cancel queued work</button></div>}
        {screenshots.length === 0 ? <p className="inventory-empty">Add a screenshot folder and scan sources to populate this library.</p> : <div className="library-layout"><ul className="screenshot-list" aria-label="Discovered screenshots">{screenshots.map((file) => { const item = evidenceFor(file); const isStale = stale(file, item); const state = analysisStates[file.path]; return <li key={file.path}><button className={`screenshot-item ${selectedPath === file.path ? "selected" : ""}`} onClick={() => setSelectedPath(file.path)}><span className="thumb">{file.path === selectedPath && preview ? <img src={preview} alt=""/> : "▧"}</span><span className="shot-meta"><strong>{file.name}</strong><small>{file.modified_at ? new Date(file.modified_at).toLocaleDateString() : "Date unavailable"} · {state ?? (isStale ? "Changed · reanalyse" : item ? item.content_type.replace(/_/g, " ") : "Not analysed")}</small><small>{item && !isStale ? item.description : ""}</small></span></button></li>; })}</ul>
        <article className="evidence-detail">{selected ? <><div className="detail-head"><div><p className="eyebrow">{currentEvidence && !stale(selected, currentEvidence) ? "AI OBSERVATION" : "SOURCE FILE"}</p><h3>{selected.name}</h3><p className="path">{selected.modified_at ? new Date(selected.modified_at).toLocaleString() : "File date unavailable"} · {selected.size_bytes ?? "?"} bytes</p></div><button className="primary" disabled={!!progress} onClick={() => void analyse([selected])}>{currentEvidence && !stale(selected, currentEvidence) ? "Reanalyse" : "Analyse"}</button></div>{preview && <img className="large-preview" src={preview} alt={`Preview of ${selected.name}`}/>}<p className="path detail-path">{selected.path}</p>{currentEvidence && !stale(selected, currentEvidence) ? <><p className="type-label">{currentEvidence.content_type.replace(/_/g, " ").toUpperCase()}</p><p className="description">{currentEvidence.description}</p>{currentEvidence.project_area && <p><strong>Project area</strong><br/>{currentEvidence.project_area}</p>}<EvidenceList title="Visible text" items={currentEvidence.visible_text}/><EvidenceList title="Notable elements" items={currentEvidence.notable_elements}/><EvidenceList title="Possible purpose" items={currentEvidence.possible_purpose}/><EvidenceList title="Possible relevance" items={currentEvidence.possible_story_relevance}/><p className="confidence">Confidence {(currentEvidence.confidence * 100).toFixed(0)}% · {currentEvidence.model.provider} / {currentEvidence.model.model}</p></> : <p className="inventory-empty">{currentEvidence ? "This file changed after analysis. Reanalyse it to refresh its evidence." : "No AI observations yet."}</p>}</> : <p className="inventory-empty">Select a screenshot to inspect its source facts and evidence.</p>}</article></div>}
      </section> : <>
      <section className="sources">
        <div className="section-heading"><div><p className="eyebrow">DISCOVERY</p><h2>Sources</h2></div><span className="count">{workspace.sources.length} registered</span></div>
        <div className="source-list">{sourceTypes.map((kind) => {
          const group = workspace.sources.filter((s) => s.source_type === kind);
          const count = group.reduce((n, s) => n + s.files.length, 0);
          const present = kind === "git_repository" ? group.length > 0 : group.some((s) => s.exists);
          const detail = kind === "git_repository" ? (present ? "Detected" : "Not detected") : `${count} ${kind === "screenshots" ? "images" : "files"}`;
          return <div className="source-row" key={kind}><span className={`status-dot ${present ? "active" : ""}`} aria-hidden="true"/><div className="source-label"><strong>{labels[kind]}</strong>{group.length > 0 && kind !== "git_repository" && <span>{group.map((s) => s.display_name).join(", ")}</span>}</div><span className={`source-value ${present ? "" : "muted"}`}>{detail}</span>
            {kind !== "git_repository" && <button className="add" onClick={() => void addSource(kind)} aria-label={`Add ${labels[kind].toLowerCase()}`}>+</button>}
          </div>;
        })}</div>
        {workspace.sources.some((s) => s.source_type !== "git_repository") && <div className="inventories">{workspace.sources.filter((s) => s.source_type !== "git_repository").map((source) => <details key={source.id}>
          <summary>{source.display_name} <span>{source.files.length} discovered</span></summary>
          {source.files.length ? <ul>{source.files.slice(0, 250).map((file) => <li key={file.path}><span title={file.path}>{file.path}</span><time>{file.modified_at ? new Date(file.modified_at).toLocaleDateString() : ""}</time></li>)}</ul> : <p className="inventory-empty">No supported files discovered yet.</p>}
          {source.files.length > 250 && <p className="inventory-empty">Showing 250 of {source.files.length} files.</p>}
        </details>)}</div>}
        {workspace.sources.some((s) => !s.exists || s.error) && <div className="issues">{workspace.sources.filter((s) => !s.exists || s.error).map((s) => <p key={s.id}><strong>{s.display_name}:</strong> {s.error ?? "Source path is missing."}</p>)}</div>}
        <div className="source-actions"><button className="text-action" onClick={() => void addSource("screenshots")}>Add screenshots</button><button className="text-action" onClick={() => void addSource("conversation")}>Add conversations</button><button className="text-action" onClick={() => void addSource("notes")}>Add notes</button></div>
      </section>
      <footer className="workspace-footer"><div className="scan-info">{latestScan ? <>Last scanned <time>{new Date(latestScan).toLocaleString()}</time></> : "Ready to discover your project sources"}<p>Local discovery only — nothing is uploaded.</p></div><button className="primary scan" disabled={busy} onClick={() => void scan()}>{busy ? <><span className="spinner"/> Scanning…</> : <>Scan sources <span>→</span></>}</button></footer>
      </>}
    </>}
    {error && <div className="error" role="alert">{error}<button onClick={() => setError("")} aria-label="Dismiss error">×</button></div>}
  </main>;
}

function EvidenceList({ title, items }: { title: string; items: string[] }) { return items.length ? <section className="evidence-list"><h4>{title}</h4><ul>{items.map((item, index) => <li key={`${index}-${item}`}>{item}</li>)}</ul></section> : null; }
