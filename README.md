# ternary-sensor

Sensor data processing with ternary classification — threshold and statistical classification, multi-sensor fusion, anomaly detection, time series analysis, and calibration with ternary feedback.

## Why This Exists

Sensor data is noisy, multi-source, and needs fast decisions. In most monitoring systems you don't need the exact temperature — you need to know if it's below normal, normal, or above normal. Similarly, anomaly detection rarely needs a continuous risk score — you need normal / suspicious / alert.

**ternary-sensor** classifies every sensor reading into one of three states (`Low`, `Normal`, `High`) using either fixed thresholds or statistical z-scores. It then provides tools to fuse multiple ternary readings (majority vote, weighted vote, weighted average), detect anomalies against a learned baseline, analyze time series with ternary derivatives, and auto-calibrate sensors using ternary feedback.

## Core Concepts

| Type | Meaning |
|---|---|
| `TernaryClass` | Classification: `Low` (-1), `Normal` (0), `High` (+1) |
| `SensorReading` | A single reading with value, timestamp, and classification |
| `SensorFusion` | Combine multiple readings: weighted average, majority vote, weighted vote |
| `AnomalyDetector` | Z-score based detection with Normal/Suspicious/Alert levels |
| `TimeSeries` | Timestamped series with ternary classification and rate-of-change |
| `Calibration` | Offset/scale calibration with ternary feedback loop |

## Quick Start

```toml
# Cargo.toml
[dependencies]
ternary-sensor = "0.1"
```

```rust
use ternary_sensor::*;

fn main() {
    // Classify a reading with fixed thresholds
    let mut reading = SensorReading::new("temperature", 35.0, 1000);
    reading.classify(20.0, 30.0);
    println!("Classification: {:?}", reading.classification); // High

    // Classify statistically (z-score)
    let mut reading2 = SensorReading::new("cpu_load", 95.0, 1001);
    reading2.classify_statistical(50.0, 15.0);
    println!("Classification: {:?}", reading2.classification); // High (z > 3)

    // Fuse multiple sensors
    let fusion = SensorFusion::new();
    let readings = vec![
        SensorReading::new("s1", 22.0, 0).with_classification(TernaryClass::Normal),
        SensorReading::new("s2", 23.0, 0).with_classification(TernaryClass::Normal),
        SensorReading::new("s3", 35.0, 0).with_classification(TernaryClass::High),
    ];
    let consensus = fusion.majority_vote(&readings);
    println!("Majority: {:?}", consensus); // Normal

    // Anomaly detection
    let detector = AnomalyDetector::new(50.0, 10.0).with_thresholds(1.5, 3.0);
    println!("{:?}", detector.detect(52.0));  // Normal
    println!("{:?}", detector.detect(75.0));  // Suspicious
    println!("{:?}", detector.detect(90.0));  // Alert
}
```

## API Overview

### SensorReading
- `new(sensor_id, value, timestamp)` — create raw reading
- `classify(low, high)` — threshold-based ternary classification
- `classify_statistical(mean, std_dev)` — z-score based (±1σ)
- `with_classification(class)` — builder pattern

### SensorFusion
- `weighted_average(readings) → f64` — combine values with sensor weights
- `majority_vote(readings) → TernaryClass` — plurality voting
- `weighted_vote(readings) → TernaryClass` — weighted ternary vote
- `set_weight(sensor_id, weight)` — configure per-sensor trust
- `min_value(readings)`, `max_value(readings)` — range queries

### AnomalyDetector
- `new(mean, std_dev)` — create with baseline statistics
- `with_thresholds(suspicious, alert)` — configure z-score thresholds
- `detect(value) → AnomalyLevel` — Normal / Suspicious / Alert
- `detect_series(values) → Vec<AnomalyLevel>` — batch detection
- `update_baseline(values)` — retrain from new data

### TimeSeries
- `add(timestamp, value)` — append data points
- `mean()`, `std_dev()` — basic statistics
- `ternary_classify(k) → Vec<TernaryClass>` — classify each point (mean ± k*σ)
- `moving_average(window) → Vec<f64>` — smoothed series
- `rate_of_change() → Vec<f64>` — first differences
- `ternary_derivative(threshold) → Vec<TernaryClass>` — classify rate of change

### Calibration
- `new()` / `with_params(offset, scale)` — create calibrator
- `calibrate(raw) → f64` — apply offset and scale
- `feedback(class)` — provide ternary feedback, auto-adjust offset
- `auto_calibrate()` — batch adjustment from feedback history
- `reset()` — clear calibration and history

## How It Works

**Classification** maps a continuous sensor value to one of three states. Threshold classification uses fixed bounds: values below `low` are `Low`, above `high` are `High`, and between are `Normal`. Statistical classification computes the z-score `(value - mean) / std_dev` and uses ±1σ boundaries — roughly 68% of normally distributed values fall in `Normal`, making the tails natural anomaly indicators.

**Fusion** aggregates multiple sensor readings. Weighted average computes `Σ(wᵢ × vᵢ) / Σwᵢ`. Majority vote counts each ternary class and returns the plurality. Weighted vote computes `Σ(wᵢ × classᵢ)` and thresholds at ±0.5. This allows high-trust sensors to dominate the consensus.

**Anomaly detection** uses z-scores against a learned baseline. Each reading's absolute z-score is compared against two thresholds: suspicious (default 1.5σ) and alert (default 3.0σ). The baseline can be updated online as new data arrives, adapting to concept drift.

**Time series analysis** provides ternary derivatives: the rate of change between consecutive points is classified as `Low` (falling fast), `Normal` (stable), or `High` (rising fast) based on a configurable threshold. This gives a compact three-valued trend signal suitable for downstream ternary systems.

**Calibration** maintains an offset and scale. When feedback indicates the reading is `Low` (too low), offset increases; `High` feedback decreases it. Batch auto-calibration adjusts based on the majority of accumulated feedback, creating a self-correcting sensor loop.

## Use Cases

- **Environmental monitoring** — classify temperature, humidity, and air quality from multiple sensors, fuse via majority vote, and trigger alerts on anomalies
- **Industrial IoT** — statistical anomaly detection on vibration, pressure, and current sensors with adaptive baselines and ternary alert levels
- **Robotics** — calibrate IMU or distance sensors using ternary feedback (reading too low / correct / too high) with auto-calibration

## Ecosystem

Part of the **SuperInstance** ternary computing ecosystem:

- [`ternary`](https://crates.io/crates/ternary) — core trit types and balanced ternary arithmetic
- [`ternary-sensor`](https://crates.io/crates/ternary-sensor) — this crate
- [`ternary-kalman`](https://crates.io/crates/ternary-kalman) — Kalman filtering for ternary states
- [`ternary-control`](https://crates.io/crates/ternary-control) — ternary PID and bang-bang controllers
- [`ternary-fuzzy`](https://crates.io/crates/ternary-fuzzy) — fuzzy logic with ternary membership
- [`ternary-anomaly`](https://crates.io/crates/ternary-anomaly) — ternary anomaly detection

## License

MIT
