# Cost Matrix

This document details the **Cost Matrix** used by the KeyForge physics engine. The cost matrix defines the penalty (in arbitrary units, roughly ms) for moving a finger from its home position to any other key.

## Row Staggered

This section details the costs for **Row-Staggered** keyboards (standard ANSI/ISO layouts).

### Overview

* **Generic Selectors**: Costs are assigned based on physics attributes: `hand` (left/right), `finger` (thumb, index, middle, ring, pinky), `row` (0-3), and relative `col` offset.
* **Home Position Assumption**: Transition costs assume the motion starts from the finger's resting **Home Key**.
* **Cross-Hand Bonus**: Switching hands incurs a flat, low cost (`80.0`), encouraging hand alternation.
* **Repeats**: Pressing the same key twice (double-tap) has a specific repeat cost.
* **Thumb**: Thumbs are treated uniqely; normally they only press the thumb key (`140.0` repeat/self cost). Cross-finger reaches to the thumb are undefined/prohibited.

---

### Cost Grid (Home Row Transitions)

The following tables show the cost for a finger resting on its **Home Key** (Row 2) to reach a target key.

**Legend**:

* **Row 0**: Number Row
* **Row 1**: Top Row (QWERTY)
* **Row 2**: Home Row (ASDF)
* **Row 3**: Bottom Row (ZXCV)

#### Combined Grid

| Row | L Pky (Ext) | L Pinky (Out) | L Pinky | L Ring | L Middle | L Index | L Index (In) | R Index (In) | R Index | R Middle | R Ring | R Pinky | R Pinky (Out) | R Pky (Ext1) | R Pky (Ext2) | R Pky (Ext3) |
| :--- | :-- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :-- | :-- | :-- |
| **0 (Num)** | - | 334.54 | 334.54 | 321.54 | 308.54 | 302.04 | 300.23 | 309.86 | 302.04 | 308.54 | 321.54 | 334.54 | 334.54 | - | - | 365.00 |
| **1 (Top)** | 217.26 | 216.74 | 202.26 | 181.68 | 162.26 | 270.46 | 273.75 | 279.01 | 270.46 | 157.00 | 174.98 | 195.14 | 209.47 | 317.73 | 331.39 | 345.00 |
| **2 (Home)** | 177.50 | 334.54 | 162.50 | 149.50 | 136.50 | 130.00 | 265.00 | 265.00 | 130.00 | 136.50 | 149.50 | 162.50 | 297.50 | - | - | 312.50 |
| **3 (Bot)** | 324.27 | 309.27 | 309.27 | 296.27 | 283.27 | 276.77 | 287.04 | 287.04 | 276.77 | 283.27 | 296.27 | 309.27 | 309.27 | - | - | - |
| **4 (Mod)** | 350.00 | - | - | - | - | - | - | - | - | - | - | - | - | 350.00 | - | - |

*Note: Asymmetries exist in **Row 0** and **Row 1** due to the left-slant geometry of standard keyboards.*

#### Thumb Costs

* **Self-Press (Thumb -> Thumb)**: `140.0`
* **Cross-Finger Reaches**: None (Removed to enforce physical realism).

---

### Transition Rules

#### 1. Same Hand Transitions

Costs are defined for moving from **Home Key (Row 2)** -> **Target Key**.

* *Example*: Right Index Home (J) -> Right Index Top (U) cost is `270.46`.
* Direct transitions between non-home rows (e.g., Top -> Bottom) are **not** defined; the model assumes the hand returns toward a neutral position or calculates cost relative to home.

#### 2. Cross-Hand Transitions (Alternation)

Any transition from **Left Hand** -> **Right Hand** (or vice versa) has a flat cost.

* **Cost**: `80.0`

#### 3. Same-Key Repeats (Double Taps)

Cost to press a key again immediately from itself.

* **Home Keys**: ~`130.0` - `162.5` (See Grid Row 2)
* **Top Row Keys**: `140.0`
* **Thumb Key**: `140.0`
* **Bottom Row Keys**: varies (e.g., Right Pinky `182.5`)

---

### Asymmetries Observed

The **row-staggered** nature causes cost differences between hands:

1. **Row 1 (Top)**: Left hand costs are generally **higher** than Right hand for corresponding fingers (e.g., Pinky `202.26` vs `195.14`).
    * *Exception*: Left Index Inner (`273.75`) is cheaper than Right Index Inner (`279.01`).
2. **Row 0 (Number)**: Left Index Inner (`300.23`) is cheaper than Right Index Inner (`309.86`).
3. **Home & Bottom Rows**: Perfectly symmetric.
