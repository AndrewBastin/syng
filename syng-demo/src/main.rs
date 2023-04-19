#![allow(non_snake_case)]
use std::time::SystemTime;

use components::Collection;
use data::CollectionData;

use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder};
use sync::ObjectGen;

use crate::{
    data::RequestData,
    utils::{get_collection_mut, get_random_request_content, path_to_string},
};

mod components;
mod data;
mod sync;
mod utils;

fn main() {
    hot_reload_init!();

    dioxus_desktop::launch_cfg(
        App,
        Config::default().with_window(WindowBuilder::default().with_title("Syng Demo")),
    );
}

fn App(cx: Scope) -> Element {
    let root_collections = use_state(cx, || Vec::<CollectionData>::new());

    // Get current system time
    let gen_start_time = SystemTime::now();

    let gen_object = ObjectGen::from(root_collections.get());

    let gen_end_time = SystemTime::now();

    let gen_time_ms = gen_end_time
        .duration_since(gen_start_time)
        .unwrap()
        .as_millis();

    let gen_obj_string = serde_json::to_string_pretty(&gen_object).unwrap();

    cx.render(rsx! {
        div {
            style { include_str!("./style.css") }

            div {
                class: "debug-panel",

                h3 { "Debug Info" }

                p { "Tree Gen took {gen_time_ms}ms" }

                pre {
                    "{gen_obj_string}"
                }
            }

            div {
                class: "colls-panel",

                button {
                    onclick: move |_| {
                        (*root_collections).make_mut().push(CollectionData::new(format!("Collection {}", root_collections.len())));
                    },
                    "Add Root Collection"
                }

                for (index, coll) in root_collections.iter().enumerate() {
                    Collection {
                        path: vec![index],
                        coll: coll,
                        on_add_folder: move |path| {
                            let mut root_colls_ref = root_collections.make_mut();
                            let coll = get_collection_mut(&mut *root_colls_ref, &path).unwrap();

                            coll.folders.push(CollectionData::new(format!("Subfolder {}/{}", path_to_string(&path), coll.folders.len())));
                        },
                        on_add_request: move |path| {
                            let mut root_colls_ref = root_collections.make_mut();
                            let coll = get_collection_mut(&mut *root_colls_ref, &path).unwrap();

                            coll.requests.push(RequestData {
                                title: format!("Request {}/[{}]", path_to_string(&path), coll.requests.len()),
                                content: get_random_request_content(),
                            });
                        },
                        on_delete_folder: move |path: Vec<usize>| {
                            let mut root_colls_ref = root_collections.make_mut();

                            if path.len() == 1 {
                                (*root_colls_ref).remove(path.last().unwrap().clone());

                                return;
                            }

                            let (containing_path_slice, index_slice) = path.split_at(path.len() - 1);

                            let containing_coll = get_collection_mut(&mut *root_colls_ref, &Vec::from(containing_path_slice)).unwrap();
                            let index = index_slice.first().unwrap().clone();

                            containing_coll.folders.remove(index);
                        },
                        on_delete_request: move |(path, index)| {
                            let mut root_colls_ref = root_collections.make_mut();
                            let coll = get_collection_mut(&mut *root_colls_ref, &path).unwrap();

                            coll.requests.remove(index);
                        }
                    }
                }
            }
        }
    })
}
