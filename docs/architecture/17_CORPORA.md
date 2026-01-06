# Corpora & Data Processing

This document details the acquisition, cleansing, and validation strategies for the text corpora used to generate frequency statistics (N-grams and words) for Keyforge.

## 1. Data Cleansing Philosophy

The primary goal of the Keyforge data pipeline is to model **human typing behavior**, not to preserve the typographic fidelity of the source documents. As such, the cleansing strategy is aggressive and strictly whitelist-based.

### Core Principles

1. **Typing vs. Typesetting:** Priority is placed on characters that exist on a standard keyboard. Typographic artifacts (smart quotes, ligatures, soft hyphens) are normalized to their keystroke equivalents or removed.
2. **Strict Word Definition:** The pipeline enforces a linguistic definition of "words" to prevent OCR errors, formatting artifacts, and concatenation errors (e.g., "today--actually") from polluting the frequency data.
3. **Flow Interruption:** When a word is discarded, the N-gram statistical chain is **reset**. The preceding word is not stitched to the following word, as this would generate false adjacency data (phantom N-grams) that the user never typed.
4. **Space Compression:** Human typing often involves variable whitespace. For statistical purposes, all sequences of horizontal whitespace (spaces, tabs) are compressed into a single Space event. Double dashes (`--`) are treated as semantic separators (spaces), not word characters.

---

## 2. Corpus: `en_std` (Modern English Prose)

The `en_std` corpus represents Standard Modern English with a focus on creative writing, dialogue, and narrative flow. It serves as the baseline for general-purpose keyboard optimization.

### 2.1 Source Data

* **Dataset Name:** `lucadiliello/bookcorpusopen` (Hugging Face)
* **Description:** An open replication of the original BookCorpus dataset (Zhu et al., 2015). It consists of 17,868 self-published books scraped from Smashwords.
* **Format:** Parquet (Columnar).
* **Structure:** One row per book.
* **Volume:** ~6 Billion characters.

### 2.2 Processing Pipeline

The raw data undergoes a parallel, zero-copy streaming transformation using a custom Rust tokenizer.

#### Step 1: Normalization

Before validation, characters are mapped to their standard keyboard equivalents to resolve typesetting artifacts.

| Category | Source Character(s) | Mapped To |
| :--- | :--- | :--- |
| **Quotes** | `“` `”` `„` | `"` |
| **Apostrophes** | `‘` `’` `´` `` ` `` | `'` |
| **Dashes** | `–` `—` `―` | `-` (Normalized for buffer, handled in Step 3) |
| **Ligatures** | `ﬁ` `ﬂ` `ﬀ` `ﬃ` `ﬄ` | `fi` `fl` `ff` `ffi` `ffl` |
| **Latin** | `æ` `œ` | `ae` `oe` |

#### Step 2: Artifact Stripping

Specific characters identified as "digital noise" or formatting metadata are mapped to spaces or removed to serve as delimiters.

* **Soft Hyphen (`\u00ad`):** Invisible formatting char; mapped to Space.
* **Control Chars (`\u009d`):** Encoding errors; mapped to Space.
* **Backslash (`\`):** Escape sequence artifacts; mapped to Space.
* **Underscore (`_`):** Markdown markers; mapped to Space.

#### Step 3: Tokenization Strategy

Characters are accumulated into a buffer. When a non-word character (anything other than alphanumeric, `'`, or `-`) is encountered, the buffer is processed:

1. **Double Dash Expansion:** The sequence `--` is explicitly treated as a separator.
    * Input: `today--actually`
    * Output: `today`, `actually`
2. **Multi-Hyphen Splitting:** Words containing more than one hyphen are split into their constituent parts.
    * Input: `please-don't-shoot`
    * Output: `please`, `don't`, `shoot`
    * *Rationale:* Prevents "sentence-words" from creating unique, statistically irrelevant tokens.
3. **Single Hyphens:** Preserved if used correctly (e.g., `well-known`, `co-op`).

#### Step 4: The Word Filter

Before a token is counted or fed into the N-gram tracker, it must pass a strict validation check. **If a token fails, it is discarded entirely.**

| Check | Condition | Rationale |
| :--- | :--- | :--- |
| **Length** | `1 <= len <= 25` | Removes buffer overflows (OCR errors missing spaces) and empty strings. |
| **Character Set** | Alphabetic, `'`, `-` only | **Excludes numbers.** "1990" is not a word for linguistic scoring. |
| **Vowel Check** | Must contain `[aeiouy]` | Removes OCR noise (e.g., `mzzt`, `krrk`) and acronyms without structure. |
| **Repetition** | No 3+ identical chars | Removes expressive typing (e.g., `nooooo`, `arrrrgh`) and keyboard mashes. |

### 2.3 Validation Tests

Automated Python scripts (`tests/validate_*.py`) are integrated into the build pipeline to ensure data integrity.

#### Test Suite 1: 1-Grams (`validate_1grams.py`)

* **Category Distribution:** Verifies 100% of output chars are within the whitelist categories.
* **Artifact Scan:** Scans for zero-occurrence of forbidden chars (`\`, `_`, `â`, `\t`).
* **Zipf's Law:** Checks correlation coefficient (< -0.85) to ensure natural language distribution.
* **Entropy:** Verifies Shannon Entropy is within English norms (3.5 - 5.5 bits/char).
* **ETAOIN:** Verifies the top 12 most frequent letters match standard English expectations.

#### Test Suite 2: N-Grams (`validate_ngrams.py`)

* **Space Compression:** Verifies that the bigram `(" ", " ")` does not exist.
* **Linguistic Consistency:** Checks that the top 20 bigrams and trigrams align with standard English (e.g., "th", "he", "the", "and").
* **Artifact Scan:** Ensures no control characters or double-dashes leak into N-grams.

#### Test Suite 3: Words (`validate_words.py`)

* **Structure:** Verifies no words contain double-hyphens or start/end with non-alphabetic characters.
* **Word Length:** Verifies weighted average word length is between 4.0 and 6.0 characters.
* **Vocabulary:** Checks that the top 10 words include standard stop words ("the", "of", "and", "to").
* **Zipf's Law:** Checks for strict adherence (< -0.95 correlation).

### 2.4 Weaknesses, Gaps, and Assumptions

While `en_std` provides a robust baseline for prose typing, the following limitations apply:

#### Domain Bias (Fiction)

* **Dialogue Heavy:** The corpus is dominated by fiction. Quotation marks and dialogue tags are over-represented.
* **Vocabulary:** Technical, scientific, and legal vocabulary is under-represented.

#### Key Gaps

* **Numbers:** Words containing digits (e.g., "3D", "y2k") are explicitly filtered out to preserve linguistic purity. Numeric frequency must be modeled separately or via synthetic injection.
* **Tab Key:** All tabs are converted to spaces.
* **Modern Communication:** The corpus does not reflect "internet slang," SMS-style abbreviations, or emoji usage.

#### Assumptions

* **Enter = Paragraph:** It is assumed that a newline character (`\n`) represents a conscious "Enter" keystroke by the user.
* **Standard US Layout:** The whitelist assumes a standard US ANSI keyboard layout. Regional punctuation (e.g., `£`, `€`) is discarded.
