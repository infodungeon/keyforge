// libs/keyforge-infra/src/util/corpus.rs

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use keyforge_adapter::loader::LoaderResult;
use keyforge_model::constants::{
    CORPUS_TOKEN_MAP, STD_CORPUS_BACKSPACE_FACTOR, STD_CORPUS_ERROR_RATE, STD_CORPUS_SENTENCE_RATIO,
};
use keyforge_model::error::ForgeError;
use keyforge_model::Corpus;
use serde_json::Value;
use std::sync::Arc;

/// Populates a corpus structure from raw n-gram segments with weighted frequencies.
///
/// # Errors
///
/// Returns `LoaderResult` if the input data is invalid.
pub fn populate_corpus_from_segments(
    corpus: &mut Corpus,
    weight: f32,
    segments: Vec<(&str, Vec<Value>)>,
) -> LoaderResult<()> {
    for (stem, part) in segments {
        match stem {
            "1grams" => parse_monograms(corpus, weight, &part)?,
            "2grams" => parse_bigrams(corpus, weight, &part)?,
            "3grams" => parse_trigrams(corpus, weight, &part)?,
            "words" => parse_words(corpus, weight, &part)?,
            _ => {}
        }
    }
    Ok(())
}

#[allow(clippy::cast_precision_loss, clippy::cast_sign_loss)]
fn parse_monograms(corpus: &mut Corpus, weight: f32, part: &[Value]) -> LoaderResult<()> {
    let mut freqs = corpus.char_freqs.to_vec();
    for e in part {
        if let Some(c) = e["char"].as_str().and_then(resolve_corpus_char) {
            if (c as u32) > 0xFFFF {
                return Err(ForgeError::InvalidData(format!(
                    "Character outside BMP not supported: {c}"
                )));
            }
            let c_u16 = c as u16;
            let freq = e["freq"].as_u64().ok_or_else(|| {
                ForgeError::InvalidData(format!("Missing frequency in 1gram entry: {e:?}"))
            })?;
            #[allow(
                clippy::cast_precision_loss,
                clippy::cast_sign_loss,
                clippy::cast_possible_truncation
            )]
            {
                freqs[c_u16 as usize] += (freq as f32 * weight).round() as u64;
            }
        }
    }
    corpus.char_freqs = Arc::from(freqs);
    Ok(())
}

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]
fn parse_bigrams(corpus: &mut Corpus, weight: f32, part: &[Value]) -> LoaderResult<()> {
    let mut bigrams = corpus.bigrams.to_vec();
    for e in part {
        let freq = e["freq"].as_u64().ok_or_else(|| {
            ForgeError::InvalidData(format!("Missing frequency in 2gram entry: {e:?}"))
        })?;
        let c1_char = e["char1"]
            .as_str()
            .and_then(resolve_corpus_char)
            .ok_or_else(|| {
                ForgeError::InvalidData(format!("Missing or invalid char1 in 2gram entry: {e:?}"))
            })?;
        let c2_char = e["char2"]
            .as_str()
            .and_then(resolve_corpus_char)
            .ok_or_else(|| {
                ForgeError::InvalidData(format!("Missing or invalid char2 in 2gram entry: {e:?}"))
            })?;

        if (c1_char as u32) > 0xFFFF || (c2_char as u32) > 0xFFFF {
            return Err(ForgeError::InvalidData(format!(
                "Character outside BMP not supported: {c1_char} or {c2_char}"
            )));
        }

        bigrams.push((
            c1_char as u16,
            c2_char as u16,
            (freq as f32 * weight).round() as u32,
        ));
    }
    corpus.bigrams = Arc::from(bigrams);
    Ok(())
}

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]
fn parse_trigrams(corpus: &mut Corpus, weight: f32, part: &[Value]) -> LoaderResult<()> {
    let mut trigrams = corpus.trigrams.to_vec();
    for e in part {
        let freq = e["freq"].as_u64().ok_or_else(|| {
            ForgeError::InvalidData(format!("Missing frequency in 3gram entry: {e:?}"))
        })?;
        let c1_char = e["char1"]
            .as_str()
            .and_then(resolve_corpus_char)
            .ok_or_else(|| {
                ForgeError::InvalidData(format!("Missing or invalid char1 in 3gram entry: {e:?}"))
            })?;
        let c2_char = e["char2"]
            .as_str()
            .and_then(resolve_corpus_char)
            .ok_or_else(|| {
                ForgeError::InvalidData(format!("Missing or invalid char2 in 3gram entry: {e:?}"))
            })?;
        let c3_char = e["char3"]
            .as_str()
            .and_then(resolve_corpus_char)
            .ok_or_else(|| {
                ForgeError::InvalidData(format!("Missing or invalid char3 in 3gram entry: {e:?}"))
            })?;

        if (c1_char as u32) > 0xFFFF || (c2_char as u32) > 0xFFFF || (c3_char as u32) > 0xFFFF {
            return Err(ForgeError::InvalidData(format!(
                "Character outside BMP not supported: {c1_char}, {c2_char}, or {c3_char}"
            )));
        }

        trigrams.push((
            c1_char as u16,
            c2_char as u16,
            c3_char as u16,
            (freq as f32 * weight).round() as u32,
        ));
    }
    corpus.trigrams = Arc::from(trigrams);
    Ok(())
}

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]
fn parse_words(corpus: &mut Corpus, weight: f32, part: &[Value]) -> LoaderResult<()> {
    let mut words = corpus.words.to_vec();
    for e in part {
        let freq = e["freq"].as_u64().ok_or_else(|| {
            ForgeError::InvalidData(format!("Missing frequency in word entry: {e:?}"))
        })?;
        if let Some(w) = e["word"].as_str() {
            words.push((w.to_string(), (freq as f32 * weight).round() as u32));
        }
    }
    corpus.words = Arc::from(words);
    Ok(())
}

