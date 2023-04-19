use serde::Serialize;

#[derive(Clone, Serialize, PartialEq, Debug)]
pub struct RequestData {
    pub title: String,
    pub content: String,
}

#[derive(Clone, Serialize, PartialEq, Debug)]
pub struct CollectionData {
    pub title: String,
    pub folders: Vec<CollectionData>,
    pub requests: Vec<RequestData>,
}

impl CollectionData {
    pub fn new(title: String) -> Self {
        CollectionData {
            title,
            folders: vec![],
            requests: vec![],
        }
    }
}
