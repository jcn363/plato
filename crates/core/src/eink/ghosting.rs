//! Ghosting reduction for e-ink displays
//!
//! Tracks partial updates and forces full refreshes to prevent ghosting artifacts.

use anyhow::Result;

/// Reduces ghosting by managing refresh cycles
#[derive(Debug)]
pub struct GhostingReducer {
    partial_update_count: u32,
    max_partial_updates: u32,
    full_refresh_interval: u32,
    last_full_refresh: std::time::Instant,
}

impl GhostingReducer {
    pub fn new(max_partial_updates: u32, full_refresh_interval_sec: u32) -> Self {
        let max_partial_updates = max_partial_updates.clamp(1, 1000);
        let full_refresh_interval = full_refresh_interval_sec.clamp(1, 3600);
        Self {
            partial_update_count: 0,
            max_partial_updates,
            full_refresh_interval,
            last_full_refresh: std::time::Instant::now(),
        }
    }

    pub fn register_partial_update(&mut self) {
        self.partial_update_count += 1;
    }

    pub fn register_full_refresh(&mut self) {
        self.partial_update_count = 0;
        self.last_full_refresh = std::time::Instant::now();
    }

    pub fn should_force_full_refresh(&self) -> bool {
        self.partial_update_count >= self.max_partial_updates
            || self.last_full_refresh.elapsed().as_secs() >= self.full_refresh_interval as u64
    }

    pub fn get_partial_count(&self) -> u32 {
        self.partial_update_count
    }

    pub fn reset(&mut self) {
        self.partial_update_count = 0;
        self.last_full_refresh = std::time::Instant::now();
    }

    pub fn set_max_partial_updates(&mut self, max: u32) -> Result<()> {
        if max == 0 {
            anyhow::bail!("Max partial updates must be greater than 0");
        }
        self.max_partial_updates = max;
        Ok(())
    }

    pub fn set_full_refresh_interval(&mut self, interval_sec: u32) -> Result<()> {
        if interval_sec == 0 {
            anyhow::bail!("Full refresh interval must be greater than 0");
        }
        self.full_refresh_interval = interval_sec;
        Ok(())
    }
}

impl Default for GhostingReducer {
    fn default() -> Self {
        Self::new(10, 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reducer_creation() {
        let reducer = GhostingReducer::new(5, 30);
        assert_eq!(reducer.partial_update_count, 0);
        assert_eq!(reducer.max_partial_updates, 5);
    }

    #[test]
    fn test_register_partial_update() {
        let mut reducer = GhostingReducer::new(5, 30);
        reducer.register_partial_update();
        assert_eq!(reducer.partial_update_count, 1);
    }

    #[test]
    fn test_should_force_full_refresh() {
        let mut reducer = GhostingReducer::new(3, 30);
        assert!(!reducer.should_force_full_refresh());

        reducer.register_partial_update();
        reducer.register_partial_update();
        assert!(!reducer.should_force_full_refresh());

        reducer.register_partial_update();
        assert!(reducer.should_force_full_refresh());
    }

    #[test]
    fn test_register_full_refresh() {
        let mut reducer = GhostingReducer::new(3, 30);
        reducer.register_partial_update();
        reducer.register_partial_update();
        reducer.register_full_refresh();
        assert_eq!(reducer.partial_update_count, 0);
        assert!(!reducer.should_force_full_refresh());
    }

    #[test]
    fn test_set_max_partial_updates() {
        let mut reducer = GhostingReducer::new(5, 30);
        assert!(reducer.set_max_partial_updates(10).is_ok());
        assert_eq!(reducer.max_partial_updates, 10);
        assert!(reducer.set_max_partial_updates(0).is_err());
    }

    #[test]
    fn test_set_full_refresh_interval() {
        let mut reducer = GhostingReducer::new(5, 30);
        assert!(reducer.set_full_refresh_interval(60).is_ok());
        assert_eq!(reducer.full_refresh_interval, 60);
        assert!(reducer.set_full_refresh_interval(0).is_err());
    }
}
