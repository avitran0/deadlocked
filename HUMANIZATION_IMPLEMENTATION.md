# Humanization Implementation Plan

## Overview
Implementation plan for advanced aimbot humanization features: Overcorrection/Undercorrection and Adaptive Muscle Memory using self-learning algorithms.

---

## Feature 1: Overcorrection/Undercorrection

### Objective
Mimic human imperfect hand-eye coordination by sometimes aiming past or short of the target, then correcting.

### State Additions

#### Add to `Target` struct (`src/cs2/target.rs`):
```rust
pub correction_state: CorrectionState,
pub correction_offset: Vec2,
pub correction_phase: u8,
pub correction_target_phase: u8,
```

#### New Enum (`src/cs2/target.rs`):
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CorrectionState {
    None,
    Overcorrecting,
    Undercorrecting,
}

impl Default for CorrectionState {
    fn default() -> Self {
        Self::None
    }
}
```

### Algorithm Flow

#### Step 1: Trigger Detection (in `humanize_aim()`)
```rust
// Check if starting new aim sequence (large movement distance)
if movement_distance > 50.0 && self.target.correction_state == CorrectionState::None {
    let correction_chance = 0.12 * humanization_amount; // 12% base chance
    if rng.random_range(0.0..1.0) < correction_chance {
        // Decide type
        if rng.random_range(0.0..1.0) < 0.6 {
            self.target.correction_state = CorrectionState::Overcorrecting;
            // Overshoot by 5-15%
            let overshoot = rng.random_range(0.05..0.15);
            self.target.correction_offset = vec2(target_x * overshoot, target_y * overshoot);
            self.target.correction_target_phase = rng.random_range(2..5) as u8;
        } else {
            self.target.correction_state = CorrectionState::Undercorrecting;
            // Undershoot by 10-20%
            let undershoot = rng.random_range(0.10..0.20);
            self.target.correction_offset = vec2(target_x * undershoot, target_y * undershoot);
            self.target.correction_target_phase = rng.random_range(3..7) as u8;
        }
        self.target.correction_phase = 0;
    }
}
```

#### Step 2: Apply Correction (in `humanize_aim()`)
```rust
// Apply correction offset based on state
let (corrected_x, corrected_y) = match self.target.correction_state {
    CorrectionState::Overcorrecting => {
        // Add extra movement initially
        if self.target.correction_phase == 0 {
            (humanized_x + self.target.correction_offset.x,
             humanized_y + self.target.correction_offset.y)
        } else {
            // Gradually remove offset
            let decay = self.target.correction_phase as f32 / self.target.correction_target_phase as f32;
            let remaining = 1.0 - decay;
            (humanized_x - self.target.correction_offset.x * remaining * 0.3,
             humanized_y - self.target.correction_offset.y * remaining * 0.3)
        }
    },
    CorrectionState::Undercorrecting => {
        // Reduce initial movement
        if self.target.correction_phase == 0 {
            (humanized_x - self.target.correction_offset.x,
             humanized_y - self.target.correction_offset.y)
        } else {
            // Gradually add remaining movement
            let progress = self.target.correction_phase as f32 / self.target.correction_target_phase as f32;
            (humanized_x + self.target.correction_offset.x * progress * 0.4,
             humanized_y + self.target.correction_offset.y * progress * 0.4)
        }
    },
    CorrectionState::None => (humanized_x, humanized_y),
};
```

#### Step 3: Phase Management
```rust
// Update correction phase
if self.target.correction_state != CorrectionState::None {
    self.target.correction_phase += 1;
    if self.target.correction_phase >= self.target.correction_target_phase {
        self.target.correction_state = CorrectionState::None;
        self.target.correction_phase = 0;
        self.target.correction_offset = Vec2::ZERO;
    }
}
```

#### Step 4: Return corrected values
```rust
vec2(corrected_x, corrected_y)
```

---

## Feature 2: Adaptive Muscle Memory (Self-Learning)

### Objective
Algorithm learns aim patterns during gameplay and adapts parameters to be faster/smoother at frequently aimed angles. No databases - pure runtime learning.

### State Additions

#### Add to `Target` struct (`src/cs2/target.rs`):
```rust
use std::collections::VecDeque;

pub aim_history: VecDeque<AimRecord>,
pub learned_smooth_bias: f32,
pub learned_jitter_reduction: f32,
pub confidence_map: [f32; 24], // 24 angle zones (360°/15° = 24)
pub frames_since_analysis: u32,
```

#### New Struct (`src/cs2/target.rs`):
```rust
#[derive(Debug, Clone)]
pub struct AimRecord {
    pub angle_zone: usize,  // 0-23 (360° / 15°)
    pub adjustment_magnitude: f32,
    pub timestamp: Instant,
}
```

#### Update Default Implementation:
```rust
impl Default for Target {
    fn default() -> Self {
        Self {
            // ... existing fields ...
            aim_history: VecDeque::with_capacity(50),
            learned_smooth_bias: 0.0,
            learned_jitter_reduction: 0.0,
            confidence_map: [0.0; 24],
            frames_since_analysis: 0,
        }
    }
}
```

### Algorithm Implementation

#### Step 1: Record Aim Data (in `humanize_aim()`)
```rust
// Convert target angle to zone (0-23)
let angle_degrees = target_y.atan2(target_x).to_degrees();
let normalized_angle = if angle_degrees < 0.0 { angle_degrees + 360.0 } else { angle_degrees };
let angle_zone = (normalized_angle / 15.0).floor() as usize % 24;

