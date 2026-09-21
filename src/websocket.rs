//! Manejo de conexiones WebSocket.

use crate::audio::spawn_pacat;
use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use log::{error, info, warn};
use std::env::{self};
use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;

/// Maneja una conexión WebSocket individual.
///
/// Recibe mensajes binarios y los escribe en el stdin del proceso `pacat`.
/// Si la conexión se cierra o hay un error, termina el proceso hijo.
pub async fn handle_connection(stream: TcpStream, peer: SocketAddr) -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let password = match args.get(1) {
        Some(p) => p.clone(),
        None => "".to_string(),
    };

    info!("Nueva conexión desde: {}", peer);
    println!("new connection");
    // Aceptar el handshake WebSocket.
    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .context("Error durante el handshake WebSocket")?;
    info!("Conexión WebSocket establecida con {}", peer);

    // Lanzar pacat para esta conexión.
    let (mut child, mut child_stdin) = spawn_pacat().await?;
    info!("Proceso pacat lanzado (PID: {})", child.id().unwrap_or(0));

    // Dividir el stream WebSocket en sink y stream.
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Resultado del bucle: si hay error, se captura y se devuelve.
    let result = async {
        // Procesar mensajes entrantes.
        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(Message::Binary(data)) => {
                    // Escribir datos al stdin de pacat.
                    if let Err(e) = child_stdin.write_all(&data).await {
                        error!("Error escribiendo a pacat: {}", e);
                        return Err(e).context("Error escribiendo a pacat");
                    }
                }
                Ok(Message::Close(_)) | Err(_) => {
                    info!("Conexión cerrada por el cliente o error de red");
                    break;
                }
                Ok(Message::Text(text)) => {
                    if !password.is_empty() && text != password {
                        let _ = ws_sender.send(Message::Close(None)).await;
                        let _ = ws_sender.flush().await;
                        warn!("Cerrando conexion contraseña incorrecta: {}", text);
                        return Ok(());
                    }
                    // No esperamos texto, pero lo registramos.
                    warn!("Mensaje de texto inesperado: {:?}", text);
                }
                Ok(Message::Ping(payload)) => {
                    // Responder a los pings para mantener viva la conexión.
                    if let Err(e) = ws_sender.send(Message::Pong(payload)).await {
                        error!("Error enviando pong: {}", e);
                        break;
                    }
                }
                Ok(Message::Pong(_)) | Ok(Message::Frame(_)) => {
                    // Ignorar pongs y frames crudos.
                }
            }
        }
        Ok(())
    }
    .await;

    // Independientemente del resultado, cerrar stdin y matar el proceso pacat.
    info!("Cerrando stdin de pacat y terminando el proceso");
    drop(child_stdin); // Cierra el pipe.
    let _ = child.kill().await; // Intenta matar si aún está vivo.
    let _ = child.wait().await; // Espera a que termine.

    // Si el bucle terminó con error, propagarlo.
    result?;

    info!("Conexión con {} finalizada correctamente", peer);
    Ok(())
}
