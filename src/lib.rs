#![forbid(unsafe_code)]

//! Sensor data processing with ternary classification.
//!
//! SensorReading, SensorFusion, AnomalyDetector, TimeSeries analysis,
//! and Calibration with ternary feedback.

use std::collections::HashMap;

/// Ternary classification for sensor readings.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum TernaryClass {
    Low = -1,
    Normal = 0,
    High = 1,
}

impl TernaryClass {
    pub fn from_i8(v: i8) -> Option<Self> {
        match v {
            -1 => Some(TernaryClass::Low),
            0 => Some(TernaryClass::Normal),
            1 => Some(TernaryClass::High),
            _ => None,
        }
    }

    pub fn to_i8(self) -> i8 {
        self as i8
    }
}

/// A single sensor reading with value and timestamp.
#[derive(Clone, Debug)]
pub struct SensorReading {
    pub sensor_id: String,
    pub value: f64,
    pub timestamp: u64,
    pub classification: TernaryClass,
}

impl SensorReading {
    pub fn new(sensor_id: &str, value: f64, timestamp: u64) -> Self {
        Self {
            sensor_id: sensor_id.to_string(),
            value,
            timestamp,
            classification: TernaryClass::Normal,
        }
    }

    /// Classify based on low/high thresholds.
    pub fn classify(&mut self, low: f64, high: f64) {
        self.classification = if self.value < low {
            TernaryClass::Low
        } else if self.value > high {
            TernaryClass::High
        } else {
            TernaryClass::Normal
        };
    }

    /// Classify based on mean and standard deviation (±1σ = Normal, etc.)
    pub fn classify_statistical(&mut self, mean: f64, std_dev: f64) {
        if std_dev <= 0.0 {
            self.classification = TernaryClass::Normal;
            return;
        }
        let z = (self.value - mean) / std_dev;
        self.classification = if z < -1.0 {
            TernaryClass::Low
        } else if z > 1.0 {
            TernaryClass::High
        } else {
            TernaryClass::Normal
        };
    }

    pub fn with_classification(mut self, c: TernaryClass) -> Self {
        self.classification = c;
        self
    }
}

/// Combines multiple sensor readings using various fusion strategies.
pub struct SensorFusion {
    weights: HashMap<String, f64>,
}

impl SensorFusion {
    pub fn new() -> Self {
        Self { weights: HashMap::new() }
    }

    pub fn set_weight(&mut self, sensor_id: &str, weight: f64) {
        self.weights.insert(sensor_id.to_string(), weight);
    }

    /// Weighted average of values.
    pub fn weighted_average(&self, readings: &[SensorReading]) -> f64 {
        let mut total_weight = 0.0;
        let mut weighted_sum = 0.0;
        for r in readings {
            let w = self.weights.get(&r.sensor_id).copied().unwrap_or(1.0);
            weighted_sum += r.value * w;
            total_weight += w;
        }
        if total_weight == 0.0 { 0.0 } else { weighted_sum / total_weight }
    }

    /// Majority vote on ternary classification.
    pub fn majority_vote(&self, readings: &[SensorReading]) -> TernaryClass {
        let mut counts: [usize; 3] = [0, 0, 0];
        for r in readings {
            match r.classification {
                TernaryClass::Low => counts[0] += 1,
                TernaryClass::Normal => counts[1] += 1,
                TernaryClass::High => counts[2] += 1,
            }
        }
        if counts[0] >= counts[1] && counts[0] >= counts[2] {
            TernaryClass::Low
        } else if counts[2] >= counts[0] && counts[2] >= counts[1] {
            TernaryClass::High
        } else {
            TernaryClass::Normal
        }
    }

    /// Weighted ternary vote.
    pub fn weighted_vote(&self, readings: &[SensorReading]) -> TernaryClass {
        let mut score = 0.0;
        for r in readings {
            let w = self.weights.get(&r.sensor_id).copied().unwrap_or(1.0);
            score += r.classification.to_i8() as f64 * w;
        }
        if score < -0.5 {
            TernaryClass::Low
        } else if score > 0.5 {
            TernaryClass::High
        } else {
            TernaryClass::Normal
        }
    }

    /// Min value across readings.
    pub fn min_value(&self, readings: &[SensorReading]) -> Option<f64> {
        readings.iter().map(|r| r.value).reduce(f64::min)
    }

    /// Max value across readings.
    pub fn max_value(&self, readings: &[SensorReading]) -> Option<f64> {
        readings.iter().map(|r| r.value).reduce(f64::max)
    }
}

/// Anomaly severity levels.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum AnomalyLevel {
    Normal = 0,
    Suspicious = 1,
    Alert = 2,
}

