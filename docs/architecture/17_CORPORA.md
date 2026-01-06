# Corpora & Data Processing

This document details the acquisition, cleansing, and validation strategies for the text corpora used to generate frequency statistics (N-grams and words) for Keyforge.

## 1. Data Cleansing Philosophy

The primary goal of the Keyforge data pipeline is to model **standard English typing behavior** on a standard ANSI keyboard. The cleansing strategy prioritizes data purity over data recovery.

### Core Principles

1. **Strict Whitelist:** Only standard ASCII characters (Decimal 32-126) are permitted. Any character outside this range (e.g., Unicode, Diacritics, Control Codes) is considered "Foreign" or "Noise."
2. **Taint Propagation:** If a word buffer contains even a *single* invalid character (e.g., "café", "naïve", or binary garbage), the **entire word is discarded**. We do not attempt to "salvage" the valid parts, as this creates partial non-words.
3. **Flow Interruption:** When a word is tainted and discarded, the N-gram statistical chain is **reset**. The preceding word is not stitched to the following word, preventing the generation of phantom bigrams that never occurred in the source text.
4. **Ground Truth Verification:** Validity is not determined by heuristics alone but by cross-referencing the output against a standard English dictionary.

---

## 2. Corpus: `en_std` (Modern English Prose)

The `en_std` corpus represents Standard Modern English with a focus on creative writing, dialogue, and narrative flow.

### 2.1 Source Data

* **Dataset Name:** `lucadiliello/bookcorpusopen` (Hugging Face)
* **Description:** An open replication of the original BookCorpus dataset.
* **Volume:** ~6 Billion characters.
* **Format:** Parquet (Columnar).

### 2.2 Processing Pipeline

The raw data undergoes a parallel, zero-copy streaming transformation using a custom Rust tokenizer.

#### Step 1: Normalization

Before validation, specific characters are mapped to their standard ASCII equivalents.

| Category | Source Character(s) | Mapped To |
| :--- | :--- | :--- |
| **Quotes** | `“` `”` `„` | `"` |
| **Apostrophes** | `‘` `’` `´` | `'` |
| **Dashes** | `–` `—` `―` | `-` |
| **Ligatures** | `ﬁ` `ﬂ` | `f` |
| **Artifacts** | `\u00ad` `\` `_` | `Space` |

#### Step 2: Taint Detection (The Gatekeeper)

As characters are read into the buffer, they are checked against the **Strict Whitelist** (ASCII 32-126).

* **Valid Char:** Added to buffer.
* **Invalid Char (e.g., `é`, `ñ`, `α`):** The current buffer is marked as **TAINTED**. The processor continues reading until the next separator (Space/Newline) to clear the bad token, but **no stats are recorded** for that token.
* **Effect:** Words like `naïve`, `façade`, or garbage binary strings are completely dropped. The N-gram context is reset.

#### Step 3: Tokenization Strategy

Once a valid (untainted) buffer is flushed (by space or punctuation), it undergoes structural processing:

1. **Double Dash Expansion:** The sequence `--` is explicitly treated as a separator.
   * Input: `today--actually`
   * Output: `today`, `actually`
2. **Multi-Hyphen Splitting:** Words containing more than one hyphen are split into their constituent parts.
   * Input: `please-don't-shoot`
   * Output: `please`, `don't`, `shoot`
   * *Rationale:* Prevents "sentence-words" from creating unique, statistically irrelevant tokens.
3. **Single Hyphens:** Preserved (e.g., `well-known`).

#### Step 4: Linguistic Validation

Before a token is accepted into `words.json` or the N-gram tracker, it must pass strict linguistic checks:

| Check | Rule | Rationale |
| :--- | :--- | :--- |
| **Structure** | Strict ASCII Alpha + `'` + `-` | Rejects numbers (`1990`) and mixed-symbol strings. |
| **Length** | `1 <= len <= 25` | Rejects OCR buffer overflows. |
| **Vowel Check** | Must contain `[aeiouy]` | Rejects acronyms/noise (`mzzt`, `krrk`). |
| **Consonant Cluster** | Max 6 consecutive consonants | Rejects agglutinative foreign words (e.g., Turkish) that slipped through. |
| **Repetition** | No 3+ identical chars | Rejects expressive typing (`nooooo`). |

### 2.3 Validation Tests

Automated Python scripts (`tests/validate_*.py`) are integrated into the build pipeline to ensure data integrity.

#### Test Suite 1: Vocabulary Verification (`validate_vocabulary.py`)

* **Method:** Cross-references `words.json` against a standard English dictionary (`words_alpha.txt`).
* **Metric:** **Token Volume Coverage**.
* **Threshold:** > 90% of the total text volume must be recognized English words.
* **Current Status:** ~96.6% Coverage (Remaining 3.4% are valid contractions like `don't`).

#### Test Suite 2: Structural Integrity (`validate_1grams.py`)

* **Category Distribution:** Verifies 100% of output chars are within the whitelist categories.
* **Artifact Scan:** Scans for zero-occurrence of forbidden chars (`\`, `_`, `â`, `\t`).
* **Entropy:** Verifies Shannon Entropy is within English norms (3.5 - 5.5 bits/char).

#### Test Suite 3: N-Grams (`validate_ngrams.py`)

* **Space Compression:** Verifies that the bigram `(" ", " ")` does not exist.
* **Linguistic Consistency:** Checks that the top 20 bigrams and trigrams align with standard English (e.g., "th", "he", "the", "and").

### 2.4 Weaknesses, Gaps, and Assumptions

While `en_std` provides a robust baseline for prose typing, the following limitations apply:

#### Domain Bias (Fiction)

* **Dialogue Heavy:** The corpus is dominated by fiction. Quotation marks and dialogue tags are over-represented.
* **Vocabulary:** Technical, scientific, and legal vocabulary is under-represented.

#### Strictness Trade-offs

* **Loan Words:** Common loan words with accents (`café`, `jalapeño`) are discarded entirely rather than normalized to (`cafe`, `jalapeno`). This is a trade-off for purity.
* **Numbers:** Numeric tokens are excluded.
* **Modern Communication:** The corpus does not reflect "internet slang," SMS-style abbreviations, or emoji usage.
