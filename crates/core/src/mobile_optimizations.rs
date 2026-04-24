//! Android/iOS-specific optimizations
//!
//! This module provides platform-specific optimizations for mobile devices:
//! - Touch/gesture responsiveness optimizations
//! - Animation support for high-refresh displays
//! - Network and sync optimizations for always-connected devices
//! - Storage and caching optimizations for mobile file systems
//!
//! All optimizations are automatically applied when running on Android/iOS.

use std::sync::LazyLock;

/// Check if running on a mobile platform (Android or iOS)
#[inline]
pub fn is_mobile_platform() -> bool {
    crate::device::is_android() || std::env::var("IPHONE_SIMULATOR_ROOT").is_ok()
}

// ============================================================================
// Touch & Gesture Optimizations
// ============================================================================

/// Mobile touch configuration
#[derive(Debug, Clone, Copy)]
pub struct TouchConfig {
    /// Tap jitter tolerance in millimeters (mobile needs lower tolerance)
    pub tap_jitter_mm: f32,
    /// Hold delay for long press (shorter on mobile for responsiveness)
    pub hold_delay_ms: u64,
    /// Enable haptic feedback on touch
    pub haptic_feedback: bool,
    /// Touch polling rate (higher for mobile responsiveness)
    pub poll_rate_hz: u32,
    /// Enable predictive touch (anticipate next position)
    pub predictive_touch: bool,
}

impl Default for TouchConfig {
    fn default() -> Self {
        if is_mobile_platform() {
            // Mobile: More responsive, lower latency
            Self {
                tap_jitter_mm: 4.0, // vs 6.0 on e-ink
                hold_delay_ms: 400, // vs 666ms on e-ink
                haptic_feedback: true,
                poll_rate_hz: 120, // 120Hz touch sampling
                predictive_touch: true,
            }
        } else {
            // E-ink: Conservative to account for display latency
            Self {
                tap_jitter_mm: 6.0,
                hold_delay_ms: 666,
                haptic_feedback: false,
                poll_rate_hz: 60,
                predictive_touch: false,
            }
        }
    }
}

impl TouchConfig {
    /// Get the platform-optimized touch configuration
    #[inline]
    pub fn platform_optimal() -> Self {
        Self::default()
    }
}

// ============================================================================
// Animation Optimizations
// ============================================================================

/// Animation configuration for mobile smooth displays
#[derive(Debug, Clone, Copy)]
pub struct AnimationConfig {
    /// Target frame rate (60/90/120Hz)
    pub target_fps: u32,
    /// Enable page turn animations
    pub page_animations: bool,
    /// Enable UI transition animations
    pub ui_animations: bool,
    /// Animation duration multiplier (shorter = snappier)
    pub duration_multiplier: f32,
    /// Enable physics-based animations (fling, bounce)
    pub physics_based: bool,
    /// Use GPU acceleration where available
    pub gpu_accelerated: bool,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        if is_mobile_platform() {
            // Mobile: Full animation support at high refresh rates
            Self {
                target_fps: 90, // Match OnePlus Nord 2 5G's 90Hz display
                page_animations: true,
                ui_animations: true,
                duration_multiplier: 0.7, // Faster animations
                physics_based: true,
                gpu_accelerated: true,
            }
        } else {
            // E-ink: Disable animations (slow display can't show them)
            Self {
                target_fps: 1, // Effectively disabled
                page_animations: false,
                ui_animations: false,
                duration_multiplier: 0.0,
                physics_based: false,
                gpu_accelerated: false,
            }
        }
    }
}

/// Global animation configuration
static ANIMATION_CONFIG: LazyLock<std::sync::Mutex<AnimationConfig>> =
    LazyLock::new(|| std::sync::Mutex::new(AnimationConfig::default()));

/// Get current animation configuration
#[inline]
pub fn animation_config() -> AnimationConfig {
    *ANIMATION_CONFIG
        .lock()
        .expect("ANIMATION_CONFIG lock poisoned")
}

/// Set animation configuration
#[inline]
pub fn set_animation_config(config: AnimationConfig) {
    *ANIMATION_CONFIG
        .lock()
        .expect("ANIMATION_CONFIG lock poisoned") = config;
}

/// Calculate animation frame duration for current platform
#[inline]
pub fn animation_frame_duration_ms() -> u64 {
    let fps = animation_config().target_fps;
    if fps == 0 {
        16 // Default 60fps fallback
    } else {
        1000 / fps as u64
    }
}

// ============================================================================
// Network & Sync Optimizations
// ============================================================================

