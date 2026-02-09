#[keyforge_testing_macros::kf_test]
mod integration_tests {
    use super::*;
    use keyforge_infra::net::client::ClientConfig;
    use keyforge_infra::net::network::{ensure_cost_matrix, ensure_file};
    use keyforge_infra::*;
    use keyforge_model::constants::MAX_INPUT_FILE_SIZE;
    use sha2::{Digest, Sha256};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_ensure_file_basic() -> anyhow::Result<()> {
        let server = MockServer::start().await;
        let temp = tempfile::tempdir()?;
        let local_path = temp.path().join("test.txt");

        let content = "hello world";
        let expected_hash = hex::encode(Sha256::digest(content));

        Mock::given(method("GET"))
            .and(path("/data/test.txt"))
            .respond_with(ResponseTemplate::new(200).set_body_string(content))
            .mount(&server)
            .await;

        let config = ClientConfig {
            api_url: server.uri(),
            asset_url: server.uri(),
            ..Default::default()
        };
        let client = HiveClient::new(config)?;

        // 1. First download
        ensure_file(
            &client,
            &client.url("/data/test.txt"),
            &local_path,
            Some(&expected_hash),
        )
        .await?;
        assert_eq!(std::fs::read_to_string(&local_path)?, content);

        // 2. Reuse existing (trusted by sidecar)
        ensure_file(
            &client,
            &client.url("/data/test.txt"),
            &local_path,
            Some(&expected_hash),
        )
        .await?;

        // 3. Fallback to full content verification (delete sidecar)
        let sidecar = local_path.with_extension("txt.sha256");
        std::fs::remove_file(sidecar)?;
        ensure_file(
            &client,
            &client.url("/data/test.txt"),
            &local_path,
            Some(&expected_hash),
        )
        .await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ensure_cost_matrix() -> anyhow::Result<()> {
        let server = MockServer::start().await;
        let temp = tempfile::tempdir()?;

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string("{}"))
            .mount(&server)
            .await;

        let config = ClientConfig {
            api_url: server.uri(),
            asset_url: server.uri(),
            ..Default::default()
        };
        let client = HiveClient::new(config)?;

        let res = ensure_cost_matrix(&client, temp.path(), "cm.json").await?;
        assert!(res.exists());
        Ok(())
    }

    #[tokio::test]
    async fn test_ensure_file_hash_mismatch() -> anyhow::Result<()> {
        let server = MockServer::start().await;
        let temp = tempfile::tempdir()?;
        let local_path = temp.path().join("test.txt");

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string("wrong content"))
            .mount(&server)
            .await;

        let config = ClientConfig {
            api_url: server.uri(),
            asset_url: server.uri(),
            ..Default::default()
        };
        let client = HiveClient::new(config)?;

        let res = ensure_file(
            &client,
            &client.url("/data/test.txt"),
            &local_path,
            Some("expected_hash"),
        )
        .await;
        assert!(matches!(res, Err(InfraError::HashMismatch { .. })));
        Ok(())
    }

    #[tokio::test]
    async fn test_ensure_file_server_error() -> anyhow::Result<()> {
        let server = MockServer::start().await;
        let temp = tempfile::tempdir()?;
        let local_path = temp.path().join("test.txt");

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let config = ClientConfig {
            api_url: server.uri(),
            asset_url: server.uri(),
            ..Default::default()
        };
        let client = HiveClient::new(config)?;

        let res = ensure_file(&client, &client.url("/data/test.txt"), &local_path, None).await;
        assert!(res.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_ensure_file_not_found() -> anyhow::Result<()> {
        let server = MockServer::start().await;
        let temp = tempfile::tempdir()?;
        let local_path = temp.path().join("test.txt");

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        let config = ClientConfig {
            api_url: server.uri(),
            asset_url: server.uri(),
            ..Default::default()
        };
        let client = HiveClient::new(config)?;

        let res = ensure_file(&client, &client.url("/data/test.txt"), &local_path, None).await;
        assert!(res.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_ensure_file_too_large() -> anyhow::Result<()> {
        let server = MockServer::start().await;
        let temp = tempfile::tempdir()?;
        let local_path = temp.path().join("test.txt");

        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-length", (MAX_INPUT_FILE_SIZE + 1).to_string())
                    .set_body_string("too big"),
            )
            .mount(&server)
            .await;

        let config = ClientConfig {
            api_url: server.uri(),
            asset_url: server.uri(),
            ..Default::default()
        };
        let client = HiveClient::new(config)?;

        let res = ensure_file(&client, &client.url("/data/test.txt"), &local_path, None).await;
        assert!(res.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_ensure_corpus_bundle() -> anyhow::Result<()> {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_string("[]"))
            .mount(&server)
            .await;

        let config = ClientConfig {
            api_url: server.uri(),
            asset_url: server.uri(),
            ..Default::default()
        };
        let _client = HiveClient::new(config)?;
        Ok(())
    }
}
