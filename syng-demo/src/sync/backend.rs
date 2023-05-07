use anyhow::{bail, Result};
use syng_demo_common::{backend::BackendFullPullResult, CollectionData, RequestData};

use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

use syng::{
    backend::SyngBackend,
    delta::{generate_delta_from_point, SyngDelta},
    objects::SyngObjectDef,
    tree_ops::{
        add_child_node, get_descendent_object_ids, get_object_at_path, remove_child_node,
        ChildAdditionPosition,
    },
};

use super::treegen::{generate_object_for_coll, generate_object_for_req, ObjectGen};

#[derive(Debug)]
pub struct DemoFEBackend {
    root_id: Option<String>,
    objects: HashMap<String, SyngObjectDef>,
}

impl SyngBackend for DemoFEBackend {
    fn get_root_object_id(&self) -> Option<String> {
        self.root_id.clone()
    }

    fn get_root_object(&self) -> Option<SyngObjectDef> {
        let root_id = self.root_id.clone()?;

        self.read_object(&root_id)
    }

    fn set_root_object(&mut self, node_id: &str) -> Result<()> {
        if !self.has_object(node_id) {
            bail!("Tried to set root object to non-existent hash")
        }

        self.root_id = Some(node_id.to_owned());

        Ok(())
    }

    fn read_object(&self, id: &str) -> Option<SyngObjectDef> {
        Some(self.objects.get(id)?.clone())
    }

    fn write_object(&mut self, def: &SyngObjectDef) -> Result<String> {
        let hash = def.get_hash()?;

        self.objects.insert(hash.clone(), def.clone());

        Ok(hash)
    }
}

pub enum NodeTranslation {
    Collection(CollectionData),
    Request(RequestData),
}

impl Default for DemoFEBackend {
    fn default() -> Self {
        let empty_node = SyngObjectDef {
            fields: BTreeMap::new(),
            children: vec![],
        };

        let hash = empty_node.get_hash().expect("Hashing empty node failed");

        let mut object_store = HashMap::new();
        object_store.insert(hash.clone(), empty_node);

        Self {
            objects: object_store,
            root_id: Some(hash),
        }
    }
}

impl DemoFEBackend {
    pub fn apply_full_pull(&mut self, data: &BackendFullPullResult) -> Result<()> {
        for obj in &data.objects {
            self.write_object(&obj).expect("Pull object write failed");
        }

        self.root_id = data.root_obj_id.clone();

        Ok(())
    }

    pub fn get_delta_for_pushing(&self, past_point: &str) -> Result<SyngDelta> {
        let delta = generate_delta_from_point(self, &past_point).expect("Delta gen failed");

        Ok(delta)
    }

    pub fn drop_unreachable_objects(&mut self, last_sync_point: &Option<String>) -> Result<()> {
        let active_objects =
            get_descendent_object_ids(self, &self.get_root_object_id().unwrap()).unwrap();

        let sync_point_active_objects = match last_sync_point {
            Some(past_point) => get_descendent_object_ids(self, past_point).unwrap(),
            None => vec![],
        };

        self.objects.retain(|hash, _| {
            active_objects.contains(hash) || sync_point_active_objects.contains(hash)
        });

        Ok(())
    }

    pub fn generate_gen_info(&self) -> Option<ObjectGen> {
        Some(ObjectGen {
            root_id: self.root_id.clone()?,
            objects: self.objects.clone(),
        })
    }

    pub fn get_collection(&self, path: &[usize]) -> Option<CollectionData> {
        let coll_obj = get_object_at_path(self, path)?.1;

        self.parse_collection_from_obj(&coll_obj)
    }

    pub fn parse_request_from_obj(&self, obj: &SyngObjectDef) -> Option<RequestData> {
        if obj.fields.get("type")? != "request" {
            return None;
        }

        let req_data = serde_json::from_str(obj.fields.get("data")?).ok()?;

        Some(req_data)
    }

