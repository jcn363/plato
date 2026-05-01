# Plato Codebase Audit

This document summarizes a static audit of the entire Plato repository.  The audit covers safety, error handling, code duplication, documentation, tests, and build‑target configuration.

## 1. Safety & Unsafe Usage
- **Approximately 95 `unsafe` blocks** across the codebase, mostly in framebuffer, input, JNI, and embedding modules.  All blocks are low‑level OS or hardware interactions, but a few lack explanatory comments (e.g., framebuffer `kobo2` mapping).  
- **Recommendation:** Review each `unsafe` block to confirm safety guarantees and add a brief comment if absent.

## 2. Error Handling
- The core crate defines a global error type `PlatoError` in `crates/error/src/`.  
- Many core modules still use `anyhow::Error` (`core/src/view/...`, `ai/src/...`).  This inconsistency prevents uniform error handling and violates the AGENT.md guideline.  
- **Recommendation:** Migrate core APIs to return `PlatoResult<T>` and convert `anyhow` errors via a helper.

## 3. Code Duplication & Architecture
- Framebuffer implementations for `kobo1` and `kobo2` duplicate large portions of logic; only constants differ.  
- Some thumbnail constants exist in both the `thumbnail` crate and the core crate.  
- **Recommendation:** Extract shared functionality into a common module and de‑duplicate constants.

## 4. Documentation & Comments
- Several public modules lack module‑level docs (`core/src/document/*`, `core/src/eink/*`).  
- Public functions in the `thumbnail` crate have minimal or no docs.  
- **Recommendation:** Add concise `//!` and `///` comments describing purpose, invariants, and safety for unsafe blocks.

## 5. Testing Coverage
- Many modules contain unit tests, but critical areas (device input, gesture handling, TTS, framebuffer) have no tests.  
- The `thumbnail` crate has a comprehensive test suite, but coverage for platform‑specific drivers is missing.  
- **Recommendation:** Add tests for missing modules and cover error paths.

## 6. Build & Target Configuration
- Correct handling of 32‑bit ARM, 64‑bit ARM, and host x86_64 targets is configured.  
- `libs_host/` contains the correct host libraries; no ARM libs are present.  
- CMake and build scripts appear correct, but CI should enforce `cargo clippy -- -D warnings`.

## 7. Concurrency & Thread‑Safety
- Usage of `Arc<Mutex<>>` and `DashMap` is appropriate, but `unsafe impl Sync` for several document types (`PdfDocument`, `HtmlDocument`) relies on external crate guarantees.  
- **Recommendation:** Verify that underlying libraries truly provide thread‑safety and add documentation if necessary.

## 8. Actionable Checklist
| Area | Item | Suggested Approach |
|------|------|--------------------|
| Unsafe blocks | Add safety comment | Search for `unsafe {` and append short comment|
| Errors | Centralize to `PlatoError` | Create `plato-core::errors::to_plato_error` helper and refactor modules |
| Duplication | Merge framebuffer logic | Create `crate::framebuffer::common` module |
| Docs | Add module docs | Write `//!` header for core public modules |
| Tests | Create missing test modules | Add `#[cfg(test)]` blocks for `input`, `gesture`, `tts`, `framebuffer` |
| CI | Run clippy as warning error | Add `cargo clippy -- -D warnings` to CI pipeline |

--- 

**Next steps:**
1. Commit `AUDIT.md` to the repository.
2. Proceed with incremental refactor following the checklist.
3. Validate changes with `cargo test`, `cargo clippy`, and a clean build for all targets.

```
