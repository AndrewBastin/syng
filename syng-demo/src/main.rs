#![allow(non_snake_case)]
use std::time::SystemTime;

use components::Collection;

use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder};
use sync::backend::DemoFEBackend;

use syng_demo_common::{CollectionData, RequestData};

use crate::utils::{get_random_request_content, path_to_string};

mod components;
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
    let backend = use_ref(cx, || DemoFEBackend::default());

    let gen_tree_start = SystemTime::now();

    let root_colls = backend
        .read()
        .get_collection_tree()
        .expect("Collection Tree Parse Failed");

    let gen_tree_end = SystemTime::now();
    let gen_tree_duration = gen_tree_end
        .duration_since(gen_tree_start)
        .unwrap()
        .as_millis();

    let tree_info = backend
        .read()
        .generate_gen_info()
        .expect("Failed generating tree gen info");

    let tree_info_str =
        serde_json::to_string_pretty(&tree_info).expect("Failed converting tree gen info to JSON");

    let root_colls_len = root_colls.len();

    let root_colls_tree = root_colls.iter().enumerate().map(|(index, coll)| {
        rsx! {
            Collection {
                path: vec![index],
                coll: coll.clone(),
                on_add_folder: move |path: Vec<usize>| {
                    let coll = backend.read().get_collection(&path).unwrap();

                    let data = CollectionData::new(format!("Subfolder {}/{}", path_to_string(&path), coll.folders.len()));
                    backend.write().add_folder(&path, data).expect("Add Folder failed");
                },
                on_add_request: move |path: Vec<usize>| {
                    let coll = backend.read().get_collection(&path).unwrap();

                    let data = RequestData {
                        title: format!("Request {}/[{}]", path_to_string(&path), coll.requests.len()),
                        content: get_random_request_content(),
                    };

                    backend.write().add_request(&path, data).expect("Add request Failed");
                },
                on_delete_folder: move |path: Vec<usize>| {
                    backend.write().delete_folder(&path).expect("Delete folder failed");
                },
                on_delete_request: move |(path, index): (Vec<usize>, usize)| {
                    backend.write().delete_request(&path, index).expect("Delete request failed");
                }
            }
        }
    });

    cx.render(rsx! {
        div {
            style { include_str!("./style.css") }

            div {
                class: "debug-panel",

                details {
                    summary {
                        "Local Repo Info"
                    }

                    "Tree gen took {gen_tree_duration}ms"

                    br {}

                    button {
                        onclick: move |_| {
                            backend.with_mut(|bk| {
                                bk.drop_unreachable_objects().expect("Drop failed");
                            })
                        },

                        "Drop Unreachable"
                    }

                    br {}

                    "Local repo data:"
                    pre {
                        tree_info_str
                    }
                }

                br {}
                br {}
                br {}

                details {
                    summary {
                        "Remote Repo Info"
                    }

                    button {
                        onclick: move |_| {
                            // Try pull remote changes
                        },

                        "Pull from Remote"
                    }

                    button {
                        onclick: move |_| {
                            // Try push to remote
                        },

                        "Push to Remote"
                    }

                    button {
                        onclick: move |_| {
                            // Revert to last pull
                        },

                        "Revert to last pull"
                    }
                }
            }

            div {
                class: "colls-panel",

                button {
                    onclick: move |_| {
                        backend.with_mut(|bk| {
                            bk.add_root_collection(CollectionData::new(format!("Collection {}", root_colls_len)))
                                .expect("Add root collection failed");
                        });
                    },
                    "Add Root Collection"
                }

                root_colls_tree
            }
    }})
}
