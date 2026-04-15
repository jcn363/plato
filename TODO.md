# TODO List for Plato Project

## High Priority (Addressing AGENTS.md Mandates)

1. **File Modularization** - Split files exceeding 1,000 lines
   - `view/reader/reader_impl/reader.rs`: 2,653 lines (AGENTS.md target: < 1,000 lines)
   - `document/html/engine.rs`: 2,679 lines
   - `document/html/engine_text.rs`: 1,076 lines
   - `view/home/ui_toggles.rs`: 1,014 lines
   - Reference: MODULARIZATION_PLAN.md (Critical Violations), GOAL.md line 7

2. **Address Remaining #[allow(dead_code)] Scaffolding**
   - Review and justify or remove dead code, particularly in cover editor and other modules
   - Reference: INTEGRATION_PROGRESS.md (Cover editor product decision), INTEGRATION_QUICK_REFERENCE.md

## Medium Priority

3. **Reader Stub Block Completion**
   - Complete extraction of logic from `reader.rs` into `reader_gestures.rs`, `reader_rendering.rs`, etc.
   - Replace stub methods with active call paths to new modules
   - Ensure all functions are under 50 lines
   - Reference: INTEGRATION_QUICK_REFERENCE.md (Open section 1)

4. **Interactive Application of Crop Selection** (Cover Editor)
   - Implement the interactive application of the crop selection in the cover editor
   - Reference: INTEGRATION_PROGRESS.md (Cover editor product decision), CROP_PLAN.md

## Low Priority / Deferred

5. **Lazy Thumbnail Implementation**
   - Deferred due to device constraints
   - Reference: MODULARIZATION_PLAN.md line 373

## Completed (for reference)

- Home view modularization: completed (home/mod.rs now 596 lines)
- Font module migration: completed (font/mod.rs now 802 lines with safe wrappers)
- PDF Tools UI completion: implemented (interactive redaction region definition and PDF merging file selection)
- Unit test segregation: completed (unit tests moved to sibling `_tests.rs` files)
- Type consolidation in reader.rs: completed (ViewPort already imported)

## Notes

- Priorities based on AGENTS.md mandates (file size limits, safe wrappers, no dead code)
- High priority items block compliance with project guidelines
- Refer to specific .md files for detailed implementation plans