/// Detects anomalies with ternary classification.
pub struct AnomalyDetector {
    baseline_mean: f64,
    baseline_std: f64,
    suspicious_threshold: f64, // multiples of std
    alert_threshold: f64,
}

impl AnomalyDetector {
    pub fn new(mean: f64, std: f64) -> Self {
        Self {
            baseline_mean: mean,
            baseline_std: std,
            suspicious_threshold: 1.5,
            alert_threshold: 3.0,
        }
    }

    pub fn with_thresholds(mut self, suspicious: f64, alert: f64) -> Self {
        self.suspicious_threshold = suspicious;
        self.alert_threshold = alert;
        self
    }

    pub fn detect(&self, value: f64) -> AnomalyLevel {
        if self.baseline_std <= 0.0 {
            return AnomalyLevel::Normal;
        }
        let zscore = (value - self.baseline_mean).abs() / self.baseline_std;
        if zscore > self.alert_threshold {
            AnomalyLevel::Alert
        } else if zscore > self.suspicious_threshold {
            AnomalyLevel::Suspicious
        } else {
            AnomalyLevel::Normal
        }
    }

    /// Detect across a series, returning anomaly levels.
    pub fn detect_series(&self, values: &[f64]) -> Vec<AnomalyLevel> {
        values.iter().map(|&v| self.detect(v)).collect()
    }

    /// Update baseline with new data.
    pub fn update_baseline(&mut self, values: &[f64]) {
        if values.is_empty() { return; }
        let n = values.len() as f64;
        let mean = values.iter().sum::<f64>() / n;
        let variance = values.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / n;
        self.baseline_mean = mean;
        self.baseline_std = variance.sqrt().max(0.0);
    }
}

/// Time series analysis with ternary thresholds.
pub struct TimeSeries {
    data: Vec<(u64, f64)>, // (timestamp, value)
}

impl TimeSeries {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn add(&mut self, timestamp: u64, value: f64) {
        self.data.push((timestamp, value));
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn values(&self) -> Vec<f64> {
        self.data.iter().map(|&(_, v)| v).collect()
    }

    pub fn timestamps(&self) -> Vec<u64> {
        self.data.iter().map(|&(t, _)| t).collect()
    }

    /// Mean of values.
    pub fn mean(&self) -> f64 {
        if self.data.is_empty() { return 0.0; }
        self.data.iter().map(|&(_, v)| v).sum::<f64>() / self.data.len() as f64
    }

    /// Standard deviation.
    pub fn std_dev(&self) -> f64 {
        if self.data.len() < 2 { return 0.0; }
        let mean = self.mean();
        let variance = self.data.iter().map(|&(_, v)| (v - mean).powi(2)).sum::<f64>() / self.data.len() as f64;
        variance.sqrt()
    }

    /// Classify each point using ternary thresholds relative to mean ± k*std.
    pub fn ternary_classify(&self, k: f64) -> Vec<TernaryClass> {
        let mean = self.mean();
        let std = self.std_dev();
        self.data.iter().map(|&(_, v)| {
            if std <= 0.0 {
                TernaryClass::Normal
            } else if v < mean - k * std {
                TernaryClass::Low
            } else if v > mean + k * std {
                TernaryClass::High
            } else {
                TernaryClass::Normal
            }
        }).collect()
    }

    /// Moving average with window size.
    pub fn moving_average(&self, window: usize) -> Vec<f64> {
        if window == 0 || self.data.len() < window { return Vec::new(); }
        self.data.windows(window).map(|w| w.iter().map(|&(_, v)| v).sum::<f64>() / window as f64).collect()
    }

    /// Rate of change between consecutive points.
    pub fn rate_of_change(&self) -> Vec<f64> {
        self.data.windows(2).map(|w| w[1].1 - w[0].1).collect()
    }

    /// Classify rate of change as ternary.
    pub fn ternary_derivative(&self, threshold: f64) -> Vec<TernaryClass> {
        self.rate_of_change().iter().map(|&r| {
            if r < -threshold {
                TernaryClass::Low
            } else if r > threshold {
                TernaryClass::High
            } else {
                TernaryClass::Normal
            }
        }).collect()
    }
}

/// Calibration with ternary feedback.
pub struct Calibration {
    offset: f64,
    scale: f64,
    feedback_history: Vec<TernaryClass>,
}

impl Calibration {
    pub fn new() -> Self {
        Self {
            offset: 0.0,
            scale: 1.0,
            feedback_history: Vec::new(),
        }
    }

