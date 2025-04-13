use axum::{body::Body, http::{Response, StatusCode}};
use tokio_util::io::ReaderStream;
use crate::util::get_cert_path;

pub async fn handle_get_cert() -> Result<Response<Body>, StatusCode> {
    let cert_file = get_cert_path();

    let file = match tokio::fs::File::open(cert_file).await {
        Ok(file) => file,
        Err(_err) => return Err(StatusCode::BAD_REQUEST),
    };

    // convert the `AsyncRead` into a `Stream`
    let stream = ReaderStream::new(file);
    // convert the `Stream` into an `axum::body::Body`
    let body = Body::from_stream(stream);

    let builder = Response::builder()
        .header("Content-Type", "application/x-x509-ca-cert; charset=utf-8")
        .header("Content-Disposition", "attachment; filename=\"cert.pem\"")
        .status(StatusCode::OK);

    Ok(builder.body(body).unwrap())
}