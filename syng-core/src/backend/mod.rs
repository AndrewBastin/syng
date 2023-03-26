pub mod test_backend;

use crate::objects::SyngObjectDef;
use anyhow::Result;

pub trait SyngBackend {
    fn has_object(&self, object_id: &str) -> bool {
        self.read_object(object_id).is_some()
    }

    fn get_root_object_id(&self) -> Option<String>;
    fn get_root_object(&self) -> Option<SyngObjectDef>;
    fn set_root_object(&mut self, node_id: &str) -> Result<()>;

    fn read_object(&self, id: &str) -> Option<SyngObjectDef>;
    fn write_object(&mut self, def: &SyngObjectDef) -> Result<String>;
}