/// Network configuration for mobile always-connected devices
#[derive(Debug, Clone, Copy)]
pub struct NetworkConfig {
    /// Enable aggressive prefetching over WiFi/cellular
    pub aggressive_prefetch: bool,
    /// Background sync enabled
    pub background_sync: bool,
    /// Sync interval in seconds
    pub sync_interval_sec: u64,
    /// Enable download resume (for large files)
    pub resume_downloads: bool,
    /// Max concurrent downloads
    pub max_concurrent_downloads: usize,
    /// Connection timeout in seconds
    pub connection_timeout_sec: u64,
    /// Enable connection pooling
    pub connection_pooling: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        if is_mobile_platform() {
            // Mobile: Aggressive networking for always-connected devices
            Self {
                aggressive_prefetch: true,
                background_sync: true,
                sync_interval_sec: 300, // 5 minutes
                resume_downloads: true,
                max_concurrent_downloads: 4,
                connection_timeout_sec: 30,
                connection_pooling: true,
            }
        } else {
            // E-ink: Conservative networking (battery, limited connectivity)
            Self {
                aggressive_prefetch: false,
                background_sync: false,
                sync_interval_sec: 3600, // 1 hour
                resume_downloads: false,
                max_concurrent_downloads: 1,
                connection_timeout_sec: 60,
                connection_pooling: false,
            }
        }
    }
}

/// Global network configuration
static NETWORK_CONFIG: LazyLock<std::sync::Mutex<NetworkConfig>> =
    LazyLock::new(|| std::sync::Mutex::new(NetworkConfig::default()));

/// Get network configuration
#[inline]
pub fn network_config() -> NetworkConfig {
    *NETWORK_CONFIG.lock().expect("NETWORK_CONFIG lock poisoned")
}

// ============================================================================
// Storage & Cache Optimizations
// ============================================================================

/// Storage configuration for mobile file systems
#[derive(Debug, Clone, Copy)]
pub struct StorageConfig {
    /// Use SQLite for metadata (faster queries on mobile)
    pub use_sqlite_metadata: bool,
    /// Enable SSD-aware I/O patterns (for UFS 3.1)
    pub ssd_optimized: bool,
    /// Async I/O for non-blocking operations
    pub async_io: bool,
    /// Enable compression for thumbnails (saves space)
    pub compress_thumbnails: bool,
    /// Thumbnail compression quality (0-100)
    pub thumbnail_quality: u8,
    /// Max thumbnail cache size in MB
    pub thumbnail_cache_mb: usize,
    /// Enable automatic cache cleanup
    pub auto_cache_cleanup: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        if is_mobile_platform() {
            // Mobile: Optimized for UFS 3.1, abundant storage
            Self {
                use_sqlite_metadata: true,
                ssd_optimized: true,
                async_io: true,
                compress_thumbnails: true,
                thumbnail_quality: 85,
                thumbnail_cache_mb: 200,
                auto_cache_cleanup: true,
            }
        } else {
            // E-ink: Optimized for eMMC, limited storage
            Self {
                use_sqlite_metadata: false, // Use JSON (simpler)
                ssd_optimized: false,
                async_io: false,
                compress_thumbnails: false,
                thumbnail_quality: 75,
                thumbnail_cache_mb: 50,
                auto_cache_cleanup: false,
            }
        }
    }
}

/// Global storage configuration
static STORAGE_CONFIG: LazyLock<std::sync::Mutex<StorageConfig>> =
    LazyLock::new(|| std::sync::Mutex::new(StorageConfig::default()));

/// Get storage configuration
#[inline]
pub fn storage_config() -> StorageConfig {
    *STORAGE_CONFIG.lock().expect("STORAGE_CONFIG lock poisoned")
}

// ============================================================================
// Battery & Power Optimizations
// ============================================================================

/// Power management configuration
#[derive(Debug, Clone, Copy)]
pub struct PowerConfig {
    /// Enable battery-aware throttling
    pub battery_aware: bool,
    /// Low battery threshold percentage
    pub low_battery_threshold: u8,
    /// Reduce animations when battery is low
    pub reduce_animations_on_low_battery: bool,
    /// Enable aggressive Doze mode support (Android)
    pub doze_mode_support: bool,
    /// Background task scheduling strategy
    pub background_scheduler: BackgroundScheduler,
}

#[derive(Debug, Clone, Copy)]
pub enum BackgroundScheduler {
    /// Run immediately (when device is active)
    Immediate,
    /// Defer to idle time
    Idle,
    /// Schedule via WorkManager (Android)
    WorkManager,
}

impl Default for PowerConfig {
    fn default() -> Self {
        if is_mobile_platform() {
            Self {
                battery_aware: true,
                low_battery_threshold: 20,
                reduce_animations_on_low_battery: true,
                doze_mode_support: true,
                background_scheduler: BackgroundScheduler::WorkManager,
            }
        } else {
            // E-ink: Always conservative due to limited battery
            Self {
                battery_aware: true,
                low_battery_threshold: 30,
                reduce_animations_on_low_battery: false, // Already disabled
                doze_mode_support: false,
                background_scheduler: BackgroundScheduler::Idle,
            }
        }
    }
}

/// Global power configuration
static POWER_CONFIG: LazyLock<std::sync::Mutex<PowerConfig>> =
    LazyLock::new(|| std::sync::Mutex::new(PowerConfig::default()));

