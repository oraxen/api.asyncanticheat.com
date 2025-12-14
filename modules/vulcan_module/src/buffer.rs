//! Vulcan Buffer System
//!
//! Most Vulcan checks use a buffer that:
//! - Increments on suspicious activity (multiplied by buffer_multiple)
//! - Decrements otherwise (by buffer_decay)
//! - Violations trigger when buffer exceeds max_buffer

use serde::{Deserialize, Serialize};

/// Buffer configuration for a check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferConfig {
    /// Maximum buffer value before triggering
    pub max: f64,
    /// Multiplier applied to buffer on fail (typically 0.25-0.75)
    pub multiple: f64,
    /// Amount subtracted on pass (decay)
    pub decay: f64,
}

impl Default for BufferConfig {
    fn default() -> Self {
        Self {
            max: 5.0,
            multiple: 0.5,
            decay: 0.25,
        }
    }
}

/// Check buffer with Vulcan-style behavior
#[derive(Debug, Clone)]
pub struct CheckBuffer {
    /// Current buffer value
    value: f64,
    /// Configuration
    config: BufferConfig,
    /// Violation count
    vl: u32,
    /// Maximum violations before punishment
    max_vl: u32,
    /// Minimum VL to start alerting
    min_vl_alert: u32,
    /// Alert interval
    alert_interval: u32,
    /// Whether this check is punishable
    punishable: bool,
}

impl CheckBuffer {
    pub fn new(config: BufferConfig, max_vl: u32) -> Self {
        Self {
            value: 0.0,
            config,
            vl: 0,
            max_vl,
            min_vl_alert: 1,
            alert_interval: 1,
            punishable: true,
        }
    }

    pub fn with_alert_settings(mut self, min_vl: u32, interval: u32) -> Self {
        self.min_vl_alert = min_vl;
        self.alert_interval = interval;
        self
    }

    pub fn with_punishable(mut self, punishable: bool) -> Self {
        self.punishable = punishable;
        self
    }

    /// Called when check fails - increments buffer
    /// Returns true if violation should be flagged
    pub fn fail(&mut self) -> bool {
        // Increment buffer by 1, then apply multiplier (multiplier < 1.0 dampens growth)
        self.value = (self.value + 1.0) * self.config.multiple;
        
        if self.value > self.config.max {
            self.vl += 1;
            // Reset buffer after violation
            self.value = self.value.min(self.config.max);
            true
        } else {
            false
        }
    }

    /// Called when check fails with a specific increment
    /// Returns true if violation should be flagged
    pub fn fail_with(&mut self, increment: f64) -> bool {
        self.value += increment;
        
        if self.value > self.config.max {
            self.vl += 1;
            self.value = self.config.max;
            true
        } else {
            false
        }
    }

    /// Called when check passes - decays buffer
    pub fn pass(&mut self) {
        self.value = (self.value - self.config.decay).max(0.0);
    }

    /// Decay buffer without checking
    pub fn decay(&mut self) {
        self.value = (self.value - self.config.decay).max(0.0);
    }

    /// Get current buffer value
    pub fn get(&self) -> f64 {
        self.value
    }

    /// Get current VL
    pub fn vl(&self) -> u32 {
        self.vl
    }

    /// Check if should alert
    pub fn should_alert(&self) -> bool {
        self.vl >= self.min_vl_alert && (self.vl % self.alert_interval == 0 || self.alert_interval == 1)
    }

    /// Check if should punish
    pub fn should_punish(&self) -> bool {
        self.punishable && self.vl >= self.max_vl
    }

    /// Reset buffer and VL
    pub fn reset(&mut self) {
        self.value = 0.0;
        self.vl = 0;
    }

    /// Get max VL
    pub fn max_vl(&self) -> u32 {
        self.max_vl
    }
}

/// Pre-configured buffer for common check patterns
impl CheckBuffer {
    /// Aim A style buffer (tight, fast triggering)
    pub fn aim_a() -> Self {
        Self::new(
            BufferConfig { max: 3.0, multiple: 0.75, decay: 0.5 },
            1,
        )
    }

    /// Aim B style buffer (more tolerant)
    pub fn aim_b() -> Self {
        Self::new(
            BufferConfig { max: 6.0, multiple: 0.5, decay: 0.65 },
            8,
        )
    }

    /// Auto Clicker style buffer (statistical)
    pub fn autoclicker() -> Self {
        Self::new(
            BufferConfig { max: 5.0, multiple: 0.5, decay: 0.25 },
            10,
        )
    }

    /// Velocity style buffer (sensitive)
    pub fn velocity() -> Self {
        Self::new(
            BufferConfig { max: 2.0, multiple: 0.5, decay: 0.1 },
            5,
        )
    }

    /// Speed style buffer (movement)
    pub fn speed() -> Self {
        Self::new(
            BufferConfig { max: 4.0, multiple: 0.5, decay: 0.25 },
            10,
        )
    }

    /// Flight style buffer
    pub fn flight() -> Self {
        Self::new(
            BufferConfig { max: 3.0, multiple: 0.5, decay: 0.5 },
            5,
        )
    }

    /// Bad Packets style buffer (strict)
    pub fn bad_packets() -> Self {
        Self::new(
            BufferConfig { max: 1.0, multiple: 1.0, decay: 0.0 },
            1,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_fail() {
        let mut buffer = CheckBuffer::new(
            BufferConfig { max: 3.0, multiple: 1.0, decay: 0.5 },
            1,
        );

        // First three fails should not trigger
        assert!(!buffer.fail());
        assert!(!buffer.fail());
        assert!(!buffer.fail());
        
        // Fourth fail should trigger
        assert!(buffer.fail());
        assert_eq!(buffer.vl(), 1);
    }

    #[test]
    fn test_buffer_decay() {
        let mut buffer = CheckBuffer::new(
            BufferConfig { max: 5.0, multiple: 1.0, decay: 1.0 },
            1,
        );

        buffer.fail();
        buffer.fail();
        assert_eq!(buffer.get(), 2.0);

        buffer.pass();
        assert_eq!(buffer.get(), 1.0);

        buffer.pass();
        assert_eq!(buffer.get(), 0.0);
    }
}

