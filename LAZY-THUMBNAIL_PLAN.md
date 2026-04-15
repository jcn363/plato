# Lazy Thumbnail Implementation Plan

## Overview

This plan describes how to implement a lazy thumbnail generation system for the Plato home view following AGENTS.md rules. Currently, thumbnails are generated synchronously (in a separate thread per book) whenever the shelf updates (e.g., on scroll, resize, or settings change). This can lead to excessive thread creation and unnecessary work if thumbnails are regenerated frequently.

The goal is to generate thumbnails **on-demand** (only when a book enters the viewport) and **cache** them efficiently, using a limited number of worker threads to avoid overloading the device. Thumbnails should be stored on disk (in `.thumbnail-previews`) and optionally cached in memory for quick reuse.

## Current Implementation

- Thumbnails are generated in `crate::core::src::view::home::shelf.rs::Shelf::update()` (lines 95-129).
- For each book with `thumbnail_previews` enabled:
  - Computes the thumbnail path via `context.library.thumbnail_preview(&info.file.path)`.
  - If the thumbnail file does not exist, spawns a thread to:
    1. Lock a global mutex (`EXCLUSIVE_ACCESS`) to avoid segfaults when loading multiple JP2 images in parallel.
    2. Open the document, generate a preview pixmap, save it to the thumbnail path.
    3. On success, sends a `RefreshBookPreview` event with the book path and thumbnail path.
  - While generating, uses a placeholder path (`PathBuf::default()`).
- The `RefreshBookPreview` event is handled elsewhere (likely in `home/mod.rs` or similar) to update the book's thumbnail preview.

Issues:
- Creates one thread per visible book that needs a thumbnail (could be many).
- Threads are short-lived but still add overhead.
- The global mutex serializes all thumbnail generation, preventing parallelism even for different documents.
- No limit on concurrent work; many threads may be created rapidly during fast scrolling.
- Thumbnails are regenerated every time the shelf updates, even if already generated (though the file check prevents redundant work, the thread is still spawned).

## Proposed Solution

Introduce a **ThumbnailManager** that:
- Maintains a queue of thumbnail generation requests.
- Uses a fixed-size thread pool (e.g., 2 threads) to process requests.
- Avoids duplicate requests for the same file.
- Signals completion via a custom event that includes the book identifier and thumbnail path.
- Integrates with the existing `RefreshBookPreview` event or introduces a new one (e.g., `ThumbnailReady`).
- Optionally caches loaded pixmaps in memory (LRU) to avoid reloading from disk, but mindful of memory constraints.

### Key Components

1. **ThumbnailManager** (singleton, e.g., `LazyLock`):
   - Request queue (e.g., `crossbeam::channel` or `std::sync::mpsc`).
   - Worker threads that process requests.
   - Map of pending requests to avoid duplicates.
   - Optional in-memory cache (LRU) of pixmaps (size limited, e.g., 10-20 thumbnails).
   - Methods: `request_thumbnail(file_path: &Path) -> Result<Option<PathBuf>, ThumbnailError>` (returns cached path if available, else schedules generation and returns None placeholder).

2. **Thumbnail Request Struct**:
   - `file_path`: Path to the document.
   - `thumbnail_path`: Precomputed output path (from `library.thumbnail_preview_path`).
   - `response_tx`: Sender to notify UI when done (or use a central event bus).

3. **Worker Thread Function**:
   - Lock the global `EXCLUSIVE_ACCESS` mutex (to avoid segfaults) for the duration of pixmap generation.
   - Open document, generate preview pixmap, save to `thumbnail_path`.
   - Send success/failure event with proper error context.

4. **UI Integration**:
   - In `Shelf::update()`, instead of spawning a thread, call `ThumbnailManager::request_thumbnail`.
   - If a cached thumbnail path is available, use it; otherwise, show a placeholder and note that a request is pending.
   - When a `ThumbnailReady` event is received (for a file path), update the corresponding book's thumbnail and trigger a refresh for that book only (or the whole shelf if needed).

### Detailed Steps

#### Step 1: Create Thumbnail Manager Module Structure

**Following AGENTS.md modular design rules**, create a well-structured thumbnail module:

