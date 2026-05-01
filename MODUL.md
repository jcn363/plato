# MODUL

## 1. Goal
The project is structured around a small set of high‑level **domains**:

* **Library** – core PDF/EPUB/E‑book parsing and manipulation logic.
* **View** – UI rendering, event handling, and layout logic.
* **Document** – representation of documents (pages, layout, rendering data).
* **Thumbnail** – a low‑cost non‑critical sub‑domain that renders preview images.

The aim of the *surgical extraction* is to isolate each domain into its own crate (or at least a module boundary) so that:

1. **Compilation dependencies are minimal** – building a binary that only needs the UI should not drag in heavyweight PDF parsing.
2. **Test isolation** – unit tests for the view system can run without the whole PDF engine.
3. **Future extensibility** – new back‑ends (e.g. a web‑based viewer) can target a subset of the domain stack without pulling the entire codebase.


## 2. Current State

```
crates/
 ├─ core/              ← all‑in‑one crate containing
 │   ├─ view/          ← Windows UI code (fonts, rendering, events)
 │   ├─ document/      ← PDF, EPUB engine
 │   ├─ thumbnail/     ← test util, high‑level cache
 │   └─ ...
 ├─ plank/             ← binary entry point
 ├─ importer/           ← CLI for converting documents
 └─ ...
```

*
The **core** crate is heavily monolithic: `view`, `document`, and `thumbnail` are all hard‑wired dependencies. 
*
`thumbnail` is a prime candidate for isolation because it only depends on a handful of third‑party crates (`image`, `rayon`, etc.).


## 3. Extraction Plan (Thumbnail)

1. **Create a new crate** `crates/thumbnail`.
2. **Move** the entire `src/thumbnail` folder into the new crate.
3. **Update Cargo.toml**
   * Add the crate to the workspace in `Cargo.toml`.
   * Add all dependencies that were required by the old implementation (e.g. `lazy_static`, `image`, `rayon`).
   * Ensure no direct dependency on `core::document` or `core::view`.
4. **Update `core/src/thumbnail.rs`** to become a thin wrapper/re‑export API that forwards calls to the new crate.
5. **Remove the old directory** to avoid duplication.
6. **Add integration tests** that exercise the new crate directly.


## 4. Step‑by‑Step Process

| Step | Action | Verification |
|------|--------|--------------|
| 1 | Add crate entry to workspace ([`Cargo.toml`]) | `cargo check` runs without errors | 
| 2 | Move code: `mv crates/core/src/thumbnail crates/thumbnail/src` | file existence verified | 
| 3 | Mirror `thumbnail/Cargo.toml` dependencies | `cargo check` & `cargo test` pass | 
| 4 | Create thin re‑export module in `core/src/thumbnail.rs` | build & tests use `crate::thumbnail::` API | 
| 5 | Delete old directory | `git status` shows no stray files | 
| 6 | Run full test suite | `All tests pass` | 


## 5. Expected Outcome

* `core` now depends only on `thumbnail` via a **public generic interface** – no heavy PDF/e‑ink specifics.
* Building a UI‑only binary (`plato-plato`) becomes leaner.
* Test matrix becomes simpler – `thumbnail` tests run in isolation.


## 6. Future Directions

* Repeat the same process for *View* and *Document* domains – the pattern is identical.
* Leverage a trait‑based façade between `core` and the domains so that swapping out implementations (e.g. a GPU‑accelerated renderer) is trivial.
* Automate dependency auditing with `cargo tree` to ensure no accidental transitive pulls.

---

**Note:** This is a living document.  Commit logs will track each step, and any change that refactors a critical domain surface will be annotated with `#domain-modularisation`.
