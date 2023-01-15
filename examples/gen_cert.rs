use anyhow::{Result, Ok};
use certify::{CertType, generate_ca, load_ca, CA, generate_cert};
use tokio::fs;

struct CertPem {
    cert_type: CertType,
    cert: String,
    key: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let pem = create_ca()?;
    gen_files(&pem).await?;

    let ca = load_ca(&pem.cert, &pem.key)?;
    
    let pem = create_cert(&ca, &["kvserver.acme.inc"], "Acme KV server", false)?;
    gen_files(&pem).await?;
    let pem = create_cert(&ca, &[], "awesome-device-id", true)?;
    gen_files(&pem).await?;
    

    Ok(())
}

fn create_ca() -> Result<CertPem> {
    let (cert, key) = generate_ca(
        &["acme.inc"],
        "CN",
        "Acme Inc",
        "Acme CA",
        None,
        Some(365 * 10),
    )?;

    Ok(CertPem {
        cert_type: CertType::CA,
        cert: cert,
        key: key,
    })
}

fn create_cert(ca: &CA, domains: &[&str], cn: &str, is_client: bool) -> Result<CertPem> {
    let (days, cert_type) = if is_client {
        (Some(365), CertType::Client)
    } else {
        (Some(365 * 10), CertType::Server)
    };

    let (cert, key) = generate_cert(ca, domains, "CN", "Acme Inc", cn, None, is_client, days)?;

    Ok(CertPem {
        cert_type: cert_type,
        cert: cert,
        key: key,
    })
}


async fn gen_files(pem: &CertPem) -> Result<()> {
    let name = match pem.cert_type {
        CertType::CA => "ca",
        CertType::Server => "server",
        CertType::Client => "client",
    };
    fs::write(format!("fixtures/{}.cert", name), pem.cert.as_bytes()).await?;
    fs::write(format!("fixtures/{}.key", name), pem.key.as_bytes()).await?;

    Ok(())
}