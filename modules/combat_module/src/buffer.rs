//! Check buffer for violation tracking with decay

use serde::{Deserialize, Serialize};

/// Check buffer with fail/pass tracking and VL management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckBuffer {
    /// Current buffer value (accumulator for fails)
    value: f64,
    /// Violation level
    vl: u32,
    /// Max VL threshold
    max_vl: u32,
    /// Buffer threshold before incrementing VL
    threshold: f64,
    /// Decay rate on pass (multiplier)
    decay: f64,
}

impl Default for CheckBuffer {
    fn default() -> Self {
        Self {
            value: 0.0,
            vl: 0,
            max_vl: 20,
            threshold: 1.0,
            decay: 0.95,
        }
    }
}

impl CheckBuffer {
    pub fn new(threshold: f64, max_vl: u32, decay: f64) -> Self {
        Self {
            value: 0.0,
            vl: 0,
            max_vl,
            threshold,
            decay,
        }
    }

    /// Record a failure, returns true if VL increased
    pub fn fail(&mut self) -> bool {
        self.fail_with(1.0)
    }

    /// Record a failure with custom weight
    pub fn fail_with(&mut self, weight: f64) -> bool {
        self.value += weight;
        if self.value >= self.threshold {
            self.vl = self.vl.saturating_add(1);
            self.value = 0.0;
            true
        } else {
            false
        }
    }

    /// Record a pass (decays buffer)
    pub fn pass(&mut self) {
        self.value *= self.decay;
        if self.value < 0.01 {
            self.value = 0.0;
        }
    }

    /// Get current buffer value
    pub fn get(&self) -> f64 {
        self.value
    }

    /// Get current VL
    pub fn vl(&self) -> u32 {
        self.vl
    }

    /// Get max VL
    pub fn max_vl(&self) -> u32 {
        self.max_vl
    }

    /// Check if over max VL threshold
    pub fn should_punish(&self) -> bool {
        self.vl >= self.max_vl
    }

    /// Reset buffer and VL
    pub fn reset(&mut self) {
        self.value = 0.0;
        self.vl = 0;
    }
}

/// Sample buffer for statistical analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleBuffer {
    samples: Vec<f64>,
    capacity: usize,
}

impl Default for SampleBuffer {
    fn default() -> Self {
        Self::new(20)
    }
}

impl SampleBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            samples: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, value: f64) {
        if self.samples.len() >= self.capacity {
            self.samples.remove(0);
        }
        self.samples.push(value);
    }

    pub fn is_full(&self) -> bool {
        self.samples.len() >= self.capacity
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &f64> {
        self.samples.iter()
    }

    pub fn last_n(&self, n: usize) -> Vec<f64> {
        self.samples.iter().rev().take(n).cloned().collect()
    }

    pub fn mean(&self) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }
        self.samples.iter().sum::<f64>() / self.samples.len() as f64
    }

    pub fn variance(&self) -> f64 {
        if self.samples.len() < 2 {
            return 0.0;
        }
        let mean = self.mean();
        let sum_sq: f64 = self.samples.iter().map(|x| (x - mean).powi(2)).sum();
        sum_sq / (self.samples.len() - 1) as f64
    }

    pub fn std_dev(&self) -> f64 {
        self.variance().sqrt()
    }

    pub fn kurtosis(&self) -> f64 {
        if self.samples.len() < 4 {
            return 0.0;
        }
        let mean = self.mean();
        let std_dev = self.std_dev();
        if std_dev < 0.0001 {
            return 0.0;
        }
        let n = self.samples.len() as f64;
        let sum_fourth: f64 = self.samples.iter().map(|x| ((x - mean) / std_dev).powi(4)).sum();
        (sum_fourth / n) - 3.0 // Excess kurtosis
    }

    pub fn distinct_count(&self) -> usize {
        let mut sorted: Vec<i64> = self.samples.iter().map(|x| (*x * 1000.0) as i64).collect();
        sorted.sort();
        sorted.dedup();
        sorted.len()
    }

    pub fn clear(&mut self) {
        self.samples.clear();
    }
}
