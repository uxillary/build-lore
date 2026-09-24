export type SourceType = "git_repository" | "screenshots" | "conversation" | "notes";

export interface FileEntry {
  path: string;
  name: string;
  modified_at: string | null;
  size_bytes: number | null;
}

export interface SourceDefinition {
  id: string;
  source_type: SourceType;
  path: string;
  display_name: string;
  exists: boolean;
  scanned_at: string | null;
  files: FileEntry[];
  error: string | null;
}

export interface BuildLoreWorkspace {
  schema_version: number;
  id: string;
  name: string;
  project_path: string;
  created_at: string;
  updated_at: string;
  sources: SourceDefinition[];
}

export type ScreenshotContentType = "application_ui" | "website" | "code" | "terminal" | "development_tool" | "design" | "error" | "documentation" | "mixed" | "unknown";
export interface ScreenshotEvidence {
  schema_version: number; id: string; source_id: string; file_path: string;
  file_size: number; modified_at: string | null; analysed_at: string;
  description: string; content_type: ScreenshotContentType; project_area: string | null;
  visible_text: string[]; notable_elements: string[]; possible_purpose: string[];
  possible_story_relevance: string[]; confidence: number;
  model: { provider: string; model: string };
}