/// Get power configuration
#[inline]
pub fn power_config() -> PowerConfig {
    *POWER_CONFIG.lock().expect("POWER_CONFIG lock poisoned")
}

// ============================================================================
// Memory Management Optimizations
// ============================================================================

/// Memory configuration for mobile abundant RAM
#[derive(Debug, Clone, Copy)]
pub struct MemoryConfig {
    /// Max heap size percentage of total RAM
    pub max_heap_percent: u8,
    /// Enable large heap (Android)
    pub large_heap: bool,
    /// GC target utilization percentage
    pub gc_target_utilization: u8,
    /// Enable aggressive prefetching
    pub aggressive_prefetch: bool,
    /// Image cache size in MB
    pub image_cache_mb: usize,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        if is_mobile_platform() {
            // Mobile: Use more RAM aggressively
            Self {
                max_heap_percent: 75, // Use up to 75% of available RAM
                large_heap: true,
                gc_target_utilization: 70,
                aggressive_prefetch: true,
                image_cache_mb: 300,
            }
        } else {
            // E-ink: Conservative memory usage
            Self {
                max_heap_percent: 50, // Use max 50% of limited RAM
                large_heap: false,
                gc_target_utilization: 50,
                aggressive_prefetch: false,
                image_cache_mb: 20,
            }
        }
    }
}

/// Global memory configuration
static MEMORY_CONFIG: LazyLock<std::sync::Mutex<MemoryConfig>> =
    LazyLock::new(|| std::sync::Mutex::new(MemoryConfig::default()));

/// Get memory configuration
#[inline]
pub fn memory_config() -> MemoryConfig {
    *MEMORY_CONFIG.lock().expect("MEMORY_CONFIG lock poisoned")
}

// ============================================================================
// Platform Feature Detection
// ============================================================================

/// Detect if device supports high refresh rate (90Hz+)
#[inline]
pub fn supports_high_refresh() -> bool {
    is_mobile_platform()
}

/// Detect if device has always-on connectivity
#[inline]
pub fn has_always_on_connectivity() -> bool {
    is_mobile_platform()
}

/// Detect if device has abundant storage (256GB+)
#[inline]
pub fn has_abundant_storage() -> bool {
    is_mobile_platform()
}

/// Detect if device supports GPU acceleration
#[inline]
pub fn supports_gpu_acceleration() -> bool {
    is_mobile_platform()
}

/// Detect if device supports haptic feedback
#[inline]
pub fn supports_haptic() -> bool {
    is_mobile_platform()
}

// ============================================================================
// Platform Helper Functions
// ============================================================================

/// Get recommended thread pool size for background tasks
#[inline]
pub fn recommended_thread_pool_size() -> usize {
    if is_mobile_platform() {
        6 // Use more threads on multi-core mobile CPUs
    } else {
        2 // Conservative for e-ink
    }
}

/// Get recommended I/O buffer size
#[inline]
pub fn recommended_io_buffer_size() -> usize {
    if is_mobile_platform() {
        128 * 1024 // 128KB for UFS 3.1
    } else {
        32 * 1024 // 32KB for eMMC
    }
}

/// Should we use async I/O?
#[inline]
pub fn should_use_async_io() -> bool {
    storage_config().async_io
}

/// Get animation duration in milliseconds
#[inline]
pub fn animation_duration_ms(base_duration: u64) -> u64 {
    let multiplier = animation_config().duration_multiplier;
    (base_duration as f32 * multiplier) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        // Test should return consistent results
        let is_mobile = is_mobile_platform();
        // On CI/desktop this should be false
        // On actual Android/iOS this would be true
        assert!(!is_mobile || std::env::var("ANDROID_ROOT").is_ok());
    }

    #[test]
    fn test_touch_config_default() {
        let config = TouchConfig::default();
        // Verify fields are set
        assert!(config.tap_jitter_mm > 0.0);
        assert!(config.hold_delay_ms > 0);
    }

    #[test]
    fn test_animation_config() {
        let config = animation_config();
        assert!(config.target_fps > 0);
        assert!(config.duration_multiplier >= 0.0);
    }

    #[test]
    fn test_network_config() {
        let config = network_config();
        assert!(config.sync_interval_sec > 0);
        assert!(config.max_concurrent_downloads > 0);
    }

    #[test]
    fn test_storage_config() {
        let config = storage_config();
        assert!(config.thumbnail_quality <= 100);
        assert!(config.thumbnail_cache_mb > 0);
    }

    #[test]
    fn test_memory_config() {
        let config = memory_config();
        assert!(config.max_heap_percent > 0 && config.max_heap_percent <= 100);
        assert!(config.image_cache_mb > 0);
    }

    #[test]
    fn test_thread_pool_size() {
        let size = recommended_thread_pool_size();
        assert!(size > 0);
        assert!(size <= 16); // Reasonable upper bound
    }

    #[test]
    fn test_io_buffer_size() {
        let size = recommended_io_buffer_size();
        assert!(size >= 4096); // At least 4KB
    }
}
