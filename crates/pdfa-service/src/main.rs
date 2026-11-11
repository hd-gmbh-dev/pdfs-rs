//! `pdfs-service` — runs the PDF/A validator as a gRPC service.
//!
//! Listens on `PDFS_ADDR` (default `0.0.0.0:11242`, the port `file-integrity`
//! expects from `pdf_integrity_url`).

use std::net::SocketAddr;

use pdfa_service::serve;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = std::env::var("PDFS_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:11242".to_string())
        .parse()?;

    println!("pdfs-service listening on {addr}");

    serve(addr, async {
        let _ = tokio::signal::ctrl_c().await;
    })
    .await?;

    Ok(())
}
