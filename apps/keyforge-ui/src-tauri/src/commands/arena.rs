use crate::utils::get_data_dir;
use keyforge_infra::AssetLoader;
use keyforge_infra::FsProvider;
use keyforge_persistence::UserRepo;
use keyforge_model::config::CorpusSource;
use keyforge_protocol::BiometricSample;
use keyforge_model::constants::ARENA_TOP_WORDS_LIMIT;
use tauri::AppHandle;

/// Generates a list of random words from the selected corpora for typing practice.
#[tauri::command]
pub async fn cmd_get_typing_words(
    app: AppHandle,
    corpora: Vec<CorpusSource>,
    count: usize,
) -> Result<Vec<String>, String> {
    use keyforge_adapter::conversion;
    let data_dir = get_data_dir(&app)?;
    let provider = FsProvider::new(data_dir);
    
    let domain_corpora: Vec<CorpusSource> = corpora
        .iter()
        .map(conversion::to_domain_corpus_source)
        .collect();

    let bundle = provider
        .load_corpus(&domain_corpora)
        .await
        .map_err(|e| format!("Failed to load corpora for Arena: {e}"))?;

    if bundle.words.is_empty() {
        return Err("The selected corpora contain no word data.".into());
    }

    let mut rng = fastrand::Rng::new();
    let mut selected = Vec::with_capacity(count);

    let top_n = ARENA_TOP_WORDS_LIMIT.min(bundle.words.len());
    let candidates = &bundle.words[0..top_n];
    let total_freq: f64 = candidates.iter().map(|(_, f)| f64::from(*f)).sum();

    for _ in 0..count {
        let target = rng.f64() * total_freq;
        let mut current = 0.0;
        let mut picked = &candidates[0].0;

        for (w, f) in candidates {
            current += f64::from(*f);
            if current >= target {
                picked = w;
                break;
            }
        }
        selected.push(picked.clone());
    }

    Ok(selected)
}

/// Persists typing session biometric data to the local user repository.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn cmd_save_biometrics(
    app: AppHandle,
    samples: Vec<BiometricSample>,
) -> Result<String, String> {
    let data_dir = get_data_dir(&app)?;
    let user_data = UserRepo::new(data_dir);
    user_data
        .record_biometrics(samples)
        .map_err(|e| e.to_string())
}

/// Loads historical biometric data for the current user.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn cmd_load_user_stats(app: AppHandle) -> Result<Vec<BiometricSample>, String> {
    let data_dir = get_data_dir(&app)?;
    let user_data = UserRepo::new(data_dir);
    Ok(user_data.get_biometrics())
}

/// Analyzes biometric data to generate a personalized typing profile.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn cmd_generate_personal_profile(app: AppHandle) -> Result<String, String> {
    let data_dir = get_data_dir(&app)?;
    let user_data = UserRepo::new(data_dir);
    user_data.generate_profile().map_err(|e| e.to_string())
}

/// Clears all historical biometric data for the current user.
#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub fn cmd_reset_user_stats(app: AppHandle) -> Result<String, String> {
    let data_dir = get_data_dir(&app)?;
    let user_data = UserRepo::new(data_dir);
    user_data.reset_biometrics().map_err(|e| e.to_string())?;
    Ok("Biometric data cleared successfully.".to_string())
}

/// Retrieves the most frequent bigrams from the selected corpora.
#[tauri::command]
pub async fn cmd_get_corpus_bigrams(
    app: AppHandle,
    corpora: Vec<CorpusSource>,
    limit: usize,
) -> Result<Vec<String>, String> {
    use keyforge_adapter::conversion;
    let data_dir = get_data_dir(&app)?;
    let provider = FsProvider::new(data_dir);

    let domain_corpora: Vec<CorpusSource> = corpora
        .iter()
        .map(conversion::to_domain_corpus_source)
        .collect();

    let bundle = provider
        .load_corpus(&domain_corpora)
        .await
        .map_err(|e| format!("Failed to load corpora: {e}"))?;

    let mut bigrams = Vec::new();

    let mut sorted_bgs = bundle.bigrams.clone();
    sorted_bgs.sort_by(|a, b| b.2.cmp(&a.2));

    for (b1, b2, _) in sorted_bgs.into_iter().take(limit) {
        let mut s = String::with_capacity(4);
        if let Some(c1) = std::char::from_u32(u32::from(b1)) { s.push(c1); }
        if let Some(c2) = std::char::from_u32(u32::from(b2)) { s.push(c2); }
        
        if !s.is_empty() && s.chars().all(char::is_alphabetic) {
            bigrams.push(s);
        }
    }

    Ok(bigrams)
}