    pub fn with_params(offset: f64, scale: f64) -> Self {
        Self { offset, scale, feedback_history: Vec::new() }
    }

    /// Apply calibration to a raw reading.
    pub fn calibrate(&self, raw: f64) -> f64 {
        (raw + self.offset) * self.scale
    }

    /// Provide ternary feedback and adjust calibration.
    /// Low = reading too low, Normal = good, High = reading too high.
    pub fn feedback(&mut self, class: TernaryClass) {
        self.feedback_history.push(class);
        let adjustment = match class {
            TernaryClass::Low => 0.1,   // reading too low → increase offset
            TernaryClass::High => -0.1, // reading too high → decrease offset
            TernaryClass::Normal => 0.0,
        };
        self.offset += adjustment;
    }

    /// Batch feedback: adjust based on majority of history.
    pub fn auto_calibrate(&mut self) {
        if self.feedback_history.len() < 3 { return; }
        let low = self.feedback_history.iter().filter(|&&c| c == TernaryClass::Low).count();
        let high = self.feedback_history.iter().filter(|&&c| c == TernaryClass::High).count();
        let n = self.feedback_history.len();
        if low > n / 2 {
            self.offset += 0.05;
        } else if high > n / 2 {
            self.offset -= 0.05;
        }
    }

    pub fn offset(&self) -> f64 {
        self.offset
    }

    pub fn scale(&self) -> f64 {
        self.scale
    }

    pub fn feedback_history(&self) -> &[TernaryClass] {
        &self.feedback_history
    }

