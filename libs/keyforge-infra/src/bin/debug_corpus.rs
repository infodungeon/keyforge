use keyforge_infra::FsProvider;
use keyforge_core::loader::AssetLoader;
use keyforge_model::config::CorpusSource;

#[tokio::main]
async fn main() {
    let root = std::env::current_dir().unwrap().join("sandbox/client");
    let provider = FsProvider::new(root);
    
    let sources = vec![CorpusSource {
        id: "text/en_std".to_string(),
        weight: 1.0,
        hash: None,
    }];

    match provider.load_corpus(&sources).await {
        Ok(corpus) => {
            let total_chars: u32 = corpus.char_freqs.iter().sum();
            let space_freq = corpus.char_freqs[' ' as usize];
            let e_freq = corpus.char_freqs['e' as usize];
            let bksp_freq = corpus.char_freqs['\x08' as usize];
            
            println!("Total Chars: {}", total_chars);
            println!("Space Freq: {}", space_freq);
            println!("'e' Freq: {}", e_freq);
            println!("Backspace Freq: {}", bksp_freq);
            println!("Bigrams: {}", corpus.bigrams.len());
            println!("Trigrams: {}", corpus.trigrams.len());
        }
        Err(e) => println!("Error: {}", e),
    }
}
