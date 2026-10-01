# Keel decisions

## 2026-08-12 — product questions closed for implementation

- Ship one binary with Writer and Developer spaces. Shared infrastructure is useful; source collections, retrieval, Personal Models, and permissions remain space-scoped.
- Gold is project-scoped. An item may be exemplary in one body of work without silently influencing another.
- Blind comparison is mandatory before promotion. Training completion alone never authorizes activation.
- Remote providers receive only items explicitly Included in the current Working Context. Gold affects local ranking but grants no remote permission.
- Engine changes retain incompatible Personal Models visibly, marked incompatible and disabled. User work is not silently hidden or deleted.

## 2026-08-12 — desktop stack

- Use Tauri 2 with Rust owning state and policy. The bundled web presentation has no independent persistence or privileged operations.
- Use schema-versioned JSON persistence initially. It keeps the data inspectable and portable while the schema is young; atomic replacement and backups protect integrity. A storage trait preserves a later SQLite migration path.
- Ollama is the first local runtime. OpenAI-compatible endpoints provide the first remote-provider contract. Other runtimes are adapters, not product rewrites.

## 2026-08-12 — owner model integration

- Keep model weights and the Python/MLX runtime outside the Keel application bundle. Keel discovers the sibling `model-training` lab or `KEEL_MODEL_TRAINING_ROOT` and invokes a fixed CLI contract.
- Writer projects route to the stable Qwen2.5 3B Writer base with approved-corpus retrieval. Developer projects route to the unmodified Qwen2.5-Coder 7B base that passed the approved maintenance suite.
- Rejected adapters remain inactive. Keel does not combine Writer and Code adapters, and repository writes continue to require the existing review gates.
