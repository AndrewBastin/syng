use anyhow::Result;
use ciborium::ser::into_writer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SyngObjectDef {
    pub fields: HashMap<String, String>,
    pub children: Vec<String>,
}

impl SyngObjectDef {
    pub fn get_hash(&self) -> Result<String> {
        let mut data_sink = Vec::<u8>::new();

        into_writer(&self, &mut data_sink)?;

        let hash = sha256::digest(data_sink.as_slice());

        Ok(hash)
    }
}

pub enum SyngObjectOp {
    FieldRemove {
        object_path: String,
        key: String,
    },

    FieldSet {
        object_path: String,
        key: String,
        new_value: String,
    },

    ChildAdd {
        object_path: String,
        child_object_id: String,
    },

    ChildRemove {
        object_path: String,
        child_object_id: String,
    },

    ChildReplace {
        object_path: String,
        child_object_id: String,
        new_child_object_id: String,
    },
}