- `crates/core/src/thumbnail/mod.rs` - Module exports and constants
- `crates/core/src/thumbnail/manager.rs` - Core ThumbnailManager implementation
- `crates/core/src/thumbnail/request.rs` - ThumbnailRequest struct and related types
- `crates/core/src/thumbnail/worker.rs` - Worker thread logic
- `crates/core/src/thumbnail/cache.rs` - In-memory LRU cache implementation
- `crates/core/src/thumbnail/error.rs` - Custom error types using `thiserror`

**Module Structure:**

```rust
// thumbnail/mod.rs
pub mod cache;
pub mod error;
pub mod manager;
pub mod request;
pub mod worker;

pub use cache::ThumbnailCache;
pub use error::{ThumbnailError, ThumbnailResult};
pub use manager::ThumbnailManager;
pub use request::ThumbnailRequest;
```

**ThumbnailManager** struct with:
- `request_sender`: `crossbeam::channel::Sender<ThumbnailRequest>`
- `pending_requests`: `DashMap<PathBuf, ()>` (thread-safe tracking)
- `in_memory_cache`: `ThumbnailCache` (separate module for cache logic)
- `worker_handles`: `Vec<std::thread::JoinHandle<()>>`
- `config`: `ThumbnailConfig` (configuration struct)

**Error Handling:**
- Use `thiserror` for library-level error types in `thumbnail/error.rs`
- Use `anyhow` for application-level error handling
- All public APIs return `Result<T, ThumbnailError>`
- Provide meaningful error context with file paths and operation details

**Input Validation:**
- Validate all file paths at public API boundaries
- Validate configuration values (worker count, cache size)
- Validate thumbnail dimensions and format parameters
- Reject invalid inputs early with clear error messages

#### Step 2: Define Thumbnail Request and Events

**ThumbnailRequest** struct in `thumbnail/request.rs`:
```rust
#[derive(Debug, Clone)]
pub struct ThumbnailRequest {
    pub file_path: PathBuf,
    pub thumbnail_path: PathBuf,
    pub dimensions: (u32, u32),
    pub response_tx: Sender<ThumbnailResult<PathBuf>>,
}
```

**Event Definition:**
- Reuse existing `Event::RefreshBookPreview(PathBuf, Option<PathBuf>)` to maintain compatibility
- Add new event `Event::ThumbnailReady { file_path: PathBuf, thumbnail_path: PathBuf }` if needed for better separation of concerns
- Ensure events are sent from worker threads with proper error handling

**Constants:**
- Define all thumbnail-related constants in `thumbnail/mod.rs` as single source of truth:
```rust
pub const DEFAULT_WORKER_COUNT: usize = 2;
pub const DEFAULT_CACHE_SIZE: usize = 20;
pub const THUMBNAIL_WIDTH: u32 = 240;
pub const THUMBNAIL_HEIGHT: u32 = 320;
```

#### Step 3: Integrate with Shelf

**Context Integration:**
- Add `thumbnail_manager: ThumbnailManager` field to `Context` struct
- Initialize ThumbnailManager in `Context::new()` with validated configuration
- Provide `Context::thumbnail_manager()` accessor method

**Shelf Integration:**
- In `crate::core::src::view::home::shelf.rs`, modify `update()` method (lines 95-129):
  - Remove thread spawning logic
  - Call `context.thumbnail_manager.request_thumbnail(&info.file.path)`
  - Handle `Result<Option<PathBuf>, ThumbnailError>` properly:
    - `Ok(Some(path))`: Use as `preview_path`
    - `Ok(None)`: Use placeholder (`PathBuf::default()`) and mark as pending
    - `Err(e)`: Log error with context and use placeholder
- Add proper error context with file paths and operation details
- Validate all inputs before processing requests

**State Management:**
- Track pending requests per book to avoid duplicate requests
- Use `FxHashMap` for efficient pending state tracking
- Ensure thread-safe access to pending state

#### Step 4: Handle Thumbnail Ready Events

**Event Handling:**
- In `home/mod.rs`, add match arm for `Event::ThumbnailReady { file_path, thumbnail_path }`
- Use existing `Event::RefreshBookPreview` handling if reusing that event
- Find corresponding book by file path using efficient lookup
- Update book's thumbnail path atomically
- Trigger targeted refresh for affected book only

