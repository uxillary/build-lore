# BuildLore context

BuildLore is a local-first project archaeology tool that aims to reconstruct evidence-backed development stories. See [`context/buildlore-project-concept.md`](../context/buildlore-project-concept.md) for the full product vision.

The product keeps project data local and does not upload source content. The desktop direction is Windows-first, using Tauri 2, React, TypeScript, Vite, Tailwind CSS, and Rust commands for privileged filesystem access.

M01 establishes project workspaces, local source registration/discovery, and versioned JSON persistence. It does not interpret source meaning or use AI. Workspace data lives in the operating system's application-data directory, not in the selected project.
