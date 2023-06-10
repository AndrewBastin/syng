#![allow(non_snake_case)]
use dioxus::prelude::*;
use syng_demo_common::{CollectionData, RequestData};

use crate::utils::path_to_string;

pub mod dialogs;

#[derive(Props)]
pub struct CollectionProps<'a> {
    path: Vec<usize>,
    coll: CollectionData,
    on_add_folder: EventHandler<'a, Vec<usize>>,
    on_delete_folder: EventHandler<'a, Vec<usize>>,
    on_add_request: EventHandler<'a, Vec<usize>>,
    on_delete_request: EventHandler<'a, (Vec<usize>, usize)>,
    on_move_folder: EventHandler<'a, Vec<usize>>, // (from obj path, to obj path)

    // (from obj path, from req index, to obj path, to req index)
    on_move_request: EventHandler<'a, (Vec<usize>, usize)>,
}

pub fn Collection<'a>(cx: Scope<'a, CollectionProps<'a>>) -> Element<'a> {
    cx.render(rsx! {
        div {
            span {
                "{cx.props.coll.title} (Path: {path_to_string(&cx.props.path)})"
            }

            button {
                onclick: move |_| cx.props.on_add_folder.call(cx.props.path.clone()),

                "Add Folder"
            }

            button {
                onclick: move |_| cx.props.on_add_request.call(cx.props.path.clone()),

                "Add Request"
            }

            button {
                onclick: move|_| cx.props.on_delete_folder.call(cx.props.path.clone()),

                "Delete"
            }

            button {
                onclick: move |_| cx.props.on_move_folder.call(cx.props.path.clone()),

                "Move"
            }

            div {
                style: r#"
                    padding-left: 10px;
                "#,

                for (i, folder) in cx.props.coll.folders.iter().enumerate() {
                    Collection {
                        path: [cx.props.path.clone(), vec![i]].concat(),
                        coll: folder.clone(),
                        on_add_folder: move |path| {
                            cx.props.on_add_folder.call(path);
                        },
                        on_add_request: move |path| {
                            cx.props.on_add_request.call(path);
                        },
                        on_delete_folder: move |path| {
                            cx.props.on_delete_folder.call(path);
                        },
                        on_delete_request: move |req_path| {
                            cx.props.on_delete_request.call(req_path);
                        },
                        on_move_folder: move |path| {
                            cx.props.on_move_folder.call(path);
                        },
                        on_move_request: move |path| {
                            cx.props.on_move_request.call(path);
                        }
                    }
                }

                for (i, req) in cx.props.coll.requests.iter().enumerate() {
                    Request {
                        path: (cx.props.path.clone(), i),
                        req: req,
                        on_delete_request: move |req_path| {
                            cx.props.on_delete_request.call(req_path);
                        },
                        on_move_request: move |path| {
                            cx.props.on_move_request.call(path);
                        }
                    }
                }
            }
        }
    })
}

#[derive(Props)]
pub struct RequestProps<'a> {
    path: (Vec<usize>, usize),
    req: &'a RequestData,
    on_delete_request: EventHandler<'a, (Vec<usize>, usize)>,

    // (from obj path, from req index, to obj path, to req index)
    on_move_request: EventHandler<'a, (Vec<usize>, usize)>,
}

pub fn Request<'a>(cx: Scope<'a, RequestProps>) -> Element<'a> {
    cx.render(rsx! {
        div {
            span {
                "{cx.props.req.title} (Path: {path_to_string(&cx.props.path.0)} req: {cx.props.path.1}) [content: {cx.props.req.content}]"
            }

            button {
                onclick: move |_| {
                    cx.props.on_move_request.call(cx.props.path.clone());
                },

                "Move"
            }
            button {
                onclick: move |_| cx.props.on_delete_request.call(cx.props.path.clone()),

                "Delete"
            }
        }
    })
}