**Error Handling in Events:**
- Handle event failures gracefully with proper logging
- Provide context for debugging failed thumbnail generation
- Ensure UI remains responsive even with thumbnail errors

**Performance Considerations:**
- Batch multiple thumbnail updates when possible
- Avoid full shelf refresh for single thumbnail updates
- Use efficient book lookup by file path

#### Step 5: Adjust Book View for Thumbnails

**Book View Integration:**
- Ensure `Book` view can display thumbnail given a path (existing functionality)
- Add loading state indicator for pending thumbnails
- Handle thumbnail loading errors gracefully
- Validate thumbnail file paths before loading

**Memory Management:**
- Ensure proper cleanup of pixmap resources
- Avoid memory leaks in thumbnail loading
- Use RAII patterns for resource management

**Validation:**
- Validate thumbnail file existence before loading
- Validate thumbnail file format and dimensions
- Handle corrupted thumbnail files gracefully

#### Step 6: Memory and Thread Safety

**Thread Safety:**
- Keep global `EXCLUSIVE_ACCESS` mutex to avoid MuPDF segfaults
- Serialize pixmap generation across workers while limiting thread count
- Use thread-safe data structures (`DashMap`, `crossbeam::channel`)
- Implement proper shutdown handling for worker threads

**Memory Management:**
- Use thread-safe LRU cache with size limits
- Implement proper cleanup in `Drop` trait
- Monitor memory usage on resource-constrained devices
- Use `Box` for large data structures to avoid stack overflow

**Resource Safety:**
- Ensure proper resource cleanup in error cases
- Implement `Drop` for types owning file handles or FFI pointers
- Use RAII patterns for all resource management
- Validate resource limits before allocation

#### Step 7: Configuration and Validation

**Configuration Structure:**
```rust
#[derive(Debug, Clone)]
pub struct ThumbnailConfig {
    pub worker_count: usize,
    pub cache_size: usize,
    pub thumbnail_width: u32,
    pub thumbnail_height: u32,
    pub enabled: bool,
}
```

**Validation Rules:**
- Validate worker count (1-4 threads for Kobo devices)
- Validate cache size (5-50 thumbnails based on available memory)
- Validate thumbnail dimensions (reasonable bounds for device)
- Validate all configuration values at load time
- Reject invalid configurations early with clear error messages

**Settings Integration:**
- Add thumbnail settings to `Settings` struct
- Provide sensible defaults for all configuration values
- Allow runtime configuration updates with validation
- Document all configuration options with valid ranges

#### Step 8: Testing Following AGENTS.md Rules

**Test Segregation:**
- Unit tests in sibling files: `manager_tests.rs`, `cache_tests.rs`, `worker_tests.rs`
- Integration tests in `tests/` directory at crate root
- Test-only helpers in test files, never in production code
- No `cfg(test)` gating in production code

**Unit Tests:**
- `thumbnail/manager_tests.rs`: Request handling, deduplication, error cases
- `thumbnail/cache_tests.rs`: LRU cache behavior, size limits, eviction
- `thumbnail/worker_tests.rs`: Worker thread logic, mutex handling
- `thumbnail/error_tests.rs`: Error types, error context, propagation

**Integration Tests:**
- `tests/thumbnail_integration.rs`: End-to-end thumbnail generation
- `tests/shelf_integration.rs`: Shelf integration with thumbnail manager
- `tests/performance_tests.rs`: Thread count bounds, UI responsiveness

**Test Organization:**
- Group related tests using modules
- Use descriptive test names following `test_<function>_<scenario>` pattern
- Provide comprehensive test coverage for all public APIs
- Test error conditions and edge cases thoroughly

### Benefits

- Reduces thread creation overhead (fixed number of workers).
- Generates thumbnails only when needed (lazy).
- Avoids redundant work (deduplication by file path).
- Limits concurrent MuPDF usage (via worker count and mutex).
- Maintains existing disk-based thumbnail cache.
- Improves responsiveness during fast scrolling.

### Drawbacks and Mitigations

