# BuildLore context

BuildLore is a local-first project archaeology tool that aims to reconstruct evidence-backed development stories. See [`context/buildlore-project-concept.md`](../context/buildlore-project-concept.md) for the full product vision.

The product keeps project data local and does not upload source content. The desktop direction is Windows-first, using Tauri 2, React, TypeScript, Vite, Tailwind CSS, and Rust commands for privileged filesystem access.

M01 establishes project workspaces, local source registration/discovery, and versioned JSON persistence. M02 adds screenshot observations as versioned evidence, kept separate from sources and future project events. Evidence is stored in `app_data_dir/evidence/{workspace_id}.json`; screenshot analysis uses OpenAI Responses API (`gpt-4o-mini`) with structured output. The credential is read only from `BUILDLORE_OPENAI_API_KEY` in the BuildLore process environment and is never saved in workspace data. Discovery stays local; only explicit, confirmed analysis sends the selected screenshot. No project-history conclusions are generated.
