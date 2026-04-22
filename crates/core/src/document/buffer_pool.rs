//! Buffer pooling for memory optimization
//! 
//! This module implements buffer reuse and pooling to reduce memory allocations
//! during PDF rendering and text extraction operations.

use anyhow::Result;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::alloc::Layout;

/// Reusable buffer for pixel data with SIMD alignment
#[derive(Debug)]
pub struct PixelBuffer {
    data: Vec<u8>,
    capacity: usize,
}

impl PixelBuffer {
    /// Create a new buffer with specified capacity and SIMD alignment
    pub fn new(capacity: usize) -> Self {
        let aligned_capacity = (capacity + 31) & !31; // 32-byte alignment
        Self {
            data: Vec::with_capacity(aligned_capacity),
            capacity: aligned_capacity,
        }
    }

    /// Create a new SIMD-aligned buffer for specific size
    pub fn new_aligned(size: usize) -> Self {
        let layout = Layout::from_size_align(size, 32)
            .expect("Invalid alignment layout");
        let ptr = unsafe { std::alloc::alloc(layout) };
        let vec = if ptr.is_null() {
            Vec::with_capacity(size)
        } else {
            unsafe {
                let mut vec = Vec::from_raw_parts(ptr, size, size);
                vec.set_len(size);
                vec
            }
        };
        
        Self {
            data: vec,
            capacity: size,
        }
    }

impl PixelBuffer {
    /// Create a new buffer with specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            capacity,
        }
    }

    /// Get the underlying data
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get mutable data
    pub fn data_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Ensure capacity
    pub fn ensure_capacity(&mut self, capacity: usize) {
        if capacity > self.capacity {
            self.data.reserve(capacity - self.data.len());
            self.capacity = capacity;
        }
    }

    /// Get current capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

/// Pool of reusable buffers
pub struct BufferPool {
    pool: Arc<Mutex<VecDeque<PixelBuffer>>>,
    min_buffer_size: usize,
    max_buffers: usize,
}

impl BufferPool {
    /// Create a new buffer pool
    pub fn new(min_buffer_size: usize, max_buffers: usize) -> Self {
        Self {
            pool: Arc::new(Mutex::new(VecDeque::with_capacity(max_buffers))),
            min_buffer_size,
            max_buffers,
        }
    }

    /// Acquire a buffer from the pool
    pub fn acquire(&self, min_size: usize) -> Result<BufferGuard> {
        let size = min_size.max(self.min_buffer_size);
        let mut pool = self.pool.lock().unwrap();
        
        // Try to find a buffer with sufficient capacity
        let buffer = if let Some(mut buf) = pool.pop_front() {
            buf.ensure_capacity(size);
            buf
        } else {
            PixelBuffer::new(size)
        };

        Ok(BufferGuard {
            buffer: Some(buffer),
            pool: Arc::clone(&self.pool),
        })
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        let pool = self.pool.lock().unwrap();
        PoolStats {
            available: pool.len(),
            min_buffer_size: self.min_buffer_size,
            max_buffers: self.max_buffers,
        }
    }

    /// Clear the pool
    pub fn clear(&self) {
        self.pool.lock().unwrap().clear();
    }
}

impl Default for BufferPool {
    fn default() -> Self {
        Self::new(1024 * 1024, 8) // 1MB minimum, 8 buffers max
    }
}

/// Guard that returns buffer to pool when dropped
pub struct BufferGuard {
    buffer: Option<PixelBuffer>,
    pool: Arc<Mutex<VecDeque<PixelBuffer>>>,
}

impl BufferGuard {
    /// Get the buffer
    pub fn buffer(&mut self) -> &mut PixelBuffer {
        self.buffer.as_mut().unwrap()
    }

    /// Consume the guard and return the buffer without returning to pool
    pub fn into_inner(mut self) -> PixelBuffer {
        self.buffer.take().unwrap()
    }
}

impl Drop for BufferGuard {
    fn drop(&mut self) {
        if let Some(buffer) = self.buffer.take() {
            let mut pool = self.pool.lock().unwrap();
            if pool.len() < pool.capacity() {
                pool.push_back(buffer);
            }
        }
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub available: usize,
    pub min_buffer_size: usize,
    pub max_buffers: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_pool_acquire_release() {
        let pool = BufferPool::new(1024, 4);
        
        {
            let mut guard = pool.acquire(2048).unwrap();
            let buffer = guard.buffer();
            assert!(buffer.capacity() >= 2048);
        }
        
        // Buffer should be returned to pool
        let stats = pool.stats();
        assert_eq!(stats.available, 1);
    }

    #[test]
    fn test_buffer_pool_capacity_limit() {
        let pool = BufferPool::new(1024, 2);
        
        let _guard1 = pool.acquire(1024).unwrap();
        let _guard2 = pool.acquire(1024).unwrap();
        let _guard3 = pool.acquire(1024).unwrap(); // Should allocate new
        
        let stats = pool.stats();
        assert_eq!(stats.available, 0); // None returned yet
    }

    #[test]
    fn test_buffer_clear() {
        let pool = BufferPool::new(1024, 4);
        
        let mut guard = pool.acquire(1024).unwrap();
        let buffer = guard.buffer();
        buffer.data_mut().extend_from_slice(&[1, 2, 3, 4]);
        buffer.clear();
        
        assert!(buffer.data().is_empty());
    }
}