// Calculate adjustment magnitude
let adjustment_magnitude = (target_x * target_x + target_y * target_y).sqrt();

// Record to history
let record = AimRecord {
    angle_zone,
    adjustment_magnitude,
    timestamp: Instant::now(),
};

self.target.aim_history.push_back(record);

// Limit history size
if self.target.aim_history.len() > 50 {
    self.target.aim_history.pop_front();
}

// Clean old records (older than 30 seconds)
let now = Instant::now();
self.target.aim_history.retain(|r| now.duration_since(r.timestamp).as_secs() < 30);
```

#### Step 2: Pattern Analysis (every 100 frames)
```rust
self.target.frames_since_analysis += 1;

if self.target.frames_since_analysis >= 100 {
    self.target.frames_since_analysis = 0;
    
    // Count frequency for each angle zone
    let mut zone_counts = [0u32; 24];
    for record in &self.target.aim_history {
        zone_counts[record.angle_zone] += 1;
    }
    
    // Update confidence for each zone
    for zone in 0..24 {
        let frequency = zone_counts[zone] as f32;
        
        if frequency >= 5.0 {
            // Increase confidence for frequently used angles
            self.target.confidence_map[zone] += 0.05;
            self.target.confidence_map[zone] = self.target.confidence_map[zone].min(1.0);
        } else {
            // Decay confidence for unused angles
            self.target.confidence_map[zone] *= 0.98;
        }
    }
    
    // Calculate overall learned biases
    let avg_confidence: f32 = self.target.confidence_map.iter().sum::<f32>() / 24.0;
    self.target.learned_smooth_bias = (avg_confidence * 0.25).min(0.25);
    self.target.learned_jitter_reduction = (avg_confidence * 0.35).min(0.35);
}
```

#### Step 3: Apply Learning (in `humanize_aim()`)
```rust
// Get familiarity for current angle zone
let familiarity = self.target.confidence_map[angle_zone];

// Apply learned adjustments
let familiarity_smooth = 1.0 + (self.target.learned_smooth_bias * familiarity);
let familiarity_jitter = 1.0 - (self.target.learned_jitter_reduction * familiarity);

// Modify humanization parameters
let micro_jitter_x = micro_jitter_x * familiarity_jitter;
let micro_jitter_y = micro_jitter_y * familiarity_jitter;
let jitter_x = jitter_x * familiarity_jitter;
let jitter_y = jitter_y * familiarity_jitter;
let smooth_factor = smooth_factor * familiarity_smooth;
```

### Learning Characteristics

**Warm-up Period (0-2 minutes):**
- All angles unfamiliar (confidence = 0)
- Full humanization applied
- Baseline performance

**Learning Phase (2-10 minutes):**
- Frequently aimed angles gain confidence
- Gradual improvement at common spots
- Up to 25% faster smooth, 35% less jitter

**Steady State (10+ minutes):**
- Stable confidence levels
- Natural "muscle memory" effect
- Unused angles decay back to baseline

**Session Reset:**
- Memory clears on restart
- Each session develops unique patterns
- No persistent storage = no fingerprinting

---

## Integration Points

### File: `src/cs2/target.rs`
1. Add new fields to `Target` struct
2. Add `CorrectionState` enum
3. Add `AimRecord` struct
4. Update `Default` implementation
5. Add `use std::collections::VecDeque;`

### File: `src/cs2/aimbot.rs`
1. Import new types: `use super::target::{CorrectionState, AimRecord};`
2. Add overcorrection logic to `humanize_aim()`
3. Add muscle memory recording to `humanize_aim()`
4. Add pattern analysis (every 100 frames)
5. Apply learned parameters to humanization calculations

---

## Testing Strategy

### Overcorrection Testing:
1. Set humanization to 5.0
2. Make quick 180° turns
3. Observe ~12% of movements overshoot slightly
4. Verify correction happens over 2-4 frames

### Muscle Memory Testing:
1. Play for 5 minutes, repeatedly aiming at same spot
2. Check `confidence_map` for increased values
3. Verify reduced jitter on familiar angles
4. Test that unused angles decay over time

---

## Performance Impact

**Memory:**
- VecDeque (50 records): ~2KB
- Confidence map (24 floats): 96 bytes
- Total overhead: ~2.5KB

**CPU:**
- Recording: O(1) per frame
- Analysis: O(50) every 100 frames
- Negligible impact (<0.1ms)

---

## Detection Resistance

**Overcorrection Benefits:**
- ✅ Breaks perfect aim paths
- ✅ Mimics human imprecision
- ✅ Variable correction timing
- ✅ Natural looking adjustments

**Muscle Memory Benefits:**
- ✅ No fixed patterns (learns per session)
- ✅ Organic performance improvement
- ✅ Creates unique "playstyle"
- ✅ Realistic skill curve
- ✅ No database = no fingerprinting

---

## Implementation Order

1. **Phase 1:** Overcorrection/Undercorrection (1-2 hours)
   - Simpler feature
   - Immediate visual impact
   - Test and validate

2. **Phase 2:** Muscle Memory Foundation (2-3 hours)
   - Add data structures
   - Implement recording
   - Test history management

3. **Phase 3:** Learning Algorithm (2-3 hours)
   - Pattern analysis
   - Confidence calculation
   - Parameter application

4. **Phase 4:** Testing & Tuning (1-2 hours)
   - Balance learning rates
   - Verify decay mechanics
   - Performance optimization

**Total Estimated Time:** 6-10 hours
