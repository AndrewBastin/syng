use anyhow::Result;
use std::collections::HashMap;

use crate::objects::SyngObjectDef;

use super::SyngBackend;

#[derive(Debug)]
pub struct SyngTestBackend {
    object_store: HashMap<String, SyngObjectDef>,
    current_root_object_id: Option<String>,
}

impl Default for SyngTestBackend {
    fn default() -> Self {
        let mut object_store = HashMap::new();

        let root_node = SyngObjectDef {
            fields: HashMap::new(),
            children: vec![],
        };

        let root_hash = root_node
            .get_hash()
            .expect("Hashing Root Node failed for SyngTestBackend");

        object_store.insert(root_hash.clone(), root_node);
        let current_root_object_id = Some(root_hash);

        SyngTestBackend {
            object_store,
            current_root_object_id,
        }
    }
}

impl SyngBackend for SyngTestBackend {
    fn has_object(&self, object_id: &str) -> bool {
        self.object_store.contains_key(object_id)
    }

    fn get_root_object_id(&self) -> Option<String> {
        self.current_root_object_id.clone()
    }

    fn get_root_object(&self) -> Option<SyngObjectDef> {
        if let Some(root_obj_id) = &self.current_root_object_id {
            self.read_object(&root_obj_id)
        } else {
            None
        }
    }

    fn set_root_object(&mut self, node_id: &str) -> Result<()> {
        self.current_root_object_id = Some(node_id.to_owned());

        Ok(())
    }

    fn read_object(&self, id: &str) -> Option<SyngObjectDef> {
        Some(self.object_store.get(&id.to_owned())?.clone())
    }

    fn write_object(&mut self, def: &SyngObjectDef) -> Result<String> {
        let hash = def.get_hash()?;

        if self.object_store.contains_key(&hash) {
            panic!(
                "Tried to write object which already (possibly) exists in the store, ID: {}",
                hash
            );
        }

        self.object_store.insert(hash.clone(), def.clone());

        Ok(hash)
    }
}
