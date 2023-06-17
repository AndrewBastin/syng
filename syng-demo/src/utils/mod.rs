use std::{
    alloc::System,
    future::Future,
    time::{Duration, SystemTime},
};

use random_string::generate;
use syng::{
    backend::SyngBackend,
    delta::{generate_delta_from_point, SyngDelta},
};

use crate::{
    remote::{get_current_remote_root, pull_from_point_from_remote},
    sync::backend::DemoFEBackend,
};

fn measure_time<T>(func: impl FnOnce() -> T) -> (Duration, T) {
    let time_start = SystemTime::now();
    let result = func();
    let time_end = SystemTime::now();

    (time_end.duration_since(time_start).unwrap(), result)
}

async fn measure_time_async<T, U>(func: impl FnOnce() -> U) -> (Duration, T)
where
    U: Future<Output = T>,
{
    let time_start = SystemTime::now();
    let result = func().await;
    let time_end = SystemTime::now();

    (time_end.duration_since(time_start).unwrap(), result)
}

pub fn path_to_string(path: &Vec<usize>) -> String {
    path.iter()
        .map(|i| i.to_string())
        .collect::<Vec<String>>()
        .join("/")
}

const REQ_CONTENT_CHARSET: &str = "0123456789abcdef";

pub fn get_random_request_content() -> String {
    generate(6, REQ_CONTENT_CHARSET)
}

pub enum DiffState {
    Even,
    RemoteAhead(SyngDelta),
    LocalAhead(SyngDelta),
    Diverged {
        local_delta: SyngDelta,
        remote_delta: SyngDelta,
    },
}

pub struct DiffGenResult {
    pub diff_fetch_time: Duration,
    pub diff_calc_time: Duration,
    pub state: DiffState,
}

pub async fn get_sync_status(
    last_synced_remote_root_id: String,
    backend: &DemoFEBackend,
) -> DiffGenResult {
    let (remote_root_fetch_time, remote_root_id) =
        measure_time_async(|| async { get_current_remote_root().await.unwrap().data.unwrap() })
            .await;

    let local_root_id = backend.get_root_object_id().unwrap();

    // Check if remote state is unchanged from last sync
    if remote_root_id == last_synced_remote_root_id {
        // Local state is the same as the remote state, hence even
        if remote_root_id == local_root_id {
            return DiffGenResult {
                diff_fetch_time: remote_root_fetch_time,
                diff_calc_time: Duration::from_secs(0),
                state: DiffState::Even,
            };
        } else {
            // Local state is not the same as remote state, but remote state unchanged since the
            // last sync, so we can assume that the local is ahead
            let (delta_calc_time, delta) = measure_time(|| {
                generate_delta_from_point(backend, &last_synced_remote_root_id)
                    .expect("Delta Generation from point failed")
            });

            return DiffGenResult {
                diff_fetch_time: remote_root_fetch_time,
                diff_calc_time: delta_calc_time,
                state: DiffState::LocalAhead(delta),
            };
        }
    } else {
        // Both of the cases we need the remote delta so just hoisting it
        let (remote_delta_fetch_time, remote_delta) = measure_time_async(|| async {
            pull_from_point_from_remote(last_synced_remote_root_id.to_owned())
                .await
                .unwrap()
        })
        .await;

        // The remote state is not the same since the last sync,
        if last_synced_remote_root_id == local_root_id {
            // The frontend is on the same point as the last sync, so the backend is ahead
            return DiffGenResult {
                diff_fetch_time: remote_root_fetch_time + remote_delta_fetch_time,
                diff_calc_time: Duration::from_secs(0),
                state: DiffState::RemoteAhead(remote_delta.data.unwrap()),
            };
        } else {
            let (local_delta_calc_time, local_delta) = measure_time(|| {
                generate_delta_from_point(backend, &last_synced_remote_root_id).unwrap()
            });

            return DiffGenResult {
                diff_fetch_time: remote_root_fetch_time + remote_delta_fetch_time,
                diff_calc_time: local_delta_calc_time,
                state: DiffState::Diverged {
                    local_delta,
                    remote_delta: remote_delta.data.unwrap(),
                },
            };
        }
    }
}