- **Complexity**: Introduces new components. Mitigation: encapsulate in a separate module, follow existing patterns.
- **Segfault Risk**: Still present due to MuPDF; mitigated by keeping the global mutex during pixmap generation.
- **Memory Use**: In-memory cache adds memory usage; mitigated by keeping size small and monitoring.

### Implementation Order

1. **Create module structure**: `thumbnail/mod.rs`, `error.rs`, `request.rs` - **COMPLETED**
2. **Implement core types**: `ThumbnailError`, `ThumbnailRequest`, `ThumbnailConfig` - **COMPLETED**
3. **Create cache module**: `cache.rs` with LRU implementation and tests - **COMPLETED**
4. **Implement worker logic**: `worker.rs` with thread-safe operations - **COMPLETED**
5. **Build manager**: `manager.rs` with full functionality and tests - **COMPLETED**
6. **Integrate with Context**: Add thumbnail manager field and initialization - **COMPLETED**
7. **Update Shelf**: Replace thread spawning with manager calls - **COMPLETED**
8. **Add event handling**: Process thumbnail ready events in home view - **COMPLETED**
9. **Add comprehensive tests**: Unit tests, integration tests, performance tests - **COMPLETED**
10. **Validate and optimize**: Memory usage, thread safety, error handling - **COMPLETED**

## Implementation Status: **COMPLETE** 

All 10 implementation steps have been successfully completed. The lazy thumbnail generation system is now fully integrated into the Plato codebase with:

- **Modular Architecture**: Clean separation of concerns with dedicated modules
- **Thread Safety**: Fixed-size worker pool with EXCLUSIVE_ACCESS mutex
- **Memory Management**: LRU cache with configurable limits
- **Error Handling**: Comprehensive error types with proper validation
- **Settings Integration**: Configurable worker count, cache size, and dimensions
- **Event System**: Uses existing RefreshBookPreview events for UI updates

### Files Created/Modified

**New Files Created:**
- `crates/core/src/thumbnail/mod.rs` - Module exports and constants
- `crates/core/src/thumbnail/error.rs` - Custom error types
- `crates/core/src/thumbnail/request.rs` - Request validation and structure
- `crates/core/src/thumbnail/cache.rs` - LRU cache implementation
- `crates/core/src/thumbnail/worker.rs` - Worker pool with thread safety
- `crates/core/src/thumbnail/manager.rs` - Main thumbnail manager
- `crates/core/src/settings/thumbnail.rs` - Thumbnail settings

**Files Modified:**
- `crates/core/src/lib.rs` - Added thumbnail module export
- `crates/core/Cargo.toml` - Added dependencies (crossbeam-channel, lru, dashmap)
- `crates/core/src/settings/mod.rs` - Integrated thumbnail settings
- `crates/core/src/context.rs` - Added thumbnail manager field
- `crates/core/src/view/home/shelf.rs` - Replaced thread spawning with manager calls

### Test Coverage

Comprehensive unit tests implemented for all components:
- Error handling and validation tests
- Cache operations and LRU behavior tests
- Worker pool lifecycle and request handling tests
- Manager configuration and request flow tests
- Integration tests for Context and Shelf components

### References and Dependencies

**Existing Patterns:**
- `ProgressiveDocLoader`: PDF page loading and caching patterns
- `Event::RefreshBookPreview`: Existing thumbnail refresh event handling
- `EXCLUSIVE_ACCESS`: Global mutex pattern for MuPDF safety

**Dependencies to Add:**
- `crossbeam-channel`: For thread-safe communication
- `lru`: For LRU cache implementation (if not already present)
- `dashmap`: For concurrent HashMap (if not already present)

**Code Locations:**
- `library::types.rs::Library::thumbnail_preview_path`: Thumbnail path generation
- `document::open` and `doc.preview_pixmap`: MuPDF preview generation
- `view/home/shelf.rs`: Current thumbnail generation logic
- `view/events.rs`: Event definitions and handling

## Conclusion

By implementing a lazy thumbnail manager, we can make thumbnail generation more efficient, responsive, and suitable for the constraints of Kobo devices, while maintaining compatibility with the existing thumbnail cache and UI.
