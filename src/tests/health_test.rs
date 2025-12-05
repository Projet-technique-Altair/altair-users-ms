#[tokio::test]
async fn health_is_ok() {
    let resp = reqwest::get("http://localhost:3001/users/health")
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert!(resp.contains("\"status\":\"ok\""));
}
