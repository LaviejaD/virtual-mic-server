mod audio;
mod config;
mod websocket;

use anyhow::{Context, Result};
use log::{error, info, warn};
use tokio::net::TcpListener;

use config::get_lan_ip;
use websocket::handle_connection;

#[tokio::main]
async fn main() -> Result<()> {
    // Inicializar logger.
    env_logger::init();

    // Obtener IP local.
    let ip = get_lan_ip()?;
    info!("IP local detectada: {}", ip);

    // Intentar enlazar al puerto 3000. Si falla, usar puerto 0 (automático).
    let preferred_port: u16 = 3000;
    let listener = match TcpListener::bind((ip, preferred_port)).await {
        Ok(listener) => {
            info!("Escuchando en {}:{}", ip, preferred_port);
            listener
        }
        Err(e) => {
            warn!(
                "No se pudo enlazar al puerto {}: {}. Intentando con puerto automático.",
                preferred_port, e
            );
            // Usar puerto 0 para que el sistema asigne uno libre.
            let listener = TcpListener::bind((ip, 0))
                .await
                .context("No se pudo enlazar a ningún puerto")?;
            let addr = listener.local_addr()?;
            info!("Escuchando en {}:{} (puerto automático)", ip, addr.port());
            listener
        }
    };

    // Mostrar la dirección final.
    let local_addr = listener.local_addr()?;
    println!("===========================================");
    println!(" Servidor WebSocket de audio PCM iniciado ");
    println!(" Dirección: ws://{}:{}", local_addr.ip(), local_addr.port());
    println!("===========================================");

    // Aceptar conexiones entrantes en un bucle.
    loop {
        let (stream, peer) = listener.accept().await?;
        info!("Conexión TCP aceptada de {}", peer);
        // Manejar cada conexión en una tarea separada.

        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, peer).await {
                error!("Error manejando conexión de {}: {:?}", peer, e);
            }
        });
    }
}
