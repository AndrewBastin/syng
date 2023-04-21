use std::collections::{BTreeMap, HashMap};

use serde::Serialize;
use syng::objects::SyngObjectDef;
use syng_demo_common::{CollectionData, RequestData};

#[derive(Debug, Serialize)]
pub struct ObjectGen {
    pub root_id: String,
    pub objects: HashMap<String, SyngObjectDef>,
}

pub fn generate_object_for_req(value: &RequestData) -> SyngObjectDef {
    let content =
        serde_json::to_string(&value).expect("RequestData to JSON Stringification failed");

    SyngObjectDef {
        fields: BTreeMap::from([
            ("type".to_owned(), "request".to_owned()),
            ("data".to_owned(), content),
        ]),
        children: vec![],
    }
}

pub fn generate_object_for_coll(
    coll: &CollectionData,
    obj_map: &mut HashMap<String, SyngObjectDef>,
) -> SyngObjectDef {
    let request_objs = coll
        .requests
        .iter()
        .map(|x| {
            let obj = generate_object_for_req(x);
            let hash = obj.get_hash().expect("Request Object hashing failed");

            (hash, obj)
        })
        .collect::<HashMap<String, SyngObjectDef>>();

    let req_hashes = request_objs
        .iter()
        .map(|x| x.0.to_owned())
        .collect::<Vec<String>>();

    obj_map.extend(request_objs);

    let collection_objs = coll
        .folders
        .iter()
        .map(|x| {
            let obj = generate_object_for_coll(x, obj_map);
            let hash = obj.get_hash().expect("Collection Object hashing failed");

            (hash, obj)
        })
        .collect::<HashMap<String, SyngObjectDef>>();

    let coll_hashes = collection_objs
        .iter()
        .map(|x| x.0.to_owned())
        .collect::<Vec<String>>();

    obj_map.extend(collection_objs);

    SyngObjectDef {
        fields: BTreeMap::from([
            ("type".to_owned(), "collection".to_owned()),
            ("title".to_owned(), coll.title.to_owned()),
        ]),
        children: [coll_hashes, req_hashes].concat(),
    }
}

impl From<&Vec<CollectionData>> for ObjectGen {
    fn from(value: &Vec<CollectionData>) -> Self {
        let mut obj_map = HashMap::<String, SyngObjectDef>::new();

        let root_obj_children = value
            .iter()
            .map(|coll| {
                let obj = generate_object_for_coll(coll, &mut obj_map);
                (obj.get_hash().unwrap(), obj)
            })
            .collect::<Vec<_>>();

        let root_obj = SyngObjectDef {
            fields: BTreeMap::new(),
            children: root_obj_children
                .iter()
                .map(|(hash, _)| hash.clone())
                .collect(),
        };
        let root_obj_id = root_obj.get_hash().unwrap();

        obj_map.extend(root_obj_children);
        obj_map.insert(root_obj_id.clone(), root_obj);

        Self {
            root_id: root_obj_id,
            objects: obj_map,
        }
    }
}
