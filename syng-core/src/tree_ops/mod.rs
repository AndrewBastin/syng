use crate::{backend::SyngBackend, objects::SyngObjectDef};

pub enum ChildAdditionPosition {
    AddToEnd,
    AddAt(usize),
}

pub fn get_object_at_path(
    backend: &impl SyngBackend,
    obj_path: &[usize],
) -> Option<(String, SyngObjectDef)> {
    if obj_path.is_empty() {
        Some((backend.get_root_object_id()?, backend.get_root_object()?))
    } else {
        let mut last_obj = (backend.get_root_object_id()?, backend.get_root_object()?);

        for index in obj_path {
            let curr_obj = &last_obj.1;

            let index_obj_id = curr_obj.children.get(index.clone())?;

            last_obj = (index_obj_id.clone(), backend.read_object(index_obj_id)?);
        }

        Some(last_obj)
    }
}

fn get_objects_along_index_path(
    backend: &impl SyngBackend,
    obj_path: &[usize],
) -> Option<Vec<(String, SyngObjectDef)>> {
    if obj_path.is_empty() {
        Some(vec![(
            backend.get_root_object_id()?,
            backend.get_root_object()?,
        )])
    } else {
        let mut objs = Vec::with_capacity(obj_path.len() + 1);

        objs.push((backend.get_root_object_id()?, backend.get_root_object()?));

        for index in obj_path {
            let curr_obj = &objs.last()?.1;

            let index_obj_id = curr_obj.children.get(index.clone())?;

            objs.push((index_obj_id.clone(), backend.read_object(index_obj_id)?));
        }

        Some(objs)
    }
}

pub fn get_descendent_object_ids(backend: &impl SyngBackend, id: &str) -> Option<Vec<String>> {
    let mut result = vec![];

    // Iterate through the tree using a queue in place of recursion
    let mut search_queue = vec![id.to_owned()];

    while let Some(object_id) = search_queue.pop() {
        let obj = backend.read_object(&object_id)?;
        result.push(object_id);

        search_queue.extend(obj.children);
    }

    Some(result)
}

pub fn get_descendent_objects(backend: &impl SyngBackend, id: &str) -> Option<Vec<SyngObjectDef>> {
    let mut result = vec![];

    // Iterate through the tree using a queue in place of recursion
    let mut search_queue = vec![id.to_owned()];

    while let Some(object_id) = search_queue.pop() {
        let obj = backend.read_object(&object_id)?;
        result.push(obj.clone());

        search_queue.extend(obj.children);
    }

    Some(result)
}

pub fn update_object(
    backend: &mut impl SyngBackend,
    obj_path: &[usize],
    new_def: &SyngObjectDef,
) -> Option<(String, SyngObjectDef)> {
    let ancestor_objs = get_objects_along_index_path(backend, obj_path)?;

    // Write the new object into the backend
    let hash = backend.write_object(&new_def).ok()?;

    // Go in reverse through all the parent objects and update the tree
    let mut last_obj_id = hash.clone();

    // Skip 1 because the last object is the actual object
    for (index, (_, obj)) in ancestor_objs.iter().enumerate().rev().skip(1) {
        let mut new_obj = obj.clone();

        new_obj.children[obj_path[index]] = last_obj_id;

        let new_hash = backend.write_object(&new_obj).ok()?;

        last_obj_id = new_hash;
    }

    // The last value of `last_obj_id` will be the root id of the updated root obj, so update the
    // root obj for the backend
    backend.set_root_object(&last_obj_id).ok()?;

    Some((hash, new_def.clone()))
}

pub fn add_child_object(
    backend: &mut impl SyngBackend,
    parent_obj_path: &[usize],
    new_def: &SyngObjectDef,
    position: ChildAdditionPosition,
) -> Option<(String, SyngObjectDef)> {
    let ancestor_objs = get_objects_along_index_path(backend, parent_obj_path)?;

    let hash = backend.write_object(&new_def).ok()?;

    let (_, direct_parent_obj) = ancestor_objs.last()?;

    let mut new_parent = direct_parent_obj.clone();

    match position {
        ChildAdditionPosition::AddToEnd => new_parent.children.push(hash.clone()),
        ChildAdditionPosition::AddAt(index) if index < new_parent.children.len() => {
            new_parent.children.insert(index, hash.clone());
        }
        _ => return None, // AddAt with index > children length, that operation is not allowed
    };

    let mut last_parent_obj_id = backend.write_object(&new_parent).ok()?;

    // Applying the changes to the entire tree
    for (index, (_, obj)) in ancestor_objs.iter().enumerate().rev().skip(1) {
        let mut new_obj = obj.clone();

        let obj_index = parent_obj_path[index];

        new_obj.children[obj_index] = last_parent_obj_id;

        let new_hash = backend.write_object(&new_obj).ok()?;

        last_parent_obj_id = new_hash;
    }

    // The last remaining value of `last_parent_obj_id` will be the root id
    backend.set_root_object(&last_parent_obj_id).ok()?;

    Some((hash, new_def.clone()))
}

pub fn remove_child_object(backend: &mut impl SyngBackend, obj_path: &[usize]) -> Option<()> {
    let ancestor_objs = get_objects_along_index_path(backend, obj_path)?;

    // Since we are removing a child object, we have to make sure atleast 2 objects are there in the
    // path (the root and the given object)
    let [
        remaining_ancestors @ ..,
        (_, parent_obj),
        _
    ] = ancestor_objs.as_slice() else {
        return None;
    };

    let mut new_parent_obj = parent_obj.clone();

    let delete_index = obj_path.last().unwrap().clone();

    new_parent_obj.children.remove(delete_index);

    let mut last_parent_obj_id = backend.write_object(&new_parent_obj).ok()?;

    // Applying the changes to the entire tree
    for (index, (_, obj)) in remaining_ancestors.iter().enumerate().rev() {
        let mut new_obj = obj.clone();

        let update_index = obj_path[index];

        new_obj.children[update_index] = last_parent_obj_id;

        let new_hash = backend.write_object(&new_obj).ok()?;

        last_parent_obj_id = new_hash;
    }

    // The last remaining value of `last_parent_obj_id` will be the root id
    backend.set_root_object(&last_parent_obj_id).ok()?;

    Some(())
}
