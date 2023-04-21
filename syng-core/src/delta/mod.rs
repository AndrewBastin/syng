use std::collections::{BTreeSet, HashMap};

use serde::{Deserialize, Serialize};

use crate::{backend::SyngBackend, objects::SyngObjectDef, tree_ops::get_descendent_object_ids};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SyngDelta {
    pub start_point: Option<String>,
    pub new_root_node: String,
    pub new_objects: HashMap<String, SyngObjectDef>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ApplyDeltaError {
    /// The current tree in the backend drifted from the starting point in the delta
    CurrentTreeDrifted,

    /// The delta is missing 1 or more objects that it was referring to
    DeltaMissingObjects(Vec<String>),

    /// The root node specified in the delta is not a valid node in the object list
    DeltaNewRootNodeInvaid,
}

fn validate_delta(backend: &impl SyngBackend, delta: &SyngDelta) -> Result<(), ApplyDeltaError> {
    // Check if start point is the current root tree of the backend
    if backend.get_root_object_id() != delta.start_point {
        return Err(ApplyDeltaError::CurrentTreeDrifted);
    }

    // Check if new_root_node is in the new_objects list
    if !delta.new_objects.contains_key(&delta.new_root_node) {
        return Err(ApplyDeltaError::DeltaNewRootNodeInvaid);
    }

    // Check if all the objects on the tree properly resolve out
    // into valid nodes that exist
    let mut unresolved_nodes = vec![];

    for object in delta.new_objects.values() {
        for child_node_id in &object.children {
            // The object should either be resolvable by the backend or the delta
            if !backend.has_object(&child_node_id) && !delta.new_objects.contains_key(child_node_id)
            {
                unresolved_nodes.push(child_node_id.clone());
            }
        }
    }

    if unresolved_nodes.len() > 0 {
        return Err(ApplyDeltaError::DeltaMissingObjects(unresolved_nodes));
    }

    Ok(())
}

pub fn apply_delta(
    backend: &mut impl SyngBackend,
    delta: &SyngDelta,
) -> Result<(String, SyngObjectDef), ApplyDeltaError> {
    // Try validating and see if the delta actually makes sense for this backend
    validate_delta(backend, delta)?;

    for object in delta.new_objects.values() {
        backend
            .write_object(object)
            .expect("Failed writing object while delta is being resolved");
    }

    backend
        .set_root_object(&delta.new_root_node)
        .expect("Failed setting root object while the delta is being resolved");

    Ok((
        delta.new_root_node.clone(),
        delta.new_objects.get(&delta.new_root_node).unwrap().clone(),
    ))
}

pub fn generate_delta_from_point(
    backend: &impl SyngBackend,
    past_head_object_id: &str,
) -> Option<SyngDelta> {
    // Storing this into a BTreeSet so that we can binary search through the objects
    let past_tree_object_ids: BTreeSet<String> =
        get_descendent_object_ids(backend, past_head_object_id)?
            .into_iter()
            .collect();

    let current_head_id = backend.get_root_object_id()?;
    let current_tree_object_ids = get_descendent_object_ids(backend, &current_head_id)?;

    let new_objects = current_tree_object_ids
        .iter()
        .filter(|obj_id| !past_tree_object_ids.contains(*obj_id))
        .map(|obj_id| {
            // This unwrap is safe because we get it already from the tree
            let obj = backend.read_object(obj_id).unwrap();
            (obj_id.clone(), obj)
        })
        .collect::<HashMap<String, SyngObjectDef>>();

    Some(SyngDelta {
        start_point: Some(past_head_object_id.to_string()),
        new_root_node: current_head_id,
        new_objects,
    })
}
