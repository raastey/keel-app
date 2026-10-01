# Keel

**Keel** (by SIXTWELVE): *AI that remembers how you work.* A local-first macOS desktop app that shows you exactly what an AI model receives before it answers, and lets you measure how that context changes the answer.

## Why Keel exists

Most AI tools hide the context they send to a model. Keel makes it the product:

- **Context Inspector.** Before any answer, you see the Working Context: objective, constraints, decisions, the sources included, the sources left out, and why.
- **Context Lab.** Choose **Compare** to send the same request three times: with no context, with only your project rules, and with the full Working Context. The answers appear side by side with word counts, timing, and a receipt for each.
- **Receipts.** Every answer records the Engine, the sources used, and whether anything left the Mac.
- **Gates.** Remote providers need an itemised send gate. Code changes need six gates. A trained Personal Model needs quality checks and a blind comparison before it can be activated.

The research behind Keel, including a measured study of prompt engineering, retrieval, and LoRA fine-tuning on the same 3B model, is in [raastey/model-training](https://github.com/raastey/model-training).

## Install (Apple silicon, macOS 13+)

1. Download `Keel_0.1.0_aarch64.dmg` from [Releases](https://github.com/raastey/keel-app/releases) and drag Keel to Applications.
2. Install [Ollama](https://ollama.com), then in Terminal run:
   ```sh
   ollama pull qwen2.5:3b
   ```
3. Open Keel and choose **Connect Ollama** on the welcome screen. Keel uses only models that run on your Mac; cloud models are ignored.
4. Create a **Writer** project, write or import material into the Library, enter a request, choose **Prepare context**, then **Send** or **Compare**.

A remote OpenAI-compatible provider is optional (**Models**). Its key is stored in macOS Keychain, and every remote request passes the send gate.

## Build from source

Requirements: Rust 1.85+, Cargo, Node 20+ (development tooling only), and the Tauri 2 CLI.

```sh
npm run dev
npm run check
cargo tauri build --bundles app,dmg
```

Release outputs are written to `src-tauri/target/release/bundle/`. Keel does not copy model weights into the app.

### Owner build: SIXTWELVE local models

On the owner's Mac, Keel discovers the sibling `../model-training` lab (or `KEEL_MODEL_TRAINING_ROOT`) and adds two MLX engines: Writer (Qwen2.5 3B with retrieval) and Code 7B (Qwen2.5-Coder 7B). Writer and Developer projects route to the correct model automatically. No prompt, source, or output leaves the Mac when either engine is used.

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
