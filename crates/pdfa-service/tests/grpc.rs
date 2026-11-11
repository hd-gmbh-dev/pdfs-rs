//! Wire-level check: the service answers the vendored `pdfservice.Pdf/Validate`
//! contract with the library's verdict and rule ids.

use std::net::{Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};

use pdfa_service::pdf::{pdf_client::PdfClient, pdf_server::PdfServer, ValidateRequest};
use pdfa_service::PdfService;
use tokio_stream::wrappers::TcpListenerStream;

const CORPUS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../corpus");

async fn start() -> SocketAddr {
    let listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");

    tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(PdfServer::new(PdfService))
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .expect("serve");
    });

    addr
}

fn fixture(name: &str) -> Vec<u8> {
    let path: PathBuf = Path::new(CORPUS).join(name);
    std::fs::read(path).expect("fixture")
}

async fn validate(addr: SocketAddr, file: Vec<u8>) -> pdfa_service::pdf::ValidateReply {
    let mut client = PdfClient::connect(format!("http://{addr}"))
        .await
        .expect("connect");

    client
        .validate(ValidateRequest { file })
        .await
        .expect("validate")
        .into_inner()
}

#[tokio::test]
async fn conformant_fixture_is_valid() {
    let addr = start().await;
    let reply = validate(addr, fixture("canonical-pdfa-1b.pdf")).await;

    assert!(reply.valid);
    assert!(reply.error.is_empty(), "{:?}", reply.error);
}

#[tokio::test]
async fn broken_fixtures_carry_iso_rule_ids() {
    let addr = start().await;

    for (name, rule_id) in [
        ("no-metadata.pdf", "ISO 19005-1:2005:6.7.2:1"),
        ("compressed-metadata-filter.pdf", "ISO 19005-1:2005:6.7.2:2"),
    ] {
        let reply = validate(addr, fixture(name)).await;

        assert!(!reply.valid, "{name}");
        let codes: Vec<&str> = reply.error.iter().map(|err| err.code.as_str()).collect();
        assert_eq!(codes, [rule_id], "{name}");
        assert!(!reply.error[0].details.is_empty(), "{name}");
    }
}

/// An *empty* `/Filter` array is tolerated by default (veraPDF reading).
#[tokio::test]
async fn empty_filter_array_is_tolerated() {
    let addr = start().await;
    let reply = validate(addr, fixture("empty-metadata-filter.pdf")).await;

    assert!(reply.valid);
    assert!(reply.error.is_empty(), "{:?}", reply.error);
}

#[tokio::test]
async fn unreadable_input_is_a_status_error() {
    let addr = start().await;
    let mut client = PdfClient::connect(format!("http://{addr}"))
        .await
        .expect("connect");

    let status = client
        .validate(ValidateRequest {
            file: b"not a pdf".to_vec(),
        })
        .await
        .expect_err("status error");

    assert_eq!(status.code(), tonic::Code::InvalidArgument);
}
