use crate::events;
use crate::server;
use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;
static CSS: Asset = asset!("/assets/main.css");

pub fn run() {
    dioxus::LaunchBuilder::new()
        .with_cfg(
            Config::default().with_window(
                WindowBuilder::new()
                    .with_title("Virtual Mic")
                    .with_always_on_top(false),
            ),
        )
        .launch(App);
}

#[component]
fn App() -> Element {
    let mut config = use_hook(|| server::setup().unwrap());
    println!("{:#?}", config);
    let sender = use_hook(|| {
        let (_handle, sender) = server::start(config.clone()).unwrap();
        sender
    });

    let tr = use_signal(|| sender);
    let mut running = use_signal(|| false);

    let mut ping = use_signal(|| config.ping.clone());
    let ip = config.ip.to_string();
    let port = config.port;

    let toggle = move |_| {
        if ping.to_string() != config.ping.clone() {
            config.ping = ping.to_string();
            let _ = tr().try_send(events::ServerEvents::Pin(config.ping.clone()));
        }
        if running() {
            let _ = tr().try_send(events::ServerEvents::Pause);
            running.set(false);
        } else {
            let _ = tr().try_send(events::ServerEvents::Start);
            running.set(true);
        }
    };

    // Colores / textos según el estado
    let (dot_color, status_text, button_text, button_bg) = if running() {
        ("#22c55e", "Activo", "Detener", "#dc2626")
    } else {
        ("#64748b", "Detenido", "Iniciar", "#16a34a")
    };

    rsx! {
        document::Stylesheet { href: CSS }
        div {
            style: "
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
                background: #0f172a;
                color: #e2e8f0;
                padding: 28px 24px;
                min-height: 100vh;
                box-sizing: border-box;
                display: flex;
                flex-direction: column;
                gap: 18px;
            ",

            // Título
            h1 {
                style: "margin: 0; font-size: 22px; font-weight: 600; text-align: center; letter-spacing: 0.3px;",
                "Virtual Mic"
            }

            // Estado
            div {
                style: "
                    display: flex; align-items: center; justify-content: center;
                    gap: 10px; padding: 12px;
                    background: #1e293b; border-radius: 12px;
                ",
                span {
                    style: "
                        width: 10px; height: 10px; border-radius: 50%;
                        background: {dot_color};
                        box-shadow: 0 0 10px {dot_color};
                        transition: all 0.2s;
                    "
                }
                span {
                    style: "font-size: 15px; font-weight: 500;",
                    "{status_text}"
                }
            }

            // PIN
            div {
                style: "
                    background: #1e293b; border-radius: 12px;
                    padding: 16px 18px; text-align: center;
                ",
                div {
                    style: "
                        font-size: 11px; text-transform: uppercase;
                        letter-spacing: 1.5px; color: #94a3b8;
                        margin-bottom: 8px;
                    ",
                    "PIN"
                }
                input { oninput: move |event| { ping.set(event.data().value() )},
                    type: "text",
                    style: "
                        font-family: 'SF Mono', 'Consolas', monospace;
                        font-size: 32px; letter-spacing: 8px;
                        font-weight: 600; color: #38bdf8;
                        padding-left: 8px;
                    ",
                    value:"{ping}"
                }
            }

            // IP
            div {
                style: "background: #1e293b; border-radius: 12px; padding: 14px 18px;",
                div {
                    style: "
                        font-size: 11px; text-transform: uppercase;
                        letter-spacing: 1.5px; color: #94a3b8;
                        margin-bottom: 6px;
                    ",
                    "Dirección"
                }
                div {
                    style: "
                        font-family: 'SF Mono', 'Consolas', monospace;
                        font-size: 13px; color: #cbd5e1;
                        word-break: break-all;
                    ",
                    "wss://{ip}:{port}"
                }
            }

            // Espaciador para empujar el botón abajo
            div { style: "flex: 1;" }

            // Botón
            button {
                onclick: toggle,
                style: "
                    padding: 14px;
                    font-size: 15px; font-weight: 600;
                    color: white;
                    background: {button_bg};
                    border: none; border-radius: 12px;
                    cursor: pointer;
                    transition: transform 0.08s;
                ",
                "{button_text}"
            }
        }
    }
}
