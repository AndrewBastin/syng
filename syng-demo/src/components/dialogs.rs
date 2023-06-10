use dioxus::prelude::*;

#[derive(Props)]
pub struct PromptDialogProps<'a> {
    show: bool,
    dialog_title: &'a str,
    message: &'a str,
    placeholder: &'a str,

    on_ok: EventHandler<'a, String>,
    on_cancel: EventHandler<'a>,
}

pub fn PromptDialog<'a>(cx: Scope<'a, PromptDialogProps<'a>>) -> Element<'a> {
    let input_value = use_state(cx, || "".to_string());

    cx.render(rsx! {
        div {
            class: "dialog-backdrop",
            hidden: if cx.props.show { "false" } else { "true" }
        }

        dialog {
            open: if cx.props.show { "true" } else { "false" },

            h3 {
                "{cx.props.dialog_title}"
            }

            p {
                "{cx.props.message}"
            }

            input {
                value: "{input_value}",
                placeholder: cx.props.placeholder,

                oninput: move |evt| input_value.set(evt.value.clone()),
            }

            button {
                onclick: move |_| {
                    cx.props.on_ok.call(input_value.to_string());
                    input_value.set("".to_string());
                },
                "OK"
            }

            button {
                onclick: move |_| {
                    cx.props.on_cancel.call(());
                    input_value.set("".to_string());
                },

                "Cancel"
            }
        }
    })
}
