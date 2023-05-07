#![allow(non_snake_case)]
use std::time::SystemTime;

use components::Collection;

use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder};
use sync::backend::DemoFEBackend;

use syng::{backend::SyngBackend, delta::apply_delta};
use syng_demo_common::{CollectionData, RequestData};

use crate::{
    remote::{pull_full_from_remote, push_to_remote},
    utils::{get_random_request_content, path_to_string},
};

mod components;
mod remote;
mod sync;
mod utils;

fn main() {
    hot_reload_init!();

    dioxus_desktop::launch_cfg(
        App,
        Config::default().with_window(WindowBuilder::default().with_title("Syng Demo")),
    );
}

#[derive(Clone)]
struct RemoteSyncLogItem {
    op: String,
    result: String,
}

fn App(cx: Scope) -> Element {
    let last_known_remote_root_id = use_state(cx, || -> Option<String> { None });
    let last_synced_remote_root_id = use_state(cx, || -> Option<String> { None });

    let remote_sync_log = use_ref(cx, || Vec::<RemoteSyncLogItem>::new());

    let backend = use_ref(cx, || DemoFEBackend::default());

    let is_repo_even_with_remote = {
        let backend_root_id = backend.read().get_root_object_id();
        let last_known_remote_root_id = last_known_remote_root_id.get();

        match (&backend_root_id, last_known_remote_root_id) {
            (Some(backend_root_id), Some(last_known_remote_root_id)) => {
                Some(backend_root_id == last_known_remote_root_id)
            }
            _ => None,
        }
    };

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
                                bk.drop_unreachable_objects(&*last_known_remote_root_id).expect("Drop failed");
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

                    "Last Known Remote ID: {*last_known_remote_root_id:?}"

                    br {}

                    "Last Synced Remote ID: {*last_synced_remote_root_id:?}"

                    br {}

                    button {
                        onclick: move |_| {
                            let lk_remote_root_id = last_known_remote_root_id.clone();

                            cx.spawn({
                                let log = remote_sync_log.to_owned();

                                async move {
                                    let result = pull_full_from_remote().await.expect("Pull failed");

                                    lk_remote_root_id.set(result.root_obj_id.clone());

                                    log.with_mut(|log| {
                                        log.push(RemoteSyncLogItem {
                                            op: "Pull Received (not applied)".to_owned(),
                                            result: serde_json::to_string_pretty(&result).unwrap()
                                        });
                                    });
                                }
                            });
                        },

                        "Get backend state (full pull; no apply)"
                    }

                    button {
                        onclick: move |_| {
                            let ls_remote_root_id = last_synced_remote_root_id.clone();
                            let lk_remote_root_id = last_synced_remote_root_id.clone();

                            cx.spawn({
                                let back = backend.to_owned();
                                let log = remote_sync_log.to_owned();

                                async move {
                                    let result = pull_full_from_remote().await.expect("Pull failed");

                                    back.with_mut(|bk| {
                                        bk.apply_full_pull(&result).expect("Pull write failed");

                                        let root_id = bk.get_root_object_id().clone();
                                        lk_remote_root_id.set(root_id.clone());
                                        ls_remote_root_id.set(root_id);
                                    });

                                    log.with_mut(|log| {
                                        log.push(RemoteSyncLogItem {
                                            op: "Pull Received".to_owned(),
                                            result: serde_json::to_string_pretty(&result).unwrap()
                                        });
                                    });
                                }
                            })
                        },

                        "Pull from Remote"
                    }

                    button {
                        onclick: move |_| {
                            let back = backend.clone();
                            let last_sync_point = last_synced_remote_root_id.clone();
                            let last_known_bk_point = last_known_remote_root_id.clone();
                            let log = remote_sync_log.to_owned();

                            let delta = backend.read().get_delta_for_pushing(&last_sync_point.get().clone().unwrap()).unwrap();

                            cx.spawn({
                                async move {
                                    push_to_remote(&delta).await.expect("Push to remote failed");

                                    log.with_mut(|lg| {
                                        lg.push(RemoteSyncLogItem {
                                            op: "Push to remote success".to_owned(),
                                            result: serde_json::to_string_pretty(&delta).unwrap()
                                        });
                                    });

                                    let curr_root = back.read().get_root_object_id();
                                    last_sync_point.set(curr_root.clone());
                                    last_known_bk_point.set(curr_root);
                                }
                            })
                        },

                        "Push to Remote"
                    }

                    button {
                        onclick: move |_| {
                            // Revert to last pull
                        },

                        "Revert to last pull"
                    }

                    br {}

                    button {
                        onclick: move |_| {
                            //
                        }
                    }


                    br {}
                    br {}

                    details {
                        summary { "Operation Log" }

                        for obj in remote_sync_log.read().iter().cloned() {
                            div {
                                "Operation: {obj.op}"
                                br {}
                                pre {
                                    obj.result
                                }
                            }
                        }
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
