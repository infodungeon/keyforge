use crate::utils::get_data_dir;
use keyforge_core::api::{load_dataset, validate_layout, KeyForgeState};
use keyforge_core::config::ScoringWeights;
use std::collections::HashMap;
use std::fs;
use tauri::AppHandle;

#[tauri::command]
pub fn cmd_list_corpora(app: AppHandle) -> Result<Vec<String>, String> {
    let root = get_data_dir(&app)?;
    let mut corpora = Vec::new();
    if let Ok(entries) = fs::read_dir(&root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "tsv" {
                    if let Some(stem) = path.file_name().and_then(|s| s.to_str()) {
                        corpora.push(stem.to_string());
                    }
                }
            }
        }
    }
    corpora.sort();
    Ok(corpora)
}

#[tauri::command]
pub fn cmd_import_corpus(
    app: AppHandle,
    file_path: String,
    name: String,
) -> Result<String, String> {
    let data_dir = get_data_dir(&app)?;
    let target_path = data_dir.join(format!("{}.tsv", name));

    // 1. Read Source
    let content =
        fs::read_to_string(&file_path).map_err(|e| format!("Failed to read source file: {}", e))?;

    // 2. Process N-grams
    let mut monograms: HashMap<char, usize> = HashMap::new();
    let mut bigrams: HashMap<String, usize> = HashMap::new();
    let mut trigrams: HashMap<String, usize> = HashMap::new();

    // Normalize: Lowercase, keep only standard layout chars
    let valid_chars = "abcdefghijklmnopqrstuvwxyz.,;'[]-!?:\"()";

    let clean_text: Vec<char> = content
        .to_lowercase()
        .chars()
        .filter(|c| valid_chars.contains(*c) || c.is_whitespace())
        .collect();

    let filtered: Vec<char> = clean_text
        .into_iter()
        .filter(|c| !c.is_whitespace())
        .collect();

    // Sliding Window
    for i in 0..filtered.len() {
        // Monogram
        let c = filtered[i];
        *monograms.entry(c).or_default() += 1;

        // Bigram
        if i + 1 < filtered.len() {
            let s: String = filtered[i..i + 2].iter().collect();
            *bigrams.entry(s).or_default() += 1;
        }

        // Trigram
        if i + 2 < filtered.len() {
            let s: String = filtered[i..i + 3].iter().collect();
            *trigrams.entry(s).or_default() += 1;
        }
    }

    // 3. Format Output
    let mut output = String::new();

    // Write Monograms
    for (c, count) in monograms {
        output.push_str(&format!("{}\t{}\n", c, count));
    }

    // Helper function to append map data (Avoids closure borrow issues)
    fn append_ngrams(output: &mut String, map: HashMap<String, usize>) {
        let mut entries: Vec<_> = map.into_iter().collect();
        entries.sort_by(|a, b| b.1.cmp(&a.1)); // Sort DESC

        // Take top N to keep file size reasonable
        let limit = if entries.first().map(|x| x.0.len()).unwrap_or(0) == 2 {
            500
        } else {
            3000
        };

        for (k, v) in entries.into_iter().take(limit) {
            output.push_str(&format!("{}\t{}\n", k, v));
        }
    }

    // Write Bi/Tri using helper
    append_ngrams(&mut output, bigrams);
    append_ngrams(&mut output, trigrams);

    // 4. Save
    fs::write(&target_path, output).map_err(|e| format!("Failed to write TSV: {}", e))?;

    Ok(format!("Imported corpus '{}' successfully.", name))
}

#[tauri::command]
pub fn cmd_load_dataset(
    app: AppHandle,
    state: tauri::State<KeyForgeState>,
    keyboard_name: String,
    corpus_filename: String,
) -> Result<String, String> {
    let root = get_data_dir(&app)?;

    let cost = root.join("cost_matrix.csv");
    let ngrams = root.join(&corpus_filename);
    let geo = root
        .join("keyboards")
        .join(format!("{}.json", keyboard_name));

    if !cost.exists() {
        return Err(format!("Not found: {:?}", cost));
    }
    if !ngrams.exists() {
        return Err(format!("Not found: {:?}", ngrams));
    }
    if !geo.exists() {
        return Err(format!("Keyboard not found: {:?}", geo));
    }

    tracing::info!(
        "Loading Dataset: KB='{}' Corpus='{}'",
        keyboard_name,
        corpus_filename
    );

    load_dataset(
        &state,
        "primary",
        cost.to_str().unwrap(),
        ngrams.to_str().unwrap(),
        &Some(geo.to_str().unwrap().to_string()),
        None,
        Some(root.to_str().unwrap()),
    )
}

#[tauri::command]
pub fn cmd_validate_layout(
    state: tauri::State<KeyForgeState>,
    layout_str: String,
    weights: Option<ScoringWeights>,
) -> Result<keyforge_core::api::ValidationResult, String> {
    validate_layout(&state, "primary", layout_str, weights)
}
