// libs/keyforge-infra/src/util/corpus.rs

use keyforge_model::Corpus;
use keyforge_model::constants::{
    CORPUS_TOKEN_MAP, 
    STD_CORPUS_ERROR_RATE, 
    STD_CORPUS_BACKSPACE_FACTOR, 
    STD_CORPUS_SENTENCE_RATIO
};

/// Resolves a corpus token string to a character.
pub fn resolve_corpus_char(token: &str) -> Option<char> {
    for (key, val) in CORPUS_TOKEN_MAP {
        if token == *key {
            return Some(*val);
        }
    }
    if token.chars().count() == 1 {
        token.chars().next().map(|c| c.to_ascii_lowercase())
    } else {
        None
    }
}

/// Injects synthetic data (Enter, Backspace) for standard prose corpora.
pub fn inject_synthetic_data(corpus: &mut Corpus, is_std: bool) {
    if !is_std { return; }

    let total_chars: u64 = corpus.char_freqs.iter().sum();
    let sentence_count: u64 = 
        corpus.char_freqs['.' as usize] + 
        corpus.char_freqs['?' as usize] + 
        corpus.char_freqs['!' as usize];

    if total_chars == 0 { return; }

    let enter_count = (sentence_count as f32 / STD_CORPUS_SENTENCE_RATIO).round() as u64;
    let bksp_count = (total_chars as f32 * STD_CORPUS_ERROR_RATE * STD_CORPUS_BACKSPACE_FACTOR).round() as u64;

    corpus.char_freqs['\n' as usize] += enter_count;
    corpus.char_freqs['\x08' as usize] += bksp_count;

    if bksp_count > 0 {
        let mut new_bigrams = Vec::new();
        for (char_code, &freq) in corpus.char_freqs.iter().enumerate() {
            if freq > 0 && char_code != '\x08' as usize && char_code != '\n' as usize {
                let ratio = freq as f32 / total_chars as f32;
                let share = (bksp_count as f32 * ratio).round() as u32;
                if share > 0 {
                    new_bigrams.push((char_code as u16, '\x08' as u16, share));
                    new_bigrams.push(('\x08' as u16, char_code as u16, share));
                }
            }
        }
        corpus.bigrams.extend(new_bigrams);
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
                    corpus.bigrams.push((p as u16, '\n' as u16, share));
                }
            }
        }
    }

    if bksp_count > 0 {
        let total_bigrams: u64 = corpus.bigrams.iter().map(|(_, _, f)| *f as u64).sum();
        if total_bigrams > 0 {
            let mut new_trigrams = Vec::new();
            for (a, b, freq) in &corpus.bigrams {
                if *a == '\x08' as u16 || *b == '\x08' as u16 || *a == '\n' as u16 || *b == '\n' as u16 {
                    continue;
                }
                
                let ratio = *freq as f32 / total_bigrams as f32;
                let share = (bksp_count as f32 * ratio).round() as u32;
                
                if share > 0 {
                    new_trigrams.push((*a, *b, '\x08' as u16, share));
                }
            }
            corpus.trigrams.extend(new_trigrams);
        }
    }

    if enter_count > 0 {
        let puncts = ['.', '?', '!'];
        let mut new_trigrams = Vec::new();
        
        let punct_bigrams: Vec<_> = corpus.bigrams.iter()
            .filter(|(_, b, _)| puncts.contains(&(*b as u8 as char)))
            .collect();
            
        let total_punct_bigrams: u64 = punct_bigrams.iter().map(|(_, _, f)| *f as u64).sum();
        
        if total_punct_bigrams > 0 {
            for (a, b, freq) in punct_bigrams {
                let ratio = *freq as f32 / total_punct_bigrams as f32;
                let share = (enter_count as f32 * ratio).round() as u32;
                
                if share > 0 {
                    new_trigrams.push((*a, *b, '\n' as u16, share));
                }
            }
            corpus.trigrams.extend(new_trigrams);
        }
    }

    corpus.bigrams.sort_unstable_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    corpus.trigrams.sort_unstable_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
}
