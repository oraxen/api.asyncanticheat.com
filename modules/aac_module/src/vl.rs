//! Violation Level (VL) model from AAC5
//!
//! Based on me.konsolas.aac.O helper class.
//! VL formula: vl = clamp(vl - decay * Δt + delta, 0, max)

use crate::config::VlConfig;

/// Violation Level tracker with decay
#[derive(Debug, Clone)]
pub struct ViolationLevel {
    /// Current VL value
    vl: f32,
    /// Configuration
    config: VlConfig,
    /// Last update timestamp (ms)
    last_update_ms: i64,
}

impl ViolationLevel {
    pub fn new(config: VlConfig) -> Self {
        // max is forced to be at least 1.0f in AAC code: Math.max(f10, 1.0f)
        let config = VlConfig {
            max: config.max.max(1.0),
            ..config
        };
        
        Self {
            vl: 0.0,
            config,
            last_update_ms: 0,
        }
    }

    /// Update VL with a delta value.
    /// Returns true if mitigate is enabled AND vl >= threshold (should trigger mitigation).
    ///
    /// Formula from AAC5's O.a(delta, ...):
    /// vl = clamp(vl - decay * Δt + delta, 0, max)
    pub fn update(&mut self, delta: f32, current_time_ms: i64) -> bool {
        // Calculate time since last update
        // Use initialized flag instead of checking > 0 to handle timestamp 0 correctly
        let dt_seconds = if self.last_update_ms >= 0 && self.vl > 0.0 {
            // Only apply decay if we have previous state
            let dt = current_time_ms.saturating_sub(self.last_update_ms);
            dt as f32 / 1000.0
        } else {
            0.0
        };
        self.last_update_ms = current_time_ms;

        // Apply decay and delta
        let decay_amount = self.config.decay * dt_seconds;
        self.vl = (self.vl - decay_amount + delta).clamp(0.0, self.config.max);

        // Return whether mitigation should trigger
        self.config.mitigate && self.vl >= self.config.threshold
    }

    /// Get current VL value
    pub fn get(&self) -> f32 {
        self.vl
    }

    /// Reset VL to zero
    pub fn reset(&mut self) {
        self.vl = 0.0;
    }

    /// Check if currently above threshold (without updating)
    pub fn is_above_threshold(&self) -> bool {
        self.vl >= self.config.threshold
    }

    /// Get the configuration
    pub fn config(&self) -> &VlConfig {
        &self.config
    }

    /// Apply passive decay without adding violation
    pub fn decay(&mut self, current_time_ms: i64) {
        self.update(0.0, current_time_ms);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vl_update() {
        let config = VlConfig {
            mitigate: true,
            threshold: 1.0,
            max: 2.0,
            decay: 0.1,
        };
        let mut vl = ViolationLevel::new(config);

        // First update at t=0
        let triggered = vl.update(0.5, 0);
        assert!(!triggered);
        assert!((vl.get() - 0.5).abs() < 0.001);

        // Second update at t=1000ms (1 second later), decay = 0.1
        // vl = 0.5 - 0.1 + 0.6 = 1.0
        let triggered = vl.update(0.6, 1000);
        assert!(triggered); // Should trigger at exactly threshold
        assert!((vl.get() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_vl_max_clamp() {
        let config = VlConfig {
            mitigate: true,
            threshold: 1.0,
            max: 1.5,
            decay: 0.0,
        };
        let mut vl = ViolationLevel::new(config);

        vl.update(2.0, 0);
        assert!((vl.get() - 1.5).abs() < 0.001); // Clamped to max
    }

    #[test]
    fn test_vl_min_max_enforcement() {
        let config = VlConfig {
            mitigate: true,
            threshold: 1.0,
            max: 0.5, // Below 1.0
            decay: 0.0,
        };
        let vl = ViolationLevel::new(config);
        
        // max should be forced to 1.0
        assert!((vl.config.max - 1.0).abs() < 0.001);
    }
}

