use dioxus::{html::button, prelude::*};

use crate::{
    sync::backend::DemoFEBackend,
    utils::{get_sync_status, DiffGenResult},
};

#[derive(Props)]
pub struct SyncStateDialogProps<'a> {
    show: bool,
    backend: DemoFEBackend,

    #[props(!optional)]
    last_synced_point: &'a Option<String>,

    on_close: EventHandler<'a>,
}

pub fn SyncStateDialog<'a>(cx: Scope<'a, SyncStateDialogProps<'a>>) -> Element<'a> {
    let loading_sync_state = use_state(cx, || false);
    let known_sync_state = use_state(cx, || -> Option<DiffGenResult> { None });

    cx.render(rsx! {
        div {
            class: "dialog-backdrop",
            hidden: if cx.props.show { "false" } else { "true" }
        }

        dialog {
            open: if cx.props.show { "true" } else { "false" },

            div {
                h1 { "Sync State" }

                button {
                    onclick: move |_| {
                        let last_sync_point = cx.props.last_synced_point.to_owned().unwrap();
                        let backend = cx.props.backend.to_owned();

                        cx.spawn(async move {
                            loading_sync_state.set(true);

                            let data = get_sync_status(last_sync_point, &backend).await;

                            loading_sync_state.set(false);
                        });
                    }
                }

                button {
                    onclick: move |_| {
                        cx.props.on_close.call(());
                    },

                    "Close"
                }
            }
        }
    })
}
