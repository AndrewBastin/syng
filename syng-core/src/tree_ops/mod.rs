use crate::{backend::SyngBackend, objects::SyngObjectDef};

pub enum ChildAdditionPosition {
    AddToEnd,
    AddAt(usize),
}

fn get_path_nodes_from_path(
    backend: &impl SyngBackend,
    node_path: &str,
) -> Option<Vec<(String, SyngObjectDef)>> {
    if node_path == "/" {
        Some(vec![(
            backend.get_root_object_id()?,
            backend.get_root_object()?,
        )])
    } else {
        node_path
            .split("/")
            .map(|id| Some((id.to_owned(), backend.read_object(id)?)))
            .collect::<Option<Vec<(String, SyngObjectDef)>>>()
    }
}

pub fn update_node(
    backend: &mut impl SyngBackend,
    node_path: &str,
    new_def: &SyngObjectDef,
) -> Option<(String, SyngObjectDef)> {
    let ancestor_nodes = get_path_nodes_from_path(backend, node_path)?;

    // Write the new object into the backend
    let hash = backend.write_object(&new_def).ok()?;

    // Go in reverse through all the parent nodes and update the tree
    let mut last_node_id = hash.clone();

    // Skip 1 because the last node is the actual node
    for (_, node) in ancestor_nodes.iter().rev().skip(1) {
        let mut new_node = node.clone();

        let index = new_node.children.iter().position(|node_id| {
            return node_id == &last_node_id;
        })?;

        new_node.children[index] = last_node_id;

        let new_hash = backend.write_object(&new_node).ok()?;

        last_node_id = new_hash;
    }

    // The last value of `last_node_id` will be the root id of the updated root node, so update the
    // root node for the backend
    backend.set_root_object(&last_node_id).ok()?;

    Some((hash, new_def.clone()))
}

pub fn add_child_node(
    backend: &mut impl SyngBackend,
    parent_node_path: &str,
    new_def: &SyngObjectDef,
    position: ChildAdditionPosition,
) -> Option<(String, SyngObjectDef)> {
    let ancestor_nodes = get_path_nodes_from_path(backend, parent_node_path)?;

    let hash = backend.write_object(&new_def).ok()?;

    let (_, direct_parent_node) = ancestor_nodes.last()?;

    let mut new_parent = direct_parent_node.clone();

    match position {
        ChildAdditionPosition::AddToEnd => new_parent.children.push(hash.clone()),
        ChildAdditionPosition::AddAt(index) if index < new_parent.children.len() => {
            new_parent.children.insert(index, hash.clone());
        }
        _ => return None, // AddAt with index > children length, that operation is not allowed
    };

    let mut last_parent_node_id = backend.write_object(&new_parent).ok()?;

    // Applying the changes to the entire tree
    for (_, node) in ancestor_nodes.iter().rev().skip(1) {
        let mut new_node = node.clone();

        let index = new_node
            .children
            .iter()
            .position(|node_id| node_id == &last_parent_node_id)?;

        new_node.children[index] = last_parent_node_id;

        let new_hash = backend.write_object(&new_node).ok()?;

        last_parent_node_id = new_hash;
    }

    // The last remaining value of `last_parent_node_id` will be the root id
    backend.set_root_object(&last_parent_node_id).ok()?;

    Some((hash, new_def.clone()))
}

pub fn remove_child_node(backend: &mut impl SyngBackend, node_path: &str) -> Option<()> {
    let ancestor_nodes = get_path_nodes_from_path(backend, node_path)?;

    // Since we are removing a child node, we have to make sure atleast 2 nodes are there in the
    // path (the root and the given node)
    let [
        remaining_ancestors @ ..,
        (_, parent_node),
        (to_delete_node_id, _)
    ] = ancestor_nodes.as_slice() else {
        return None;
    };

    let mut new_parent_node = parent_node.clone();

    let index = new_parent_node
        .children
        .iter()
        .position(|id| id == to_delete_node_id)?;

    new_parent_node.children.remove(index);

    let mut last_parent_node_id = backend.write_object(&new_parent_node).ok()?;

    // Applying the changes to the entire tree
    for (_, node) in remaining_ancestors.iter().rev() {
        let mut new_node = node.clone();

        let index = new_node
            .children
            .iter()
            .position(|node_id| node_id == &last_parent_node_id)?;

        new_node.children[index] = last_parent_node_id;

        let new_hash = backend.write_object(&new_node).ok()?;

        last_parent_node_id = new_hash;
    }

    // The last remaining value of `last_parent_node_id` will be the root id
    backend.set_root_object(&last_parent_node_id).ok()?;

    Some(())
}
