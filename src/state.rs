use crate::events;
use crate::server;
use anyhow;
use tokio;
#[derive(Clone)]
pub struct Appstate {
    pub isrunning: bool,
    // serv: Arc<Mutex<Option<std::thread::JoinHandle<anyhow::Result<(), ()>>>>>,
    pub config: server::ServerConfig,
    pub mpsc: tokio::sync::mpsc::Sender<events::ServerEvents>,
}
