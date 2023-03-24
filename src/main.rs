use backend::test_backend::SyngTestBackend;

use crate::{backend::SyngBackend, tree_ops::update_node};

mod backend;
mod objects;
mod tree_ops;

fn main() {
    let mut test_backend = SyngTestBackend::default();

    println!("Initial status: {:?}", test_backend);

    let mut new_root_obj = test_backend.get_root_object().unwrap().clone();

    new_root_obj
        .fields
        .insert("name".to_owned(), "Andrew".to_owned());

    let result = update_node(&mut test_backend, "/", &new_root_obj);

    println!("Op status: {:?}", result);
    println!("{:?}", test_backend);
}
