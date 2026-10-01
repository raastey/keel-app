# Keel production-testing guide

## Build under test

- Product version: 0.1.0
- Target: Apple silicon macOS 13+
- Installer: `src-tauri/target/release/bundle/dmg/Keel_0.1.0_aarch64.dmg`
- User data: `~/Library/Application Support/com.sixtwelve.keel/`
- Credentials: macOS Keychain service `com.sixtwelve.keel`

This build is suitable for controlled user production testing. It is not yet a public signed/notarized release. Test with copies or version-controlled repositories; Developer Apply intentionally performs real writes after authorization.

## Required scenarios

1. Fresh launch shows the Engine / Personal Model / Working Context explanation.
2. Create a Writer project, edit a document, quit, relaunch, and confirm restoration.
3. Import a text or code folder. Verify duplicates are skipped and unsupported/binary/oversized files are ignored.
4. Mark sources Canonical, Reference only, Excluded, and Quarantined. Confirm Excluded and Quarantined sources appear under Not included and never under Relevant material.
5. Configure Ollama, check it, prepare Working Context, inspect all seven sections, generate, and inspect the receipt.
6. Configure a test OpenAI-compatible endpoint. Confirm every remote request shows an itemised send gate and Cancel sends nothing.
7. Add a Developer proposal against a disposable Git repository. Confirm an undeclared or traversing path is rejected, Apply requires review, and tests require a second authorization.
8. Configure the `sixai` bridge, run Doctor, and use a disposable smoke run ID. Confirm completion creates Needs review and cannot become Active until four checks plus blind comparison are recorded.
9. Switch Engines and confirm an active Personal Model tied to the old Engine remains visible as Incompatible.
10. Check keyboard operation, Command Palette (⌘K), Context Inspector (⌘I), light/dark modes, focus rings, and reduced-motion behavior.

## Model-training boundary

The current `sixai` bridge treats `model-training` as the local training laboratory and validates its adapter artifact before registering a Candidate. Its corpus remains governed by that laboratory's own reviewed manifests. Keel does not copy Library sources into that corpus automatically; silent learning is prohibited. A future shared manifest protocol may make that transfer explicit and reviewable.

## Release boundaries

- Supported inference routes: Ollama and OpenAI-compatible chat-completions endpoints.
- Imported source extraction is plain-text/code formats. PDF, DOCX, OCR, embeddings, and safetensors/GGUF execution require dedicated adapters in later milestones.
- Generation is request/response in this build; token streaming and cancellation UI are next hardening work.
- macOS distribution must be Developer ID signed and notarized before public sale.
- Linux credential-store and package integration is intentionally not part of this macOS test build.

## Reporting

Include the operation, expected outcome, visible error, Engine route, and macOS version. Never attach `keel-data.json` without reviewing it: it contains local source text by design. API keys are never stored there.
