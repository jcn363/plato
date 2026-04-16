# Performance Optimization Plan for Plato

## Executive Summary

Analysis of the Plato codebase (279 Rust source files) reveals several performance optimization opportunities:

### Key Metrics
- **Total clone() calls**: 263 instances
- **Total to_string()/to_vec()/to_owned()**: 592 instances
- **Total unwrap()/expect()/panic**: 419 instances
- **Total Vec::new()/HashMap::new**: 231 instances (could use with_capacity)
- **Threading**: Uses std::thread extensively, no rayon or tokio

## Priority 1: High Impact Optimizations

### 1.1 Reduce Unnecessary Cloning in Hot Paths

**Issue**: 263 clone() calls, many in performance-critical paths like library scanning and rendering.

**Locations**:
- `crates/core/src/library/scan.rs`: Multiple clones during file scanning
- `crates/core/src/view/home/mod.rs`: Book info cloning during display
- `crates/plato/src/app.rs`: Event handling clones

**Solution**:
- Use references (&T) instead of owned values where possible
- Implement Copy trait for small structs (Fp, simple types)
- Use Cow<str> for string fields that are often borrowed

### 1.2 Optimize Library Search and Filtering

**Issue**: Linear O(n) search across all documents (noted in home/mod.rs line 62)

**Current State**:
```rust
// Linear search through all documents
for (_, info) in &mut self.db {
    // filter logic
}
```

**Solution**:
- Add indexed search using a trie or suffix array for title/author
- Cache filtered results to avoid re-scanning on every keystroke
- Implement incremental filtering

### 1.3 Pre-allocate Collections with Known Capacity

**Issue**: 231 instances of Vec::new()/HashMap::new without capacity hints

**Example Locations**:
- `crates/core/src/library/scan.rs`: File collection loops
- `crates/core/src/sync.rs`: Item merging operations

**Solution**:
```rust
// Instead of:
let mut vec = Vec::new();
for item in items {
    vec.push(transform(item));
}

// Use:
let mut vec = Vec::with_capacity(items.len());
for item in items {
    vec.push(transform(item));
}
```

### 1.4 Optimize String Operations

**Issue**: 592 to_string()/to_vec()/to_owned() calls creating unnecessary allocations

**Hot Spots**:
- Path manipulation in library operations
- Metadata extraction and formatting
- UI text rendering preparation

**Solution**:
- Use string interning for repeated strings (file paths, metadata keys)
- Implement Display trait instead of to_string() chains
- Use format_args! for logging instead of string concatenation

## Priority 2: Medium Impact Optimizations

### 2.1 Async Thumbnail Generation

**Current Issue** (from home/mod.rs line 60):
> Thumbnail generation is synchronous (blocking UI)

**Solution**:
- Move thumbnail generation to worker thread pool
- Use crossbeam-channel for result delivery
- Implement progressive loading with placeholder thumbnails

### 2.2 Optimize Large Library Performance

**Issue** (from home/mod.rs line 61):
> Large libraries (1000+ books) can be slow to scroll

**Solutions**:
- Implement virtual scrolling (only render visible items)
- Lazy load thumbnails on demand
- Cache rendered book cards in memory pool
- Use spatial indexing for quick visible-range queries

### 2.3 Reduce Lock Contention

**Current Pattern**: Multiple thread::spawn with shared state

**Locations**:
- `crates/plato/src/app.rs`: Multiple background threads
- `crates/core/src/gesture.rs`: Touch event processing
- `crates/core/src/input.rs`: Input parsing threads

**Solution**:
- Use lock-free data structures where possible
- Implement actor model with message passing
- Consider using crossbeam or rayon for parallel work

### 2.4 Optimize File I/O Operations

**Issue**: Synchronous file reads blocking main thread

**Locations**:
- `crates/core/src/helpers.rs:135`: fs::read_to_string
- `crates/core/src/document/epub/opener.rs`: ZIP entry reads
- `crates/core/src/sync.rs`: Multiple file operations

**Solution**:
- Batch file operations
- Use memory-mapped files for large documents
- Implement read-ahead caching for sequential access patterns

## Priority 3: Architecture Improvements

### 3.1 Add Profiling Infrastructure

**Action Items**:
- Add criterion benchmarks for hot paths
- Integrate perf/flamegraph support
- Create benchmark suite for library operations

### 3.2 Consider Rayon for Data Parallelism

**Candidate Operations**:
- Thumbnail generation batch processing
- Metadata extraction for multiple files
- Library sorting and filtering
- PDF page rendering

### 3.3 Optimize Memory Layout

**Actions**:
- Use #[repr(C)] for FFI structures
- Pack small structs with #[derive(Copy, Clone)]
- Consider structure-of-arrays vs array-of-structures for rendering data

## Quick Wins (Can be implemented in < 1 day each)

1. **Add with_capacity() to pre-sized collections** (~2 hours)
2. **Replace unnecessary clone() with references** (~4 hours)
3. **Implement Copy for Fp and small types** (~1 hour)
4. **Add string interning for paths** (~3 hours)
5. **Optimize logging to avoid string allocs** (~2 hours)

## Measurement Strategy

Before/after metrics to track:
1. Library scan time (1000 books)
2. Search/filter response time
3. Thumbnail generation throughput
4. UI frame rate during scrolling
5. Memory usage peak and average
6. Binary size impact

## Risk Assessment

**Low Risk**:
- Pre-allocation with with_capacity()
- Reducing unnecessary clones
- Copy trait implementations

**Medium Risk**:
- Async thumbnail generation
- Virtual scrolling implementation
- String interning

**High Risk**:
- Major architectural changes (rayon integration)
- Lock-free data structure replacements

## Recommended Implementation Order

1. Week 1: Quick wins (clones, pre-allocation, Copy traits)
2. Week 2: String optimization and logging improvements
3. Week 3: Async thumbnail generation
4. Week 4: Virtual scrolling and lazy loading
5. Week 5-6: Profiling-driven optimizations
6. Week 7-8: Architecture improvements (rayon, actor model)
