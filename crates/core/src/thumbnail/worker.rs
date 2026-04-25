use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, LazyLock, Mutex};
use std::thread;

use crate::buffer_pool;
use crate::document::open;
use crate::framebuffer::Framebuffer;
use crate::thumbnail::error::{ThumbnailError, ThumbnailResult};
use crate::thumbnail::request::ThumbnailRequest;

/// Global mutex to prevent concurrent access when loading multiple JP2 images
static EXCLUSIVE_ACCESS: LazyLock<Mutex<u8>> = LazyLock::new(|| Mutex::new(0));

/// Worker thread that processes thumbnail generation requests
pub struct ThumbnailWorker {
    receiver: Arc<Mutex<Receiver<ThumbnailRequest>>>,
}

impl ThumbnailWorker {
    /// Creates a new thumbnail worker with a shared receiver
    pub fn new(receiver: Arc<Mutex<Receiver<ThumbnailRequest>>>) -> Self {
        Self { receiver }
    }

    /// Starts the worker thread
    pub fn start(self) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            loop {
                // Safely receive a request from the shared channel
                let request = {
                    match self.receiver.lock() {
                        Ok(rx) => match rx.recv() {
                            Ok(req) => req,
                            Err(_) => break, // Channel closed
                        },
                        Err(_) => {
                            eprintln!("Thumbnail worker lock poisoned, shutting down");
                            break;
                        }
                    }
                };

                let result = self.process_request(&request);

                // Send result back to requester
                if request.response_tx.send(result).is_err() {
                    // Log error but continue processing other requests
                    eprintln!("Failed to send thumbnail generation result");
                }
            }
        })
    }

    /// Processes a single thumbnail generation request
    fn process_request(&self, request: &ThumbnailRequest) -> ThumbnailResult<PathBuf> {
        // Validate request
        request.validate()?;

        // Check if thumbnail already exists
        if request.thumbnail_path.exists() {
            return Ok(request.thumbnail_path.clone());
        }

        // Generate thumbnail with exclusive access to prevent segfaults
        let _guard = EXCLUSIVE_ACCESS
            .lock()
            .map_err(|_| ThumbnailError::thread_pool("EXCLUSIVE_ACCESS lock poisoned"))?;

        // Open document and generate preview using buffer pool
        let full_path = request.file_path.clone();
        let pixmap = buffer_pool::with_thumbnail_buffer(|_buffer| {
            open(full_path)
                .and_then(|mut doc| {
                    doc.preview_pixmap(
                        request.dimensions.0 as f32,
                        request.dimensions.1 as f32,
                        crate::device::CURRENT_DEVICE.color_samples(),
                    )
                })
                .ok_or_else(|| {
                    ThumbnailError::document_processing(
                        anyhow::anyhow!("Failed to generate preview pixmap"),
                        request.file_path.display().to_string(),
                    )
                })
        })?;

        // Save thumbnail to disk
        pixmap
            .save(request.thumbnail_path.to_str().unwrap_or("thumbnail.png"))
            .map_err(|_| {
                ThumbnailError::save_failed(request.thumbnail_path.display().to_string())
            })?;

        Ok(request.thumbnail_path.clone())
    }
}

/// Worker pool for managing multiple thumbnail generation threads
pub struct ThumbnailWorkerPool {
    handles: Vec<thread::JoinHandle<()>>,
    sender: Sender<ThumbnailRequest>,
}

impl ThumbnailWorkerPool {
    /// Creates a new worker pool with the specified number of workers
    pub fn new(worker_count: usize) -> ThumbnailResult<Self> {
        if worker_count == 0 {
            return Err(ThumbnailError::thread_pool("worker count cannot be zero"));
        }

        // Limit maximum workers to prevent resource exhaustion
        let max_workers = if crate::device::is_android() { 8 } else { 4 };
        let actual_count = worker_count.min(max_workers);

        let (sender, receiver) = mpsc::channel::<ThumbnailRequest>();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut handles = Vec::with_capacity(actual_count);

        for _ in 0..actual_count {
            let worker = ThumbnailWorker::new(receiver.clone());
            handles.push(worker.start());
        }

        Ok(Self { handles, sender })
    }

