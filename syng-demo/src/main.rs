use std::collections::HashMap;
use ciborium::ser::into_writer;

use syng::backend::test_backend::SyngTestBackend;
use syng::backend::SyngBackend;
use syng::objects::SyngObjectDef;
use syng::delta::{generate_delta_from_point, apply_delta};
use syng::tree_ops::{add_child_node, ChildAdditionPosition};

fn main() {
    // Write a test program that uses the SyngTestBackend to generate a tree and generate delta between two tree states
    let mut backend = SyngTestBackend::default();

    let initial_root_node_id = backend.get_root_object_id().unwrap();

    println!("Initial backend:\n{:?}\n", backend);
    
    let mut test_map = HashMap::new();
    test_map.insert("name".to_owned(), "Andrew".to_owned());

    let new_test_node = SyngObjectDef {
        fields: test_map,
        children: vec![],
    };

    add_child_node(&mut backend, "/", &new_test_node, ChildAdditionPosition::AddToEnd);


    println!("Backend after node change:\n{:?}\n", backend);

    let delta = generate_delta_from_point(&backend, &initial_root_node_id).unwrap();
    println!("Generated delta:\n{:?}\n", delta);

    let mut backend2 = SyngTestBackend::default();
    
    println!("New Backend:\n{:?}\n", backend2);

    println!("Delta Application on Test:\n{:?}\n", apply_delta(&mut backend2, &delta));

    println!("Backend after delta application:\n{:?}\n", backend2);

    let root_obj = backend.get_root_object().unwrap();
    let mut cbor_vec = Vec::<u8>::new();

    into_writer(&root_obj, &mut cbor_vec).unwrap();

    println!("CBOR Hex for root obj (len: {}):", cbor_vec.len());
    for byte in cbor_vec.iter() {
        print!("{:02X} ", byte);
    }

    println!("\n");

    let mut cbor_vec_delta = Vec::<u8>::new();

    into_writer(&delta, &mut cbor_vec_delta).unwrap();

    println!("CBOR Hex for delta (len: {}):", cbor_vec_delta.len());
    for byte in cbor_vec_delta.iter() {
        print!("{:02X} ", byte);
    }

    println!();
}