/// Resolves a corpus token string to a character.
#[must_use]
pub fn resolve_corpus_char(token: &str) -> Option<char> {
    for (key, val) in CORPUS_TOKEN_MAP {
        if token == *key {
            return Some((*val).to_ascii_lowercase());
        }
    }

    if token.len() >= 2
        && token.len().is_multiple_of(2)
        && token.chars().all(|c| c.is_ascii_hexdigit())
    {
        let mut bytes = Vec::with_capacity(token.len() / 2);
        for i in (0..token.len()).step_by(2) {
            if let Ok(byte) = u8::from_str_radix(&token[i..i + 2], 16) {
                bytes.push(byte);
            } else {
                return None;
            }
        }
        if let Ok(s) = String::from_utf8(bytes) {
            return s.chars().next().map(|c| c.to_ascii_lowercase());
        }
    }

    if token.chars().count() == 1 {
        token.chars().next().map(|c| c.to_ascii_lowercase())
    } else {
        None
    }
}

/// Injects synthetic data (Enter, Backspace) for standard prose corpora.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub fn inject_synthetic_data(corpus: &mut Corpus, is_std: bool) {
    if !is_std {
        return;
    }

    let mut freqs = corpus.char_freqs.to_vec();
    let total_chars: u64 = freqs.iter().sum();
    let sentence_count: u64 = freqs['.' as usize] + freqs['?' as usize] + freqs['!' as usize];

    if total_chars == 0 {
        return;
    }

    let enter_count = (sentence_count as f32 / STD_CORPUS_SENTENCE_RATIO).round() as u64;
    let bksp_count =
        (total_chars as f32 * STD_CORPUS_ERROR_RATE * STD_CORPUS_BACKSPACE_FACTOR).round() as u64;

    freqs['\n' as usize] += enter_count;
    freqs['\x08' as usize] += bksp_count;
    corpus.char_freqs = Arc::from(freqs);

    let mut bigrams = corpus.bigrams.to_vec();
    if bksp_count > 0 {
        let mut new_bigrams = Vec::new();
        for (char_code, &freq) in corpus.char_freqs.iter().enumerate() {
            if freq > 0 && char_code != '\x08' as usize && char_code != '\n' as usize {
                let ratio = freq as f32 / total_chars as f32;
                let share = (bksp_count as f32 * ratio).round() as u32;
                if share > 0 {
                    new_bigrams.push(('\x08' as u16, char_code as u16, share));
                    new_bigrams.push((char_code as u16, '\x08' as u16, share));
                }
            }
        }
        bigrams.extend(new_bigrams);
    }

    if enter_count > 0 {
        let puncts = ['.', '?', '!'];
        let total_punct = sentence_count.max(1);

        for p in puncts {
            let p_freq = corpus.char_freqs[p as usize];
            if p_freq > 0 {
                let ratio = p_freq as f32 / total_punct as f32;
                let share = (enter_count as f32 * ratio).round() as u32;
                if share > 0 {
                    bigrams.push((p as u16, '\n' as u16, share));
                }
            }
        }
    }
    bigrams.sort_unstable_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    corpus.bigrams = Arc::from(bigrams);

    let mut trigrams = corpus.trigrams.to_vec();
    if bksp_count > 0 {
        let total_bigrams: u64 = corpus.bigrams.iter().map(|(_, _, f)| u64::from(*f)).sum();
        if total_bigrams > 0 {
            let mut new_trigrams = Vec::new();
            for (a, b, freq) in &*corpus.bigrams {
                if *a == '\x08' as u16
                    || *b == '\x08' as u16
                    || *a == '\n' as u16
                    || *b == '\n' as u16
                {
                    continue;
                }

                let ratio = *freq as f32 / total_bigrams as f32;
                let share = (bksp_count as f32 * ratio).round() as u32;

                if share > 0 {
                    new_trigrams.push((*a, *b, '\x08' as u16, share));
                }
            }
            trigrams.extend(new_trigrams);
        }
    }

    if enter_count > 0 {
        let puncts = ['.', '?', '!'];
        let mut new_trigrams = Vec::new();

        let punct_bigrams: Vec<_> = corpus
            .bigrams
            .iter()
            .filter(|(_, b, _)| puncts.contains(&(*b as u8 as char)))
            .collect();

        let total_punct_bigrams: u64 = punct_bigrams.iter().map(|(_, _, f)| u64::from(*f)).sum();

        if total_punct_bigrams > 0 {
            for (a, b, freq) in punct_bigrams {
                let ratio = *freq as f32 / total_punct_bigrams as f32;
                let share = (enter_count as f32 * ratio).round() as u32;

                if share > 0 {
                    new_trigrams.push((*a, *b, '\n' as u16, share));
                }
            }
            trigrams.extend(new_trigrams);
        }
    }

    trigrams.sort_unstable_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
    corpus.trigrams = Arc::from(trigrams);
}