    pub fn parse_collection_from_obj(&self, obj: &SyngObjectDef) -> Option<CollectionData> {
        if obj.fields.get("type")? != "collection" {
            return None;
        }

        let Some(coll_title) = obj.fields.get("title") else { return None };

        let mut requests = vec![];
        let mut collections = vec![];

        for hash in &obj.children {
            let translation = self.translate_node(&hash).expect("Node translation fail");

            match translation {
                NodeTranslation::Collection(coll) => collections.push(coll),
                NodeTranslation::Request(req) => requests.push(req),
            }
        }

        Some(CollectionData {
            title: coll_title.clone(),
            requests,
            folders: collections,
        })
    }

    pub fn translate_node(&self, node_id: &str) -> Option<NodeTranslation> {
        let obj = self.read_object(node_id)?;

        if let Some(req_data) = self.parse_request_from_obj(&obj) {
            return Some(NodeTranslation::Request(req_data));
        }

        if let Some(coll_data) = self.parse_collection_from_obj(&obj) {
            return Some(NodeTranslation::Collection(coll_data));
        }

        None
    }

    pub fn get_collection_tree(&self) -> Option<Vec<CollectionData>> {
        let root_obj = self.get_root_object()?;

        let colls = root_obj
            .children
            .iter()
            .map(|child_hash| {
                let obj = self.read_object(child_hash)?;

                self.parse_collection_from_obj(&obj)
            })
            .collect::<Option<Vec<_>>>()?;

        Some(colls)
    }

    pub fn add_root_collection(&mut self, def: CollectionData) -> Result<()> {
        let mut new_objs_map = HashMap::new();
        let coll_obj = generate_object_for_coll(&def, &mut new_objs_map);

        for (_, obj) in new_objs_map.iter() {
            self.write_object(obj)?;
        }

        add_child_node(
            self,
            &[],
            &coll_obj,
            syng::tree_ops::ChildAdditionPosition::AddToEnd,
        )
        .expect("Write root collection failed");

        Ok(())
    }

    pub fn add_folder(&mut self, coll_path: &[usize], def: CollectionData) -> Result<()> {
        let (_, coll_obj_at_path) =
            get_object_at_path(self, coll_path).expect("Node at path extract failed");

        let add_pos = coll_obj_at_path
            .children
            .iter()
            .enumerate()
            .find_map(|(index, hash)| {
                let obj = self
                    .read_object(hash)
                    .expect("Folder point search hash failed");

                if obj.fields.get("type") == Some(&"request".to_owned()) {
                    Some(ChildAdditionPosition::AddAt(index))
                } else {
                    None
                }
            })
            .unwrap_or(ChildAdditionPosition::AddToEnd);

        let mut new_objs_map = HashMap::new();
        let coll_obj = generate_object_for_coll(&def, &mut new_objs_map);

        for (_, obj) in new_objs_map.iter() {
            self.write_object(obj)?;
        }

        add_child_node(self, coll_path, &coll_obj, add_pos).expect("Write folder failed");

        Ok(())
    }

    pub fn add_request(&mut self, path: &[usize], def: RequestData) -> Result<()> {
        let req_obj = generate_object_for_req(&def);

        add_child_node(self, path, &req_obj, ChildAdditionPosition::AddToEnd)
            .expect("Write request failed");

        Ok(())
    }

    pub fn delete_folder(&mut self, path: &[usize]) -> Result<()> {
        remove_child_node(self, path).expect("Delete folder failed");

        Ok(())
    }

    pub fn delete_request(&mut self, path: &[usize], req_index: usize) -> Result<()> {
        let (_, coll_obj_at_path) =
            get_object_at_path(self, path).expect("Node at path extract failed");

        let req_pos = coll_obj_at_path
            .children
            .iter()
            .enumerate()
            .find_map(|(index, hash)| {
                let obj = self
                    .read_object(hash)
                    .expect("Folder point search hash failed");

                if obj.fields.get("type") == Some(&"request".to_owned()) {
                    Some(index)
                } else {
                    None
                }
            })
            .expect("Cannot delete request if no requests inside collection");

        let remove_index = req_pos + req_index;

        remove_child_node(self, &[&path[..], &[remove_index]].concat())
            .expect("Remove request failed");

        Ok(())
    }
}
