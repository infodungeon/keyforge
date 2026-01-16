# Cost Matrix: Physics & Biomechanics

This document details the **Baseline Cost Matrix** used by the KeyForge physics engine. It provides two discrete models to account for physical hardware differences:

1. **Model A:** Standard Row-Staggered (ANSI/ISO).
2. **Model B:** Columnar/Ortholinear (Ergodox, Planck, Corne).

## 1. Scientific Basis & Methodology

The engine utilizes a quantitative approach based on **Fitts’ Law** and **Biomechanical Penalty Coefficients** derived from ergonomic research (Carpalx, Workman, Colemak-DH).

The cost ($C$) for any given key transition is calculated as:
$$ C = \text{Base} + (\text{ID} \times W_{dist}) + P_{row} + P_{finger} + P_{lateral} $$

* **Base (100):** Calibrated to the Index Finger on the Home Row.
* **Fitts' Index of Difficulty (ID):** $\log_2(D/W + 1)$, quantifying the time to reach a target based on distance ($D$) and key width ($W$).
* **$P_{finger}$:** Strength penalties (Index=0, Middle=5, Ring=20, Pinky=45).
* **$P_{row}$:** Row penalties (Home=0, Top=20, Bottom=30, Number=50).

---

## 2. Model A: Row-Staggered

*Hardware: Standard ANSI/ISO Keyboards.*

This model accounts for the physical asymmetry of standard keyboards (e.g., the left-slant stagger and the extreme reach of the Left Index to the central column).

### 2.1 Static Cost Grid (Home Row Transitions)

**Unit Scale:** 100 = Baseline effort.

| Row | L Pky (Ext) | L Pky (Out) | L Pinky | L Ring | L Middle | L Index | L Index (In) | R Index (In) | R Index | R Middle | R Ring | R Pinky | R Pinky (Out) | R Pky (Ext1) | R Pky (Ext2) |
| :--- | :--: | :--: | :--: | :--: | :--: | :--: | :--: | :--: | :--: | :--: | :--: | :--: | :--: | :--: | :--: |
| **0 (Num)** | **281** | **244** | **244** | **219** | **204** | **199** | **198** | **226** | **199** | **204** | **219** | **244** | **243** | **247** | **289** |
| **1 (Top)** | **210** | **210** | **196** | **171** | **156** | **151** | **155** | **181** | **151** | **156** | **171** | **196** | **200** | **243** | **257** |
| **2 (Home)** | **182** | **154** | **145** | **120** | **105** | **100** | **150** | **150** | **100** | **105** | **120** | **145** | **175** | **199** | **231** |
| **3 (Bot)** | **215** | **207** | **207** | **182** | **167** | **162** | **195** | **172*** | **172*** | **177*** | **192*** | **217*** | **260** | - | - |
| **4 (Mod)** | **239** | **145** | **125** | - | - | - | - | - | - | - | **125** | **145** | **165** | **239** | - |

**Model A Specific Adjustments:**

* **Asymmetric Index Reach:** `L Index (In)` at Row 3 (the 'B' key) is **195**, significantly higher than `R Index (In)` at Row 3 (the 'N' key) at **172**. This accounts for the 1.5u vs 0.5u lateral distance difference caused by stagger.
* **Angle Mod (Right Bot):** Right-hand bottom row costs (*marked \**) are **+10** higher than left-hand equivalents due to ulnar deviation caused by the stagger.
* **Modifiers (Row 4):** Keys like `Alt`, `Win`, and `Ctrl` are modeled as **Hand Shifts** or **Pinky reaches**, not Thumb keys.
* **Menu Key:** Calculated at **165** (Linear lateral penalty).

---

## 3. Model B: Ortho & Columnar

*Hardware: Ergodox, Corne, Planck, Moonlander.*

This model assumes a "Physical Grid" or "Vertical Stagger" where vertical motion is strictly linear.

*Note: While Grid (Planck) and Columnar (Corne) have different vertical offsets for the Pinky, this model assumes the user adjusts their hand angle to compensate, normalizing the cost difference.*

### 3.1 Static Cost Grid

*Note: Due to grid symmetry, Left and Right hands share identical costs.*

| Row | Pinky (Out) | Pinky | Ring | Middle | Index | Index (Inner) |
| :--- | :--: | :--: | :--: | :--: | :--: | :--: |
| **Num** | **244** | **243** | **218** | **203** | **198** | **241** |
| **Top** | **210** | **195** | **170** | **155** | **150** | **198** |
| **Home** | **154** | **145** | **120** | **105** | **100** | **150** |
| **Bottom** | **207** | **205** | **180** | **165** | **160** | **208** |

### 3.2 Thumb Cluster Grid

*Note: Model B assumes a dedicated Thumb Cluster. Thumbs are treated as the strongest digit (Base Cost 0), with penalties only for distance/reach.*

| Key Position | Cost | Description |
| :--- | :--: | :--- |
| **Primary (Home)** | **100** | Resting position (e.g., Space/Bksp). Comparable to Index Home. |
| **Secondary (Ext)** | **152** | Slight extension (e.g., Tab/Enter). Comparable to Top Row reach. |
| **Tertiary (Tuck)** | **162** | Curling under palm (e.g., Alt/Layer). Comparable to Bottom Row. |

---

## 4. Dynamic Rules & Sequence Modifiers

*These rules apply to the physics engine universally across both models.*

### 4.1 Sequence Flow (Rolls & Redirects)

* **Inward Rolls (Bonus: -30%):** Outer $\to$ Inner finger (e.g., Pinky $\to$ Index). Biomechanically superior.
* **Outward Rolls (Neutral):** Inner $\to$ Outer finger.
* **Skipgrams / Disjoint Rolls (Bonus: -15%):** A sequence involving non-adjacent fingers (e.g., Index $\to$ Ring $\to$ Middle). While technically a direction change, the unused middle finger reduces the "redirect" strain.
* **Redirects (Penalty: +50%):** A jagged sequence changing direction on adjacent/dependent finger
