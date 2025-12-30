# Configuration Schema

**Version:** 4.0
**Context:** Tunable parameters for the Physics and Evolution engines.

## 1. SearchConfig (Simulated Annealing)

The `SearchConfig` struct controls the behavior of the optimizer.

| Parameter | Type | Default | Description | Constraints |
| :--- | :--- | :--- | :--- | :--- |
| **`steps`** | `usize` | `100,000` | The maximum number of mutations to attempt. | Must be `> 0`. |
| **`start_temp`** | `f32` | `100.0` | Initial temperature. Higher values allow more chaotic jumps early on. | Must be `>= 0`. |
| **`end_temp`** | `f32` | `0.01` | Final temperature target. Determines the decay rate. | Must be `>= 0`. |
| **`seed`** | `u64` | `42` | The PRNG seed. Ensures deterministic runs for debugging. | None. |
| **`patience`** | `usize` | `500` | Number of steps without improvement before triggering a Reheat. | Must be `> 0`. |
| **`reheats`** | `usize` | `3` | How many times to spike the temperature back up if stuck in a local minimum. | Must be `>= 0`. |
| **`reheat_factor`** | `f32` | `0.5` | Multiplier for `start_temp` when reheating (e.g., 0.5 * 100.0 = 50.0). | Must be `> 0`. |

## 2. Rubric (Scoring Weights)

The `Rubric` defines the "Personality" of the layout (e.g., Comfort vs Speed).

| Parameter | Type | Description |
| :--- | :--- | :--- |
| **`sfb_weight`** | `f32` | Penalty for using the same finger twice in a row. |
| **`distance_weight`** | `f32` | Penalty for finger travel distance. |
| **`roll_weight`** | `f32` | Bonus (negative cost) for inward rolling motions. |
| **`redirect_weight`** | `f32` | Penalty for changing direction mid-stream (e.g., Left -> Right -> Left). |
