//! Configuración TLS para el servidor WebSocket.
use crate::config;
use anyhow::{Context, Result};
use log::info;
use rcgen::{CertifiedKey, generate_simple_self_signed};
use rustls::ServerConfig;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

const CERT_PATH: &str = "cert.pem";
const KEY_PATH: &str = "key.pem";

/// Carga la configuración TLS.
/// Si existen `cert.pem` y `key.pem` en el directorio actual, los usa.
/// Si no, genera un certificado autofirmado para `localhost`, la IP LAN
/// indicada, y `127.0.0.1`, y lo guarda en disco para reutilizarlo.
pub fn load_or_generate_tls_config(lan_ip: &str) -> Result<Arc<ServerConfig>> {
    let certs: Vec<CertificateDer<'static>>;
    let key: PrivateKeyDer<'static>;
    let base = if let Some(c) = config::get_config_dir() {
        c.join("virtual-mic-server")
    } else {
        PathBuf::new()
    };
    let cert_p = base.join(CERT_PATH);
    let key_p = base.join(KEY_PATH);

    if cert_p.exists() && key_p.exists() {
        info!(
            "Cargando certificado TLS desde {:#?} y {:#?}",
            cert_p, key_p
        );
        (certs, key) = load_from_pem(&cert_p, &key_p)?;
    } else {
        fs::create_dir_all(base).unwrap();
        info!("Generando certificado autofirmado para LAN...");
        (certs, key) = generate_self_signed(lan_ip)?;
        save_to_pem(&certs, &key, cert_p.clone(), key_p.clone())?;
        info!(
            "Certificado autofirmado guardado en {:#?} y {:#?}",
            cert_p, key_p
        );
    }

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .context("No se pudo construir ServerConfig TLS")?;

    Ok(Arc::new(config))
}

fn load_from_pem(
    cert_path: &PathBuf,
    key_path: &PathBuf,
) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {
    let cert_file = fs::File::open(cert_path).context("Abriendo cert.pem")?;
    let mut cert_reader = std::io::BufReader::new(cert_file);
    let certs: Vec<CertificateDer<'static>> = rustls_pemfile::certs(&mut cert_reader)
        .collect::<Result<Vec<_>, _>>()
        .context("Parseando cert.pem")?;

    let key_file = fs::File::open(key_path).context("Abriendo key.pem")?;
    let mut key_reader = std::io::BufReader::new(key_file);
    let key = rustls_pemfile::private_key(&mut key_reader)
        .context("Parseando key.pem")?
        .ok_or_else(|| anyhow::anyhow!("No se encontró clave privada en {:#?}", key_path))?;

    Ok((certs, key))
}

fn generate_self_signed(
    lan_ip: &str,
) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {
    let subject_alt_names = vec![
        "localhost".to_string(),
        "127.0.0.1".to_string(),
        lan_ip.to_string(),
    ];

    let CertifiedKey { cert, signing_key } = generate_simple_self_signed(subject_alt_names)
        .context("Generando certificado autofirmado")?;

    let cert_der = CertificateDer::from(cert.der().to_vec());
    let key_der = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(signing_key.serialize_der()));

    Ok((vec![cert_der], key_der))
}

fn save_to_pem(
    certs: &[CertificateDer<'static>],
    key: &PrivateKeyDer<'static>,
    certs_p: PathBuf,
    key_p: PathBuf,
) -> Result<()> {
    // Certificado
    let mut cert_pem = String::new();
    for cert in certs {
        cert_pem.push_str(&pem_encode_cert(cert));
    }
    fs::write(certs_p, cert_pem).context("Escribiendo cert.pem")?;

    // Clave privada (PKCS#8)
    let key_bytes = match key {
        PrivateKeyDer::Pkcs8(k) => k.secret_pkcs8_der().to_vec(),
        _ => anyhow::bail!("Tipo de clave no soportado para guardar"),
    };
    let key_pem = pem_encode_key(&key_bytes);
    fs::write(key_p, key_pem).context("Escribiendo key.pem")?;

    Ok(())
}

fn pem_encode_cert(cert: &CertificateDer<'_>) -> String {
    pem_encode("CERTIFICATE", cert.as_ref())
}

fn pem_encode_key(key: &[u8]) -> String {
    pem_encode("PRIVATE KEY", key)
}

fn pem_encode(label: &str, data: &[u8]) -> String {
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(data);
    let mut out = String::new();
    out.push_str(&format!("-----BEGIN {}-----\n", label));
    for chunk in b64.as_bytes().chunks(64) {
        out.push_str(std::str::from_utf8(chunk).unwrap());
        out.push('\n');
    }
    out.push_str(&format!("-----END {}-----\n", label));
    out
}
