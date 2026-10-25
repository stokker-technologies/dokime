/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Verify HTTP TLS initialization before any backend connection.

/* === Modules === */

#[path = "../../src/app/http.rs"]
mod http;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::io::{Read, Write};
use std::sync::Arc;

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[tokio::test]
async fn first_https_request() {
    let certificate = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let trust = reqwest::Certificate::from_der(certificate.cert.der()).unwrap();
    let client = http::client().add_root_certificate(trust).no_proxy().build().unwrap();
    let key = rustls::pki_types::PrivatePkcs8KeyDer::from(certificate.signing_key.serialize_der());
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let config = rustls::ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(vec![certificate.cert.der().clone()], key.into())
        .unwrap();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let origin = format!("https://localhost:{}", listener.local_addr().unwrap().port());
    let server = tokio::task::spawn_blocking(move || {
        let (socket, _) = listener.accept().unwrap();
        let connection = rustls::ServerConnection::new(Arc::new(config)).unwrap();
        let mut stream = rustls::StreamOwned::new(connection, socket);
        let mut request = Vec::new();
        while !request.ends_with(b"\r\n\r\n") {
            let mut byte = [0];
            stream.read_exact(&mut byte).unwrap();
            request.push(byte[0]);
            assert!(request.len() < 16384);
        }
        stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nready").unwrap();
        stream.flush().unwrap();
    });
    let response = client.get(origin).send().await.unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    assert_eq!(response.text().await.unwrap(), "ready");
    server.await.unwrap();
}
