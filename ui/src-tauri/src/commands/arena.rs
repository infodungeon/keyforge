use crate::models::{BiometricSample, UserStatsStore};
use crate::utils::get_data_dir;
use fastrand;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::AppHandle;

#[tauri::command]
pub fn cmd_get_typing_words(app: AppHandle, count: usize) -> Result<Vec<String>, String> {
    let data_dir = get_data_dir(&app)?;
    let corpus_path = data_dir.join("google-books-common-words.txt");

    if !corpus_path.exists() {
        return Err("Corpus file not found at data/google-books-common-words.txt".into());
    }

    let content = fs::read_to_string(corpus_path).map_err(|e| e.to_string())?;

    let mut pool: Vec<String> = content
        .lines()
        .take(2000)
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('\t').collect();
            if !parts.is_empty() {
                let word = parts[0].to_lowercase();
                if (word.len() > 1 || word == "a" || word == "i")
                    && word.chars().all(|c| c.is_alphabetic())
                {
                    return Some(word);
                }
            }
            None
        })
        .collect();

    let mut rng = fastrand::Rng::new();
    rng.shuffle(&mut pool);

    let selected: Vec<String> = pool.into_iter().take(count).collect();
    Ok(selected)
}

#[tauri::command]
pub fn cmd_save_biometrics(
    app: AppHandle,
    samples: Vec<BiometricSample>,
) -> Result<String, String> {
    let data_dir = get_data_dir(&app)?;
    let stats_path = data_dir.join("user_stats.json");

    // 1. Load existing stats or create new
    let mut store = if stats_path.exists() {
        let content = fs::read_to_string(&stats_path).map_err(|e| e.to_string())?;
        serde_json::from_str::<UserStatsStore>(&content).unwrap_or_default()
    } else {
        UserStatsStore::default()
    };

    // 2. Append new data
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    store.sessions += 1;
    store.total_keystrokes += samples.len() as u64;

    // Add timestamp to samples if missing (though UI sends it, we enforce current time for batch)
    // Actually, we trust the incoming samples but ensure they are valid.
    for mut s in samples {
        if s.timestamp == 0 {
            s.timestamp = now;
        }
        store.biometrics.push(s);
    }

    // 3. Prune if too large?
    // For now, we keep all data. JSON handles ~100k entries fine (approx 5MB).
    // Future optimization: Rotate files or compress.

    // 4. Save
    let json = serde_json::to_string_pretty(&store).map_err(|e| e.to_string())?;
    fs::write(stats_path, json).map_err(|e| e.to_string())?;

    Ok(format!(
        "Saved {} samples. Total: {}",
        store.total_keystrokes,
        store.biometrics.len()
    ))
}
