use serde::{Deserialize, Serialize};
use syng::delta::{ApplyDeltaError, SyngDelta};
use syng::objects::SyngObjectDef;

#[derive(Serialize, Deserialize, Debug)]
pub struct BackendCurrRootResult {
    pub data: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BackendFullPullResult {
    pub root_obj_id: Option<String>,
    pub objects: Vec<SyngObjectDef>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum BackendPullFromError {
    BackendHasNoRoot,
    InvalidFromPoint,
    DeltaGenError,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BackendPullFromResult {
    pub data: Result<SyngDelta, BackendPullFromError>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum BackendPushError {
    DeltaApplyFailed(ApplyDeltaError),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BackendPushResult {
    pub data: Result<(), BackendPushError>,
}
