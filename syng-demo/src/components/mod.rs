#![allow(non_snake_case)]
use dioxus::prelude::*;

use crate::data::{CollectionData, RequestData};
use crate::utils::path_to_string;

#[derive(Props)]
pub struct CollectionProps<'a> {
    path: Vec<usize>,
    coll: &'a CollectionData,
    on_add_folder: EventHandler<'a, Vec<usize>>,
    on_delete_folder: EventHandler<'a, Vec<usize>>,
    on_add_request: EventHandler<'a, Vec<usize>>,
    on_delete_request: EventHandler<'a, (Vec<usize>, usize)>,
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

            div {
                style: r#"
                    margin-left: 10px;
                "#,

                for (i, folder) in cx.props.coll.folders.iter().enumerate() {
                    Collection {
                        path: [cx.props.path.clone(), vec![i]].concat(),
                        coll: folder,
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
                        }
                    }
                }

                for (i, req) in cx.props.coll.requests.iter().enumerate() {
                    Request {
                        path: (cx.props.path.clone(), i),
                        req: req,
                        on_delete_request: move |req_path| {
                            cx.props.on_delete_request.call(req_path);
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
}

pub fn Request<'a>(cx: Scope<'a, RequestProps>) -> Element<'a> {
    cx.render(rsx! {
        div {
            span {
                "{cx.props.req.title} (Path: {path_to_string(&cx.props.path.0)} req: {cx.props.path.1}) [content: {cx.props.req.content}]"
            }

            button {
                onclick: move |_| cx.props.on_delete_request.call(cx.props.path.clone()),

                "Delete"
            }
        }
    })
}
