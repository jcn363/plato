# Dead Code Investigation and Removal Plan for Plato Codebase

## Overview

This plan outlines a systematic approach to identify, review, justify, or remove dead code in the Plato codebase following the guidelines in AGENTS.md. Dead code increases maintenance burden, creates confusion, and can hide potential bugs.

## AGENTS.md Guidelines for Dead Code

According to AGENTS.md under "Dead Code Investigation":
> **Mandatory rule:** Eliminate dead code — Remove unused functions, imports, fields, and modules immediately; never leave dead code for later.

The investigation process outlined in AGENTS.md:
1. **Find `#[allow(dead_code)]` attributes** — These indicate reserved future functionality or unused code. Review each.
2. **Check for unused code patterns**:
   - Unused constants with `#[allow(dead_code)]`
   - Unused struct fields (prefix with `_` if intentional)
   - Unused imports
   - Unused methods on public types
3. **Validate before removal**:
   - Search for usages of the code in question
   - Confirm it's not called via reflection or macros
   - Check if it's part of a public API that must be maintained
4. **Remove in order of priority**:
   - Remove obviously unused constants first
   - Then unused private functions and methods
   - Finally, consider if public types should be kept for API compatibility
5. **Run clippy with warnings as errors** to catch new dead code:
   ```bash
   RUSTFLAGS="-D warnings" cargo check --target x86_64-unknown-linux-gnu
   ```

## Phase 1: Discovery - Finding Potential Dead Code

### Step 1: Identify `#[allow(dead_code)]` Attributes
Search for all instances of `#[allow(dead_code)]` in the codebase.

### Step 2: Identify Unused Items via Compiler Warnings
Run `cargo clippy` with appropriate flags to detect:
- Unused imports
- Unused variables
- Unused functions
- Unused struct fields
- Unused constants

### Step 3: Manual Pattern Recognition
Search for common dead code patterns:
- Functions that are never called
- Struct fields that are never read
- Constants that are never used
- Modules that are never imported

## Phase 2: Analysis - Justifying or Flagging for Removal

For each piece of potential dead code discovered:

### Analysis Questions:
1. **Is it actually used?** 
   - Search the entire codebase for references
   - Check for dynamic loading (though rare in embedded Rust)
   - Check for FFI usage
   - Check for macro invocation

2. **Is it intentional placeholder/future work?**
   - Look for accompanying comments like TODO, FIXME
   - Check if it's part of an incomplete feature
   - See if there are related issues or documentation

3. **Is it part of a public API?**
   - Public struct fields
   - Public functions in libraries
   - Trait implementations that might be used by downstream crates

4. **Does it serve documentation or example purposes?**
   - Example code that demonstrates usage
   - Commented-out code showing alternatives

### Decision Matrix:
| Code Type | Keep If | Remove If |
|-----------|---------|-----------|
| Private function/method | Used somewhere | Never called |
| Public function/method | Part of stable API | Truly unused and safe to remove |
| Struct field | Used in initialization/access | Never read/written |
| Constant | Used somewhere | Never referenced |
| Import | Used in file | Completely unused |
| Module | Imported somewhere | Never imported |
| `#[allow(dead_code)]` item | Justified future use | No justification |

## Phase 3: Validation - Ensuring Safe Removal

Before removing any code:
1. **Run tests** - Ensure no test regressions
2. **Check build** - Ensure code still compiles
3. **Run clippy** - Verify no new warnings introduced
4. **Consider runtime detection** - If uncertain, add logging temporarily

## Phase 4: Removal Process

For each item confirmed as dead code:
1. **Remove the code** - Delete unused functions, imports, fields, etc.
2. **Remove related `#[allow(dead_code)]`** - If the item is removed, the allowance is no longer needed
3. **Update documentation** - If removing code that was referenced in docs
4. **Commit with clear message** - Explain what was removed and why

## Specific Focus Areas Based on Initial Scan

From a quick grep, I've already identified several areas with `#[allow(dead_code)]` attributes:

### Font System (`/home/user/Desktop/plato/crates/core/src/font/face.rs`)
Multiple `#[allow(dead_code)]` attributes on methods in the font face implementation.

### Reader UI Components (`/home/user/Desktop/plato/crates/core/src/view/reader/`)
- results_bar.rs: Multiple dead code allowances
- chapter_label.rs: Multiple dead code allowances  
- margin_cropper.rs: Multiple dead code allowances
- tool_bar.rs: Multiple dead code allowances
- results_label.rs: Multiple dead code allowances
- bottom_bar.rs: Dead code allowance

## Implementation Approach

This plan will be executed by:
1. Running systematic discovery scans
2. Analyzing each finding using the decision matrix
3. Validating potential removals
4. Creating removal commits in small, focused batches
5. Running validation after each batch

## Success Criteria

The dead code investigation is complete when:
1. No `#[allow(dead_code)]` attributes remain without clear justification
2. Clippy reports no unused code warnings (with `-W clippy::all`)
3. Manual inspection confirms no obviously unused code remains
4. All builds and tests continue to pass
5. Commit history shows systematic removal of justified dead code

## References

- AGENTS.md: Dead Code Investigation section
- Rust idioms for clean code
- Plato's coding conventions from AGENTS.md
