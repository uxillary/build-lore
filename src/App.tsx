import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { BuildLoreWorkspace, SourceType } from "./types";

const labels: Record<SourceType, string> = { git_repository: "Git repository", screenshots: "Screenshots", conversation: "Conversations", notes: "Notes" };
const sourceTypes: SourceType[] = ["git_repository", "screenshots", "conversation", "notes"];
const isDesktop = "__TAURI_INTERNALS__" in window;

export default function App() {
  const [workspace, setWorkspace] = useState<BuildLoreWorkspace | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [ready, setReady] = useState(false);

  useEffect(() => {
    if (!isDesktop) { setReady(true); return; }
    invoke<BuildLoreWorkspace | null>("load_recent").then(setWorkspace).catch((e) => setError(String(e))).finally(() => setReady(true));
  }, []);

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
    {error && <div className="error" role="alert">{error}<button onClick={() => setError("")} aria-label="Dismiss error">×</button></div>}
  </main>;
}
