# Keel

**Keel** (by SIXTWELVE) — *AI that remembers how you work.*  A Rust/Tauri local-first AI desktop product. macOS first; Linux follows through the same Rust core.

## Run and verify

Requirements: Rust 1.85+, Cargo, Node 20+ (development tooling only), and the Tauri 2 CLI.

```sh
npm run dev
npm run check
cargo tauri build --bundles app,dmg
```

Release outputs are written to `src-tauri/target/release/bundle/`. Keel does not copy model weights into the app. On the owner's Mac it automatically discovers `../model-training` (or `KEEL_MODEL_TRAINING_ROOT`) and adds two local engines: Writer with retrieval and Code 7B. Writer and Developer projects route to the correct model automatically. Ollama and OpenAI-compatible engines remain optional; remote keys are stored in macOS Keychain.

### Use the local models

1. Open Keel and finish the short welcome screen.
2. Create a **Writer** project for writing chat, or a **Developer** project for code chat.
3. In Writer, enter a request, choose **Prepare context**, review it, then choose **Send**. Developer model answers are read-only until a reviewed patch passes Keel's write gates.
4. Open **Models** to check both local engines. `SIXTWELVE Writer` and `SIXTWELVE Code 7B` should each report ready.

The first answer can take longer because the local model is loaded into memory. No prompt, source, or model output leaves the Mac when either SIXTWELVE engine is used.

## Architecture

- `src-tauri/src`: Rust domain, persistence, context assembly, Engine adapters, training bridge, and guarded Developer operations.
- `ui`: bundled interface; all privileged and policy-sensitive work remains in Rust.
- `PROJECT.md`: durable product, privacy, and engineering contract.
- `DECISIONS.md`: consequential product and architecture decisions.
- `docs/PRODUCTION_TESTING.md`: tester setup, scenarios, and known release boundaries.
- `Keel - Design Handoff v2.dc.html`: visual and interaction source of truth.

## Design package

## Contents

```
Keel - Design Handoff v2.dc.html         The current handoff — open in any browser
support.js                               Runtime required by the document (keep alongside)
tokens/context.tokens.json               Design tokens, source of truth
tokens/context.tokens.css                Same tokens as CSS custom properties
README.md                                This file
```

Open the `.dc.html` file directly in a browser. Keep `support.js` and the `tokens/`
folder next to it — the document links to the token files.

## What the document covers

| § | Section |
|---|---|
| 00 | What Keel is — thesis, vocabulary, non-negotiables, app map |
| 01 | Naming — Keel (selected), Context, Studiolo |
| 02 | Identity, lockups, app icon sets (macOS / Windows / Linux, 1024→16), space marks |
| 03 | Foundations — light + dark palettes, type scale, status badges, grid, radius, motion |
| 04 | Welcome + 6 setup steps |
| 04b | Choose your Engine — four routes, seven states |
| 05 | Global shell, Home / Continue (populated, empty, dark) |
| 05b | Project Memory & Continue readiness |
| 06 | Writer workspace, Article Plan, dark reading mode |
| 06b | Pre-answer bar & “What Keel used” receipts |
| 07 | Context Inspector — 7 sections, why-included, over budget, no context |
| 08b | Six gates, Developer receipt, not-ready-for-edits |
| 08 | Developer workspace — five approval gates, test failure, readiness, repositories |
| 09 | Library |
| 10b | Learning Inbox — seven row states, candidate review |
| 10 | Training Center — Learning Inbox, promotion sheet, blind comparison |
| 11 | Models, Privacy, Storage & license |
| 11c | Models — Engine / Personal Models / Working Context |
| 11b | Bring your own engine — local runtimes, imported model files, API-key providers, routing, send gate |
| 12 | Command Palette, keyboard map, 8 critical states |
| 13 | Component inventory with states |
| 14 | Voice & vocabulary (say / never say) |
| 15 | Accessibility floor, platform parity, performance contract |
| 14b | Vocabulary — say / never say |
| 16 | Build order (Trust → Continuity → Developer → Learning) |

## Non-negotiables

- The **Context Inspector** ships in phase 1. It is the product's argument.
- Status is **never color alone** — shape and word always present.
- A finished training run is amber **“Needs review”**, never green. Completion is not quality.
- No AI orbs, sparkles, neon gradients, fake terminals, chat bubbles as the primary
  Writer layout, gamification, or fake time estimates.
- Nothing is written to a repository without six explicit gates.
- Library indexing and training are **local only, always** — no provider, no exception.
- Any work that leaves the machine passes an itemised send gate and shows a persistent
  **Remote · <provider>** marker (word + shape, never color alone). Keys live in the system
  keychain and never appear in sessions, exports, or diagnostics.
- Plain language on the surface (Engine, Personal Model, Working Context); quantization,
  parameter counts and hashes live one disclosure deeper.

## Asset production notes

Icons in the document are inline SVG at a 44×44 grid and are production-ready as drawn.
Export the app icon at 1024/512/256/128/64/32/16 (@1x @2x) to `.icns`, `.ico`, and a
hicolor PNG tree. The 32px and 16px variants are separately simplified — use those
drawings, do not downscale the 1024.

Interface iconography is the host platform's system set (SF Symbols / Fluent / Adwaita)
at 16px, 1.5pt. Only three glyphs are custom: the space marks, the context budget bar,
and the readiness disc.

## Open questions for the team

1. One binary with two spaces, or two products?
2. Is Gold status per-project or global to a space?
3. Should blind comparison be mandatory before any promotion?
4. May a remote provider see Gold sources, or only Included ones?
