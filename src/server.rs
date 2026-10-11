use crate::{
    arguments,
    config::{PREFERRED_PORT, get_lan_ip},
    events, tls, utils,
    websocket::handle_connection,
};
use anyhow::Result;
use log::{error, info, warn};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::TlsAcceptor;

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub ping: String,
    pub ip: IpAddr,
    pub port: u16,
}
/// Esta funcion se llama por defecto cuando se inicia la aplicacion
/// carga las configuraciones y permite ser un puente entre dioxus y el hilo del servidor
pub fn start(
    config: ServerConfig,
) -> Result<
    (
        std::thread::JoinHandle<Result<(), ()>>,
        tokio::sync::mpsc::Sender<events::ServerEvents>,
    ),
    (),
> {
    let (tx, rx) = tokio::sync::mpsc::channel::<events::ServerEvents>(32);
    Ok((std::thread::spawn(move || run(config, Some(rx))), tx))
}
/// Esta funcion crea una configuracion por defecto para el servidor
pub fn setup() -> Result<ServerConfig, ()> {
    let ip = match get_lan_ip() {
        Ok(r) => {
            info!("IP local detectada: {}", r);
            r
        }
        Err(_) => {
            error!("Error en optener la ip local");
            return Err(());
        }
    };

    let port = match std::net::TcpListener::bind((ip, PREFERRED_PORT)) {
        Ok(l) => {
            drop(l);
            PREFERRED_PORT
        }

        Err(_) => std::net::TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port(),
    };
    Ok(ServerConfig {
        ping: utils::generate_pin(6),
        ip,
        port,
    })
}
/// Inicia el servidor
/// config: son las configuraciones basicas para el servidor
/// events: esto es opcional para poder elejir si el servidor acepta eventos del frontend o no
pub fn run(
    mut config: ServerConfig,
    events: Option<tokio::sync::mpsc::Receiver<events::ServerEvents>>,
) -> Result<(), ()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("No se pudo inicializar el Runtime de Tokio");
    rt.block_on(async move {
        // Configuración TLS (carga o genera certificado).
        let tls_config = match tls::load_or_generate_tls_config(&config.ip.to_string()) {
            Ok(r) => r,
            Err(_) => {
                error!("Error en cargar/crear, los certificados.");
                return ();
            }
        };
        let tls_acceptor = TlsAcceptor::from(Arc::clone(&tls_config));

        // Enlazar.
        let listener = match TcpListener::bind((config.ip, config.port)).await {
            Ok(listener) => {
                info!("Escuchando en {}:{}", config.ip, config.port);
                listener
            }
            Err(e) => {
                warn!(
                    "No se pudo enlazar al puerto {}: {}. Intentando con puerto automático.",
                    config.port, e
                );
                let listener = match TcpListener::bind((config.ip, 0)).await {
                    Ok(r) => r,
                    Err(_) => {
                        error!("TcpListener no se pudo enlzar a ningun puerto");
                        return ();
                    }
                };
                let addr = match listener.local_addr() {
                    Ok(r) => r,
                    Err(_) => {
                        error!("Error en obtener la direccion local.");
                        return ();
                    }
                };
                info!(
                    "Escuchando en {}:{} (puerto automático)",
                    config.ip,
                    addr.port()
                );
                listener
            }
        };

        let local_addr = match listener.local_addr() {
            Ok(r) => r,
            Err(_) => {
                error!("Error en obtener la ip local.");
                return ();
            }
        };
        println!("===========================================");
        println!("Servidor WebSocket WSS (TLS) iniciado     ");
        println!("Dirección: wss://{}:{}", local_addr.ip(), local_addr.port());
        if events.is_some() || arguments::exist("-p".to_string()) {
            println!("Ping: {}", config.ping)
        }
        println!("===========================================");
        // Esto maneja los eventos provenientes del la parte de dioxus
        if let Some(mut receiver) = events {
            let mut lsr = false;
            let mut joinhandle: Vec<tokio::task::JoinHandle<()>> = vec![];
            loop {
                tokio::select! {
                event = receiver.recv() => {
                    match event {
                            Some(events::ServerEvents::Start) => {
                                lsr = true;
                                },
                            Some(events::ServerEvents::Pause) => {

                                lsr = false;
                                for j in &joinhandle {
                                j.abort();
                                };
                                #[allow(unused_must_use)]
                                &joinhandle.clear();
                                },
                                Some(events::ServerEvents::Pin(e))=> config.ping = e ,
                            Some(events::ServerEvents::Stop) => {
                                break
                                },
                            None => {
                            info!("Canal de eventos cerrado, terminando servidor");
                            break;
                            }
                                }
                            }

                result =  listener.accept(), if lsr  => {
                            match result {
                                Ok((tcp_stream,peer )) => {
                                    joinhandle.push(
                                lister(tls_acceptor.clone(),
                                    tcp_stream,
                                    peer.clone(),
                                    config.clone()
                                ));
                            },
                            Err(_) => error!("error en escuchar la nose :D")
                            }
                        }
                }
            }
            return ();
        }
        // Solo llega aqui si el servidor se inicia con "-t" para evitar cargar la parte visual
        loop {
            match listener.accept().await {
                Ok((tcp_stream, peer)) => {
                    lister(
                        tls_acceptor.clone(),
                        tcp_stream,
                        peer.clone(),
                        config.clone(),
                    );
                }
                Err(_) => error!("error en escuchar la nose :D"),
            }
        }
    });
    Ok(())
}
/// Espera una conexion aceptada para usarla en el tls
fn lister(
    tls_acceptor: TlsAcceptor,
    tcp_stream: TcpStream,
    peer: SocketAddr,
    config: ServerConfig,
) -> tokio::task::JoinHandle<()> {
    info!("Conexión TCP aceptada de {}", peer);
    let acceptor = tls_acceptor.clone();
    let pin = config.ping.clone();

    let r = tokio::spawn(async move {
        // 1) Handshake TLS
        let tls_stream = match acceptor.accept(tcp_stream).await {
            Ok(s) => s,
            Err(e) => {
                error!("Error en handshake TLS con {}: {}", peer, e);
                return;
            }
        };
        info!("TLS establecido con {}", peer);

        // 2) Handshake WebSocket + lógica
        if let Err(e) = handle_connection(tls_stream, peer, pin).await {
            error!("Error manejando conexión de {}: {:?}", peer, e);
        }
    });
    r
}
