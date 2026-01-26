// libs/keyforge-compute/tests/runtime_integration.rs

#[keyforge_testing_macros::kf_test]
mod runtime_tests {
    use super::*;
    use async_trait::async_trait;
    use keyforge_adapter::loader::{AssetLoader, LoaderResult};
    use keyforge_compute::Runtime;
    use keyforge_model::geometry::{KeyboardDefinition, KeyboardMeta};
    use keyforge_model::{Asset, Corpus};
    use std::path::Path;
    use std::sync::Arc;

    #[derive(Debug)]
    struct MockLoader;

    #[async_trait]
    impl AssetLoader for MockLoader {
        async fn load<T: Asset>(&self, _id: &str) -> LoaderResult<Arc<T>> {
            let kb_any = Arc::new(KeyboardDefinition {
                meta: KeyboardMeta::default(),
                ..Default::default()
            }) as Arc<dyn std::any::Any + Send + Sync>;
            Ok(kb_any.downcast::<T>().unwrap())
        }

        async fn load_corpus(
            &self,
            _sources: &[keyforge_model::config::CorpusSource],
        ) -> LoaderResult<Arc<Corpus>> {
            Ok(Arc::new(Corpus::default()))
        }

        fn root(&self) -> &Path {
            Path::new(".")
        }
    }

    #[tokio::test]
    async fn test_runtime_execution() {
        let loader = MockLoader;
        let session = keyforge_compute::SessionBuilder::new(&loader)
            .with_keyboard_def(Arc::new(KeyboardDefinition::default()))
            .with_corpus(&[])
            .await
            .unwrap()
            .with_keycodes("default")
            .await
            .unwrap()
            .build()
            .unwrap();

        let runtime = Runtime::from(session);
        let logger = keyforge_evolution::NoOpCallback;
        // Fix: Pass logger by value to satisfy ProgressCallback + 'static bound
        let res = runtime.run_optimization(logger, &[]).await;
        assert!(res.is_ok());
    }
}
