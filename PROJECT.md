# Keel

Keel is a paid, local-first desktop workspace for inspectable AI memory. The private development build discovers the user's separately stored SIXTWELVE Writer and Code models; public builds guide users through acquiring compatible local models or connecting another engine. Model weights remain outside the application bundle.

## Product contract

- One application with strictly separated Writer and Developer spaces.
- The Context Inspector is the product's trust boundary: users can see and control what the Engine receives before an answer or action.
- Library indexing and Personal Model training are local-only.
- Remote requests require an itemised send gate and receive only explicitly Included Working Context.
- Keel never learns silently. Material enters training only through the Learning Inbox.
- A completed training run is `Needs review`; promotion requires all Quality Checks and a blind comparison.
- Repository changes require six gates: Scope, File list, Diff, Apply choice, Test command, Result summary.
- Status is always communicated with a word and shape as well as color.
- API keys live in the operating-system credential store and never in Keel data, logs, exports, or diagnostics.

## Architecture

- `src-tauri`: Rust/Tauri desktop application, domain logic, persistence, retrieval, engines, training bridge, and guarded developer operations.
- `ui`: bundled presentation layer. It may request operations but owns no security or policy decisions.
- User data is stored under the platform application-data directory. Writes are atomic and schema-versioned.
- Engine and training integrations are ports with explicit capabilities. SIXTWELVE MLX is the preferred local runtime for the owner's build, Ollama is supported, and remote support uses an OpenAI-compatible HTTP protocol.
- macOS is the first release target. Platform abstractions must remain compatible with a later Linux build.

## Engineering floor

- Stable Rust, formatted and warning-free under Clippy.
- Unit tests for domain policy and integration tests for persistence and context assembly.
- No unrestricted filesystem or shell API exposed to the UI.
- Cancellable long-running work, bounded inputs, safe path handling, and actionable errors.
- Accessible keyboard operation, visible focus, reduced-motion support, light/dark parity, 1120x720 minimum window.