    /// Submits a thumbnail generation request to the worker pool
    pub fn submit(&self, request: ThumbnailRequest) -> ThumbnailResult<()> {
        request.validate()?;
        self.sender
            .send(request)
            .map_err(|_| ThumbnailError::Channel)?;
        Ok(())
    }

    /// Shuts down the worker pool and waits for all workers to finish
    pub fn shutdown(mut self) -> ThumbnailResult<()> {
        // Drop the sender to close the channel
        let _sender = std::mem::replace(&mut self.sender, mpsc::channel().0);

        // Wait for all workers to finish
        for handle in self.handles.drain(..) {
            handle
                .join()
                .map_err(|_| ThumbnailError::thread_pool("failed to join worker thread"))?;
        }

        Ok(())
    }
}

impl Drop for ThumbnailWorkerPool {
    fn drop(&mut self) {
        // Best effort cleanup - don't panic in drop
        // Create a dummy request to close the channel if needed
        let (tx, _) = mpsc::channel::<ThumbnailResult<PathBuf>>();
        let _ = self.sender.send(ThumbnailRequest {
            file_path: PathBuf::new(),
            thumbnail_path: PathBuf::new(),
            dimensions: (0, 0),
            response_tx: tx,
        });
    }
}

impl Clone for ThumbnailWorkerPool {
    fn clone(&self) -> Self {
        // Create a new worker pool with a default number of workers
        // Try 2 workers first, fall back to 1 if that fails
        Self::new(2)
            .or_else(|_| Self::new(1))
            .unwrap_or_else(|_| {
                // If even 1 worker fails, we have a critical system error.
                // Log and return a pool with 0 workers (effectively disabling it)
                // rather than panicking in production.
                eprintln!("CRITICAL: Failed to create thumbnail worker pool fallback");
                Self {
                    handles: Vec::new(),
                    sender: mpsc::channel().0,
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    fn create_test_request() -> (ThumbnailRequest, mpsc::Receiver<ThumbnailResult<PathBuf>>) {
        let (tx, rx) = mpsc::channel();
        let request = ThumbnailRequest::new(
            PathBuf::from("test.pdf"),
            PathBuf::from("test.png"),
            (100, 100),
            tx,
        );
        (request, rx)
    }

    #[test]
    fn test_worker_pool_new() {
        let pool = ThumbnailWorkerPool::new(2);
        assert!(pool.is_ok());
    }

    #[test]
    fn test_worker_pool_zero_workers() {
        let pool = ThumbnailWorkerPool::new(0);
        assert!(pool.is_err());
    }

    #[test]
    fn test_worker_pool_too_many_workers() {
        let pool = ThumbnailWorkerPool::new(10);
        assert!(pool.is_err());
    }

    #[test]
    fn test_request_validation() {
        let (request, _) = create_test_request();

        // Valid request should pass
        assert!(request.validate().is_ok());

        // Invalid dimensions should fail
        let (tx, _) = mpsc::channel();
        let invalid_request = ThumbnailRequest::new(
            PathBuf::from("test.pdf"),
            PathBuf::from("test.png"),
            (0, 0), // Invalid dimensions
            tx,
        );
        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_worker_pool_shutdown() {
        let pool = ThumbnailWorkerPool::new(1).unwrap();
        let result = pool.shutdown();
        assert!(result.is_ok());
    }

    #[test]
    fn test_worker_pool_clone() {
        let pool = ThumbnailWorkerPool::new(2).unwrap();
        let cloned = pool.clone();
        // Should not panic and should create a valid pool
        let result = cloned.shutdown();
        assert!(result.is_ok());
    }
}