#[keyforge_testing_macros::kf_test]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_resolve_corpus_char() {
        assert_eq!(resolve_corpus_char("SPACE"), Some(' '));
        assert_eq!(resolve_corpus_char("A"), Some('a')); // Normalization
        assert_eq!(resolve_corpus_char("61"), Some('a')); // Hex
        assert_eq!(resolve_corpus_char("616"), None); // Invalid hex length
        assert_eq!(resolve_corpus_char("6G"), None); // Invalid hex char
        assert_eq!(resolve_corpus_char("invalid"), None);
    }

    #[test]
    fn test_populate_corpus_from_segments() {
        let mut corpus = Corpus::default();
        let segments = vec![
            ("1grams", vec![json!({"char": "a", "freq": 100})]),
            (
                "2grams",
                vec![json!({"char1": "a", "char2": "b", "freq": 50})],
            ),
            (
                "3grams",
                vec![json!({"char1": "a", "char2": "b", "char3": "c", "freq": 10})],
            ),
            ("words", vec![json!({"word": "test", "freq": 5})]),
        ];

        populate_corpus_from_segments(&mut corpus, 1.0, segments).unwrap();
        assert_eq!(corpus.char_freqs[97], 100);
        assert_eq!(corpus.bigrams.len(), 1);
        assert_eq!(corpus.trigrams.len(), 1);
        assert_eq!(corpus.words.len(), 1);
    }

    #[test]
    fn test_inject_synthetic_data() {
        let mut corpus = Corpus::default();
        let mut freqs = corpus.char_freqs.to_vec();
        freqs['a' as usize] = 1000;
        freqs['.' as usize] = 10;
        corpus.char_freqs = Arc::from(freqs);

        corpus.bigrams = Arc::from(vec![('a' as u16, '.' as u16, 100)]);

        inject_synthetic_data(&mut corpus, true);

        assert!(corpus.char_freqs['\n' as usize] > 0); // Enter injected
        assert!(corpus.char_freqs['\x08' as usize] > 0); // Backspace injected
        assert!(corpus
            .bigrams
            .iter()
            .any(|(a, b, _)| *a == '.' as u16 && *b == '\n' as u16));
    }

    #[test]
    fn test_parse_monograms_invalid() {
        let mut corpus = Corpus::default();
        let part = vec![json!({"char": "a"})]; // Missing freq
        assert!(parse_monograms(&mut corpus, 1.0, &part).is_err());

        // Outside BMP
        let part = vec![json!({"char": "🦀", "freq": 100})];
        assert!(parse_monograms(&mut corpus, 1.0, &part).is_err());
    }

    #[test]
    fn test_parse_bigrams_invalid() {
        let mut corpus = Corpus::default();
        assert!(parse_bigrams(&mut corpus, 1.0, &[json!({"char1":"a","char2":"b"})]).is_err());
        assert!(parse_bigrams(&mut corpus, 1.0, &[json!({"freq":100,"char2":"b"})]).is_err());
        assert!(parse_bigrams(&mut corpus, 1.0, &[json!({"freq":100,"char1":"a"})]).is_err());
        assert!(parse_bigrams(
            &mut corpus,
            1.0,
            &[json!({"freq":100,"char1":"🦀","char2":"b"})]
        )
        .is_err());
    }

    #[test]
    fn test_parse_trigrams_invalid() {
        let mut corpus = Corpus::default();
        assert!(parse_trigrams(
            &mut corpus,
            1.0,
            &[json!({"char1":"a","char2":"b","char3":"c"})]
        )
        .is_err());
        assert!(parse_trigrams(
            &mut corpus,
            1.0,
            &[json!({"freq":100,"char2":"b","char3":"c"})]
        )
        .is_err());
        assert!(parse_trigrams(
            &mut corpus,
            1.0,
            &[json!({"freq":100,"char1":"a","char3":"c"})]
        )
        .is_err());
        assert!(parse_trigrams(
            &mut corpus,
            1.0,
            &[json!({"freq":100,"char1":"a","char2":"b"})]
        )
        .is_err());
        assert!(parse_trigrams(
            &mut corpus,
            1.0,
            &[json!({"freq":100,"char1":"🦀","char2":"b","char3":"c"})]
        )
        .is_err());
    }

    #[test]
    fn test_parse_words_invalid() {
        let mut corpus = Corpus::default();
        assert!(parse_words(&mut corpus, 1.0, &[json!({"word":"test"})]).is_err());
    }
}
