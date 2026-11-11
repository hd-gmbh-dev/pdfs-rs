//! gRPC surface for the façade: implements `pdfservice.Pdf/Validate`, the contract
//! `file-integrity` already calls today.
//!
//! The reply carries the validator's own findings: `ValidateError.code` is the ISO
//! rule id (`ISO 19005-1:2005:6.7.2:2`), `details` the readable message.

use pdfa::{validate_bytes, Profile};
use tonic::{Request, Response, Status};

pub mod pdf {
    tonic::include_proto!("pdfservice");
}

use pdf::{ValidateError, ValidateReply, ValidateRequest};

/// Server implementation; holds no state.
#[derive(Debug, Default)]
pub struct PdfService;

#[tonic::async_trait]
impl pdf::pdf_server::Pdf for PdfService {
    async fn validate(
        &self,
        request: Request<ValidateRequest>,
    ) -> Result<Response<ValidateReply>, Status> {
        let bytes = request.into_inner().file;

        let report = validate_bytes(&bytes, Profile::PdfA1b)
            .map_err(|err| Status::invalid_argument(format!("cannot validate PDF: {err}")))?;

        Ok(Response::new(ValidateReply {
            valid: report.conformant,
            error: report
                .failures
                .into_iter()
                .map(|failure| ValidateError {
                    code: failure.rule_id,
                    details: failure.message,
                })
                .collect(),
        }))
    }
}

/// Serves the validator on `addr` until the future `shutdown` completes.
///
/// # Errors
///
/// Returns the transport error when the address cannot be bound or the server fails.
pub async fn serve<F>(
    addr: std::net::SocketAddr,
    shutdown: F,
) -> Result<(), tonic::transport::Error>
where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    tonic::transport::Server::builder()
        .add_service(pdf::pdf_server::PdfServer::new(PdfService))
        .serve_with_shutdown(addr, shutdown)
        .await
}
