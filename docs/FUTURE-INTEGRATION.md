# Future Integration: ternary-sensor

## Current State
Provides `SensorReading` with ternary classification (Low/Normal/High), `SensorFusion` with weighted averaging and majority voting, `AnomalyDetector` with threshold and statistical classification, time-series analysis, and sensor calibration with ternary feedback.

## Integration Opportunities

### With ternary-cell
Sensors are the input layer of the ternary-cell tick cycle. `SensorReading::classify_statistical()` converts raw analog readings into `TernaryClass` — exactly the input format for `TernaryCell::tick()`. The `SensorFusion::majority_vote()` method resolves conflicts between multiple sensors monitoring the same room parameter (e.g., three temperature sensors disagreeing). This becomes the cell's perception phase before `surprise` computation.

### With ternary-control
Sensor readings feed directly into `PidController::compute_ternary()`. The sensor classification (Low/Normal/High) becomes the process variable for the controller. `Calibration` ensures sensor drift doesn't corrupt the control loop over time. The ternary output of the controller then feeds back as actuator commands, closing the loop: sense → classify → control → act → sense.

### With ternary-bayesian
`AnomalyDetector`'s threshold classification becomes the likelihood function for Bayesian updates. When a sensor reads anomalous, `TernaryDist::posterior()` updates the belief about the room's true state, accounting for sensor noise. This creates a probabilistic sensor fusion layer that's more robust than majority vote.

## Potential in Mature Systems
In room-as-codespace, each physical room has multiple sensors feeding into a ternary-cell grid. `SensorFusion` runs at Layer 0 (ESP32) — the microcontroller reads sensors, classifies them as ternary, and sends a single trit per parameter to Layer 1. The `Calibration` struct maintains per-sensor baseline offsets that drift-compensate over time. At Layer 2, `AnomalyDetector` flags persistent anomalies for escalation.

## Cross-Pollination Ideas
**Music × Sensors:** Audio sensors in a room produce ternary classifications (quiet/normal/loud) that feed into `agent-rhythm-rs`. The rhythm detection agent uses ternary acoustic patterns to identify room occupancy and activity patterns. A ternary rhythm signature becomes a room fingerprint.

**Evolution × Sensors:** `evolution-ternary` could evolve optimal sensor placement. Each individual in the population encodes sensor positions; fitness is information-theoretic coverage of the room.

## Dependencies for Next Steps
- `ternary-cell` must accept `SensorReading` as tick input
- Hardware abstraction: define a `TernarySensor` trait for ESP32 GPIO reads
- Real-time calibration loop integration with `ternary-control`
