# Plato Architecture Overview

This document provides a high-level overview of Plato's architecture, design principles, and key architectural decisions.

## System Architecture

Plato is organized as a layered architecture with clear separation of concerns:

```
┌─────────────────────────────────────────────────────────────┐
│                    Application Layer                         │
│  (View System, UI Components, Event Handling)                 │
├─────────────────────────────────────────────────────────────┤
│                    Business Logic Layer                      │
│  (Library Management, Document Handling, Settings)          │
├─────────────────────────────────────────────────────────────┤
│                    Service Layer                             │
│  (Rendering, Input, Storage, Synchronization)               │
├─────────────────────────────────────────────────────────────┤
│                    Hardware Abstraction Layer                │
│  (Framebuffers, Frontlight, Battery, Input Devices)       │
├─────────────────────────────────────────────────────────────┤
│                    External Libraries                      │
│  (MuPDF, HarfBuzz, SDL2)                                    │
└─────────────────────────────────────────────────────────────┘
```

## Core Design Principles

### 1. Modular Architecture

Modules are organized by domain with minimal public surface area:

- **document/**: Document handling (PDF, EPUB, HTML)
- **library/**: Book library management
- **view/**: UI view system
- **settings/**: Configuration management
- **framebuffer/**: Display output abstraction
- **font/**: Text rendering and font management

Each module follows the Single Responsibility Principle and exposes only necessary public APIs.

### 2. Trait-Based Abstractions

Major components use traits for testability and flexibility:

```rust
// Document trait - implemented by all document types
pub trait Document {
    fn pages(&self) -> usize;
    fn page(&self, index: usize) -> Option<&dyn Page>;
    fn toc(&self) -> Option<&[TocEntry]>;
}

// Framebuffer trait - abstracts display hardware
pub trait Framebuffer {
    fn dims(&self) -> (u32, u32);
    fn update(&mut self, rect: &Rectangle, mode: UpdateMode) -> Result<u32, Error>;
}
```

### 3. Single Source of Truth

All constants and configuration values have one authoritative location:

- Constants defined in `consts.rs` or owning modules
- Settings managed by `ConfigManager` with validation
- No duplication of magic numbers or configuration values

### 4. Error Handling Strategy

- `anyhow` for application-level error handling
- `thiserror` for library-level error types
- Fail-fast validation at API boundaries
- Clear, actionable error messages

### 5. Resource Management

- RAII patterns for resource cleanup
- Explicit Drop implementations for FFI resources
- Careful memory management on resource-constrained devices

## Key Architectural Decisions

### Why Trait-Based Document Abstraction?

**Decision**: Use traits (`Document`, `Page`) rather than concrete types for document handling.

**Rationale**:
- Allows support for multiple formats (PDF, EPUB, HTML, images) with common interface
- Enables testability through mock implementations
- Supports future document types without core changes
- Permits format-specific optimizations while maintaining consistent API

**Trade-offs**:
- Slight runtime overhead from dynamic dispatch
- More complex type signatures
- Requires careful design of trait boundaries

### Why Custom HTML Engine?

**Decision**: Implement custom HTML/CSS engine rather than using existing libraries.

**Rationale**:
- Optimized for e-ink display characteristics (no animations, limited colors)
- Reduced binary size compared to full browser engines
- Better control over rendering pipeline
- Can optimize for low-power, low-refresh displays

**Trade-offs**:
- Limited CSS support compared to browsers
- Maintenance burden of custom engine
- Need to implement features that browsers provide natively

### Why MuPDF for PDF Rendering?

**Decision**: Use MuPDF via FFI rather than pure Rust PDF libraries.

**Rationale**:
- MuPDF is mature, fast, and has excellent rendering quality
- Supports complex PDF features (forms, annotations, JavaScript)
- Small footprint suitable for embedded devices
- Active development and security updates

**Trade-offs**:
- FFI complexity and safety concerns
- Dependency on external C library
- Build system complexity

### Why View Tree Architecture?

**Decision**: Organize UI as a tree of views with event bubbling.

**Rationale**:
- Matches UI structure naturally (windows contain buttons, etc.)
- Efficient event routing (events traverse tree to leaves)
- Supports z-ordering through tree structure
- Enables composable UI components

**Trade-offs**:
- Event flow can be complex to trace
- Requires careful lifecycle management
- Deep trees may impact performance

## Module Dependencies

```
view/ → geom, framebuffer, font, input, settings
library/ → metadata, settings, validation
document/ → framebuffer, metadata, settings
settings/ → validation
framebuffer/ → geom, device
font/ → (minimal dependencies)
```

Dependencies flow downward; lower layers don't depend on higher layers.

## Performance Considerations

### Memory Management
- Pre-allocated buffers where possible
- Careful pixmap caching
- Box large structures to avoid stack overflow

### Rendering Pipeline
- Dirty rectangle tracking for partial updates
- E-ink specific update modes (partial, full, fast)
- Thumbnail caching for library view

### Document Loading
- Progressive loading for large documents
- Page caching with LRU eviction
- Background preloading of adjacent pages

## Security Considerations

- Input validation at all public API boundaries
- Path validation to prevent directory traversal
- No execution of embedded scripts (PDF JavaScript disabled)
- Sandboxed document parsing where possible
- Validator crate for complex validation scenarios (email formats, string length, numeric ranges)

## Code Quality Standards (AGENTS.md Compliance)

### DRY (Don't Repeat Yourself)
- Common patterns extracted to shared helpers (e.g., `walkdir_visible` in helpers.rs)
- No code duplication across files
- Constants defined in single authoritative location

### Input Validation
- All public APIs validate inputs before processing
- Early failure with clear error messages
- Path existence checks before file operations
- String non-empty validation where required

### Error Handling
- `anyhow` for application-level error handling (binaries, top-level logic)
- `thiserror` for library-level error types (battery module)
- Consistent error context with `.with_context()`
- No `unwrap()` in production code (only in tests)

### Modular Design
- No source file exceeds 1000 lines
- No function exceeds 50 lines
- Clear separation of concerns (parsing, rendering, I/O)
- Traits for testability where appropriate

### Documentation
- Stub implementations documented with rationale (e.g., "Not supported on Kobo e-readers")
- Architecture decisions documented with trade-offs
- Module-level documentation in `mod.rs` files

## Testing Strategy

- Unit tests in sibling `*_tests.rs` files
- Integration tests in `tests/` directory
- Trait-based mocking for isolated testing
- Hardware abstraction enables host-based testing

## Future Architecture Directions

1. **Plugin System**: Extensible document format support
2. **Cloud Sync**: Background synchronization architecture
3. **Theme System**: Comprehensive theme engine
4. **Annotation System**: Enhanced annotation storage and rendering

## References

- Module-level documentation in `*/mod.rs` files
- `ARCHITECTURE.md` files in subdirectories
- AGENTS.md for coding standards and rules
