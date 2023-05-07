use anyhow::Result;
use once_cell::sync::Lazy;
use reqwest::Client;
use syng::delta::SyngDelta;
use syng_demo_common::backend::{
    BackendCurrRootResult, BackendFullPullResult, BackendPullFromResult, BackendPushResult,
};

static CLIENT: Lazy<Client> = Lazy::new(|| Client::new());

pub async fn pull_full_from_remote() -> Result<BackendFullPullResult> {
    Ok(CLIENT
        .get("http://localhost:8080/pull")
        .send()
        .await?
        .json::<BackendFullPullResult>()
        .await?)
}

pub async fn get_current_remote_root() -> Result<BackendCurrRootResult> {
    Ok(CLIENT
        .get("http://localhost:8080/curr_root")
        .send()
        .await?
        .json::<BackendCurrRootResult>()
        .await?)
}

pub async fn pull_from_point_from_remote(point_hash: String) -> Result<BackendPullFromResult> {
    Ok(CLIENT
        .get(format!("http://localhost:8080/pull_from/{}", point_hash))
        .send()
        .await?
        .json::<BackendPullFromResult>()
        .await?)
}

pub async fn push_to_remote(delta: &SyngDelta) -> Result<BackendPushResult> {
    Ok(CLIENT
        .post("http://localhost:8080/push")
        .json(delta)
        .send()
        .await?
        .json::<BackendPushResult>()
        .await?)
}
