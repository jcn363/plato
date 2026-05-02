# MOD-PLAN.md

## Purpose
Create a concrete roadmap for maximizing modularity across the Plato codebase while preserving existing functionality and meeting the strict quality rules defined in AGENTS.md.

## Goals
- Reduce crate and module size limits (1000 lines per file, 50 lines per function).
- Enforce single‑source‑of‑truth principles for errors, configuration, constants, and shared utilities.
- Isolate platform‑specific code (Kobo, Android, iOS) behind traits.
- Keep public API surface minimal and well‑documented.
- Ensure builds have zero warnings and zero errors on all targets.
- Maintain fast test feedback (unit < ms, integration < seconds).

## Current Structure Overview
- **crates/core** – core library containing document handling, UI, device abstraction, networking, plugins, metadata, and helpers.
- **crates/plato** – binary for Kobo devices.
- **crates/importer** – CLI for importing documents.
- **crates/fetcher** – fetches articles from the internet.
- **crates/epub_edit** – EPUB editing library.
- **crates/epub_editor** – CLI for editing EPUBs.
- **crates/plato-android / plato-ios** – platform adapters.
- **crates/ai** – AI embedding & provider abstraction.
- **crates/search** – search index implementation.
- **crates/thumbnail** – thumbnail generation service.
- **crates/rar** – RAR archive support.
- **crates/error** – error types (currently only used in tests).

## Modularization Strategy
### 1. Crate Decomposition
| New Crate | Responsibility | Rationale |
|---|---|---|
| **plato-core** (existing) | Core shared library (document models, geometry, UI primitives) | Retain as the central glue. |
| **plato-device** | `Device` trait, Kobo, Android, iOS implementations | Isolates hardware specifics. |
| **plato-document** | Document parsers (PDF, EPUB, RAR, etc.) and related errors | Keeps heavy parsing logic separate. |
| **plato-config** | Configuration structs, validation, defaults | Single source for all config. |
| **plato-error** | Unified `PlatoError` enum and `PlatoResult` alias | Enforces the error handling rule. |
| **plato-utils** | Small helpers, lazy statics, constant definitions, logging macros | Prevents helper bloat in core. |
| **plato-plugin** | Plugin trait, registration, loader | Decouples optional extensions. |
| **plato-network** | HTTP fetchers, OPDS client, AI provider HTTP logic | Groups external I/O. |
| **plato-test-mocks** (dev‑dependency crate) | Mock implementations for `Device`, `Document`, etc. | Enables unit testing without hardware. |
| **plato‑ui** | View trait, event handling, rendering queue, UI components | Separates presentation layer.

Existing crates that already match these concerns (e.g., `ai`, `search`, `thumbnail`, `rar`) remain unchanged but will depend on the new shared crates where appropriate.

### 2. File‑Level Refactoring
- Split any file > 800 lines into sibling modules (e.g., `core/src/device/mod.rs` -> `device/mod.rs`, `device/kobo.rs`, `device/android.rs`).
- Ensure every public function ≤ 50 lines; extract inner logic to private helpers.
- Group related items into sub‑modules (`core/src/geom/*.rs`, `core/src/gesture/*.rs`).
- Move generic helpers (`helpers.rs`) to `plato-utils`.
- Relocate constants (`consts.rs`, `constants.rs`) to `plato-utils` or `plato-config` if they are user‑configurable.

### 3. Trait Extraction & Abstraction
- **Device** – already defined; move to `plato-device` crate. Provide `MockDevice` in `plato-test-mocks`.
- **Document** – create a trait exposing `open`, `pages`, `metadata`. Implementors: PDF, EPUB, RAR. Place in `plato-document`.
- **View** – move to `plato-ui` crate, keep only generic rendering abstractions.
- **Plugin** – move to `plato-plugin` crate.
- **NetworkClient** – abstract HTTP calls for OPDS, AI providers; live in `plato-network`.

### 4. Error Handling Unification
1. Create `plato-error/src/lib.rs` with:
   ```rust
   #[derive(Debug, thiserror::Error)]
   pub enum PlatoError { /* existing variants */ }
   pub type PlatoResult<T> = Result<T, PlatoError>;
   ```
2. Replace all `anyhow::Error` in library code with `PlatoError` via `#[from]` conversions.
3. Keep `anyhow` only in binary crates (`crates/plato`, `importer`, etc.) for top‑level aggregation.
4. Provide `From<anyhow::Error>` for `PlatoError` if necessary.

### 5. Configuration Centralization
- Define a `Config` struct in `plato-config/src/lib.rs` mirroring `Settings-sample.toml`.
- Implement `Deserialize` with validation (`serde` + custom `validate` method).
- Provide a single `load_config(path: &Path) -> PlatoResult<Config>`.
- All crates read config via `plato-config` dependency.

### 6. Dependency Management
- Update root `Cargo.toml` workspace `dependencies` section to list shared crates, using workspace inheritance.
- Move version‑pinned third‑party crates (e.g., `serde`, `log`) to `[workspace.dependencies]`.
- Enforce `RUSTFLAGS="-D warnings"` in CI.

### 7. Testing Structure
- **Unit tests** – keep `#[cfg(test)] mod tests` inside each module or sibling `module_tests.rs`.
- **Integration tests** – create `tests/` hierarchy mirroring public APIs (e.g., `tests/document.rs`).
- Add a test helper crate (`plato-test-mocks`) for reusable mock objects.
- Ensure all tests compile with `--target x86_64-unknown-linux-gnu`.

### 8. Documentation & Architecture
- Add `docs/architecture/modularization.md` describing the new crate layout.
- Each crate’s `lib.rs` starts with a module‑level doc comment explaining responsibilities.
- Update `README.md` to link to the new architecture diagram.

## Roadmap & Priorities
1. **Phase 0 – Baseline Audit** (1 day) – Run scripts to collect file sizes, function lengths, error‑type usage.
2. **Phase 1 – Error & Config Crates** (2 days) – Create `plato-error` and `plato-config`, migrate existing error enums, update imports.
3. **Phase 2 – Device Crate Split** (2 days) – Move `device.rs` and platform implementations, adjust Cargo.toml, run build.
4. **Phase 3 – Core Sub‑Crate Extraction** (5 days) – Create `plato-document`, `plato-ui`, `plato-utils`; relocate code, fix broken imports.
5. **Phase 4 – Refactor Large Files** (3 days) – Apply the 1000‑line file rule, extract helpers, update module trees.
6. **Phase 5 – Tests & CI** (2 days) – Re‑organize tests, add mock crate, ensure all CI builds pass without warnings.
7. **Phase 6 – Documentation** (1 day) – Add architecture diagrams, module docs, update README.

## Implementation Checklist (High‑Level)
- [ ] Create new crates and add to workspace.
- [ ] Migrate `PlatoError` to `plato-error`.
- [ ] Move configuration structs to `plato-config`.
- [ ] Extract `Device` trait and implementations.
- [ ] Split large modules into sub‑modules.
- [ ] Update all `use` paths accordingly.
- [ ] Run `cargo fmt`, `cargo clippy`, full build for each target after every phase.
- [ ] Add unit‑test coverage for newly created modules.
- [ ] Update CI scripts to run on new crate layout.

By following this plan the codebase will become easier to understand, test, and extend while adhering to the strict modularity and quality standards defined in the project policy.
