#[tokio::test]
async fn me_route_responds() {
    let resp = reqwest::get("http://localhost:3001/users/me")
        .await
        .unwrap();

    assert!(resp.status().is_success());
}
