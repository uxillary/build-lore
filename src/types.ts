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
