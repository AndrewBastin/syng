use crate::data::CollectionData;
use random_string::generate;

pub fn path_to_string(path: &Vec<usize>) -> String {
    path.iter()
        .map(|i| i.to_string())
        .collect::<Vec<String>>()
        .join("/")
}

pub fn get_collection_mut<'a>(
    colls: &'a mut Vec<CollectionData>,
    path: &Vec<usize>,
) -> Option<&'a mut CollectionData> {
    if path.len() == 0 {
        return None;
    };

    let mut looking_collection = colls.get_mut(path[0])?;

    for i in path.iter().skip(1) {
        looking_collection = looking_collection.folders.get_mut(*i)?;
    }

    Some(looking_collection)
}

const REQ_CONTENT_CHARSET: &str = "0123456789abcdef";

pub fn get_random_request_content() -> String {
    generate(6, REQ_CONTENT_CHARSET)
}