    /// Reset calibration parameters.
    pub fn reset(&mut self) {
        self.offset = 0.0;
        self.scale = 1.0;
        self.feedback_history.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ternary_class_values() {
        assert_eq!(TernaryClass::Low.to_i8(), -1);
        assert_eq!(TernaryClass::Normal.to_i8(), 0);
        assert_eq!(TernaryClass::High.to_i8(), 1);
    }

    #[test]
    fn test_sensor_reading_classify() {
        let mut r = SensorReading::new("temp", 15.0, 100);
        r.classify(20.0, 30.0);
        assert_eq!(r.classification, TernaryClass::Low);
    }

    #[test]
    fn test_sensor_reading_classify_high() {
        let mut r = SensorReading::new("temp", 35.0, 100);
        r.classify(20.0, 30.0);
        assert_eq!(r.classification, TernaryClass::High);
    }

    #[test]
    fn test_sensor_reading_classify_normal() {
        let mut r = SensorReading::new("temp", 25.0, 100);
        r.classify(20.0, 30.0);
        assert_eq!(r.classification, TernaryClass::Normal);
    }

    #[test]
    fn test_sensor_reading_statistical() {
        let mut r = SensorReading::new("temp", 100.0, 100);
        r.classify_statistical(50.0, 10.0);
        assert_eq!(r.classification, TernaryClass::High);
    }

    #[test]
    fn test_fusion_weighted_average() {
        let f = SensorFusion::new();
        let readings = vec![
            SensorReading::new("a", 10.0, 0),
            SensorReading::new("b", 20.0, 0),
        ];
        assert!((f.weighted_average(&readings) - 15.0).abs() < 1e-9);
    }

    #[test]
    fn test_fusion_weighted_with_weights() {
        let mut f = SensorFusion::new();
        f.set_weight("a", 2.0);
        f.set_weight("b", 1.0);
        let readings = vec![
            SensorReading::new("a", 10.0, 0),
            SensorReading::new("b", 20.0, 0),
        ];
        assert!((f.weighted_average(&readings) - 40.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_fusion_majority_vote() {
        let f = SensorFusion::new();
        let readings = vec![
            SensorReading::new("a", 0.0, 0).with_classification(TernaryClass::High),
            SensorReading::new("b", 0.0, 0).with_classification(TernaryClass::High),
            SensorReading::new("c", 0.0, 0).with_classification(TernaryClass::Low),
        ];
        assert_eq!(f.majority_vote(&readings), TernaryClass::High);
    }

    #[test]
    fn test_fusion_min_max() {
        let f = SensorFusion::new();
        let readings = vec![
            SensorReading::new("a", 5.0, 0),
            SensorReading::new("b", 15.0, 0),
        ];
        assert_eq!(f.min_value(&readings), Some(5.0));
        assert_eq!(f.max_value(&readings), Some(15.0));
    }

    #[test]
    fn test_anomaly_detector_normal() {
        let ad = AnomalyDetector::new(50.0, 10.0);
        assert_eq!(ad.detect(52.0), AnomalyLevel::Normal);
    }

    #[test]
    fn test_anomaly_detector_suspicious() {
        let ad = AnomalyDetector::new(50.0, 10.0);
        assert_eq!(ad.detect(75.0), AnomalyLevel::Suspicious);
    }

    #[test]
    fn test_anomaly_detector_alert() {
        let ad = AnomalyDetector::new(50.0, 10.0);
        assert_eq!(ad.detect(90.0), AnomalyLevel::Alert);
    }

    #[test]
    fn test_anomaly_detector_series() {
        let ad = AnomalyDetector::new(50.0, 10.0);
        let levels = ad.detect_series(&[50.0, 75.0, 90.0]);
        assert_eq!(levels[0], AnomalyLevel::Normal);
        assert_eq!(levels[1], AnomalyLevel::Suspicious);
        assert_eq!(levels[2], AnomalyLevel::Alert);
    }

    #[test]
    fn test_anomaly_detector_update_baseline() {
        let mut ad = AnomalyDetector::new(0.0, 1.0);
        ad.update_baseline(&[10.0, 10.0, 10.0, 10.0, 10.0]);
        // With std_dev ~0, everything is treated as Normal
        assert_eq!(ad.detect(10.0), AnomalyLevel::Normal);
        // Update with varied baseline
        ad.update_baseline(&[10.0, 12.0, 8.0, 11.0, 9.0]);
        // 10.0 should be normal (near mean)
        assert_eq!(ad.detect(10.0), AnomalyLevel::Normal);
        // Very far from mean should be suspicious or alert
        assert_ne!(ad.detect(100.0), AnomalyLevel::Normal);
    }

    #[test]
    fn test_time_series_mean_std() {
        let mut ts = TimeSeries::new();
        ts.add(0, 10.0);
        ts.add(1, 20.0);
        ts.add(2, 30.0);
        assert!((ts.mean() - 20.0).abs() < 1e-9);
        assert!(ts.std_dev() > 0.0);
    }

    #[test]
    fn test_time_series_ternary_classify() {
        let mut ts = TimeSeries::new();
        for i in 0..10 {
            ts.add(i, 50.0);
        }
        ts.add(10, 100.0);
        ts.add(11, 0.0);
        let classes = ts.ternary_classify(1.0);
        assert!(classes.iter().any(|&c| c == TernaryClass::High));
        assert!(classes.iter().any(|&c| c == TernaryClass::Low));
    }

    #[test]
    fn test_time_series_moving_average() {
        let mut ts = TimeSeries::new();
        ts.add(0, 10.0);
        ts.add(1, 20.0);
        ts.add(2, 30.0);
        let ma = ts.moving_average(2);
        assert_eq!(ma.len(), 2);
        assert!((ma[0] - 15.0).abs() < 1e-9);
    }

    #[test]
    fn test_time_series_rate_of_change() {
        let mut ts = TimeSeries::new();
        ts.add(0, 10.0);
        ts.add(1, 15.0);
        ts.add(2, 13.0);
        let roc = ts.rate_of_change();
        assert_eq!(roc.len(), 2);
        assert!((roc[0] - 5.0).abs() < 1e-9);
        assert!((roc[1] - (-2.0)).abs() < 1e-9);
    }

    #[test]
    fn test_time_series_ternary_derivative() {
        let mut ts = TimeSeries::new();
        ts.add(0, 10.0);
        ts.add(1, 20.0);
        ts.add(2, 10.0);
        let td = ts.ternary_derivative(3.0);
        assert_eq!(td[0], TernaryClass::High);
        assert_eq!(td[1], TernaryClass::Low);
    }

    #[test]
    fn test_calibration_basic() {
        let cal = Calibration::with_params(5.0, 2.0);
        assert!((cal.calibrate(10.0) - 30.0).abs() < 1e-9);
    }

    #[test]
    fn test_calibration_feedback_adjusts_offset() {
        let mut cal = Calibration::new();
        let initial = cal.offset();
        cal.feedback(TernaryClass::Low);
        assert!(cal.offset() > initial);
        cal.feedback(TernaryClass::High);
        assert!((cal.offset() - initial).abs() < 1e-9);
    }

    #[test]
    fn test_calibration_auto_calibrate() {
        let mut cal = Calibration::new();
        for _ in 0..5 {
            cal.feedback(TernaryClass::Low);
        }
        cal.auto_calibrate();
        assert!(cal.offset() > 0.0);
    }

    #[test]
    fn test_calibration_reset() {
        let mut cal = Calibration::with_params(5.0, 2.0);
        cal.feedback(TernaryClass::Low);
        cal.reset();
        assert!((cal.offset()).abs() < 1e-9);
        assert!(cal.feedback_history().is_empty());
    }
}
