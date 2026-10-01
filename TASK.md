Task
Implement a production-testable macOS Keel desktop app from the v2 design handoff.

Phase
Handoff

Scope
- In: Trust, Continuity, Developer, Learning flows; Rust core; macOS packaging; local-model and training integration boundaries; automated verification.
- Out: shipping proprietary model weights, operating a hosted inference service, Linux packaging in this release.

Constraints
- Rust-first Tauri 2 architecture; macOS first and Linux-compatible.
- Follow PROJECT.md privacy, context, learning, and repository-write gates.

Relevant sources
- `Keel - Design Handoff v2.dc.html`: interaction and visual source of truth.
- `tokens/context.tokens.json`: design token source.
- `PROJECT.md`: durable product and engineering contract.
- `DECISIONS.md`: resolved product and architecture decisions.

Verified state
- Design package exists and contains no prior app implementation.
- Rust 1.95, Cargo 1.95, Node 22, and Tauri CLI 2.11 are installed.

Acceptance criteria
- A new macOS user can onboard, connect an Engine, import/index sources, inspect Working Context, and generate with an auditable receipt.
- Projects, documents, memories, source decisions, settings, and receipts persist across launches.
- Remote sends, learning promotion, and developer writes cannot bypass their gates.
- The app builds, tests, lints, packages, and passes a manual production smoke test.

Verification
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo tauri build`.

Status / next action
The local-model integration is implemented: Keel discovers the sibling model-training lab, provisions Writer and Code 7B engines, and routes by project space. Rebuild, run automated checks, and perform a live Writer/Developer generation smoke test. Public release still requires portable model acquisition, Developer ID signing, and notarization.
