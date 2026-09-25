# BuildLore context

BuildLore is a local-first project archaeology tool that aims to reconstruct evidence-backed development stories. See [`context/buildlore-project-concept.md`](../context/buildlore-project-concept.md) for the full product vision.

The product keeps project data local and does not upload source content. The desktop direction is Windows-first, using Tauri 2, React, TypeScript, Vite, Tailwind CSS, and Rust commands for privileged filesystem access.

M01 establishes project workspaces, local source registration/discovery, and versioned JSON persistence. M02 adds screenshot observations as versioned evidence, kept separate from sources and future project events. Evidence is stored in `app_data_dir/evidence/{workspace_id}.json`; screenshot analysis uses OpenAI Responses API with strict structured output and the centrally configured `gpt-5-mini` model. Current OpenAI documentation lists GPT-5 mini for image input and structured outputs, and describes it as a cost-sensitive model. Its lower input token price than GPT-4.1 mini suits image-heavy screenshot observations without using a flagship model.

The OpenAI API key is stored through the Rust/Tauri boundary using the `keyring` crate's Windows Credential Manager backend. It is never sent to React for reading and is not stored in workspace/evidence JSON, localStorage, `.env`, or repository files. Credential precedence is `BUILDLORE_OPENAI_API_KEY` developer override, then the saved Windows credential, then not configured. Provider status checks credential availability only; it makes no paid request.

Screenshot privacy boundary: source discovery and scanning remain local. Only the screenshot explicitly selected in the UI is sent to OpenAI, after the UI names the screenshot and obtains confirmation; other project files are not included in that request. Saving a key does not authorize or trigger analysis. Screenshot observations remain evidence only and do not infer project events or history. M02.1 delivers application-level provider setup and prepares manual screenshot acceptance; M03 project history is not implemented.
