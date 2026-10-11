/// Eventos que el servidor puede aceptar
/// Start para inicar el servidor
/// Pause para evitar aceptar mas conexiones mientra maneja las que tiene abiertas
/// Stop para matar al servidor mientra maneja las conexciones que todavia tiene abiertas

#[derive(PartialEq)]
pub enum ServerEvents {
    #[allow(dead_code)]
    Stop,
    Start,
    Pause,
    Pin(String),
}
