mod arguments;
mod audio;
mod config;
mod events;
mod server;
// mod state;
mod tls;
mod ui;
mod utils;
mod websocket;

use anyhow::Result;

fn main() -> Result<(), ()> {
    env_logger::init();
    if arguments::exist("-t".to_string()) {
        let mut config = server::setup().unwrap();
        if let Some(p) = arguments::get("-p".to_string()) {
            config.ping = p;
        }

        server::run(config, None).unwrap();
        return Ok(());
    }

    ui::run();

    Ok(())
}
