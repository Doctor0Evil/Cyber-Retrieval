#![forbid(unsafe_code)]

mod config;
mod session;

use crate::config::ProxyConfig;
use crate::session::{SecureChannelProfile, SessionGuard, SessionGuardError, SessionToken};
use http::{Request, Response, StatusCode};
use hyper::body::to_bytes;
use hyper::client::HttpConnector;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Client, Server};
use std::convert::Infallible;
use std::net::SocketAddr;
use time::OffsetDateTime;

type HttpClient = Client<HttpConnector>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = ProxyConfig::load_from_file("sessionguard-proxy.toml")?;

    let client = Client::new();
    let listen: SocketAddr = cfg.listen_addr.parse().expect("invalid listen_addr");

    let shared_cfg = cfg.clone();
    let make_svc = make_service_fn(move |_conn| {
        let client = client.clone();
        let cfg = shared_cfg.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                handle_request(req, client.clone(), cfg.clone())
            }))
        }
    });

    println!("SessionGuard proxy listening on {}", listen);
    Server::bind(&listen).serve(make_svc).await?;

    Ok(())
}

async fn handle_request(
    req: Request<Body>,
    client: HttpClient,
    cfg: ProxyConfig,
) -> Result<Response<Body>, Infallible> {
    match process_request(req, client, cfg).await {
        Ok(resp) => Ok(resp),
        Err(resp) => Ok(resp),
    }
}

async fn process_request(
    req: Request<Body>,
    client: HttpClient,
    cfg: ProxyConfig,
) -> Result<Response<Body>, Response<Body>> {
    // 1. Extract token (simple header-based example; you can adjust).
    let maybe_token_header = req.headers().get("x-session-token");

    let token_json = match maybe_token_header {
        Some(hv) => match hv.to_str() {
            Ok(s) => s,
            Err(_) => {
                return Err(error_response(
                    StatusCode::BAD_REQUEST,
                    "invalid_header_encoding",
                ))
            }
        },
        None => {
            return Err(error_response(
                StatusCode::UNAUTHORIZED,
                "missing_x-session-token_header",
            ))
        }
    };

    let token: SessionToken = match serde_json::from_str(token_json) {
        Ok(t) => t,
        Err(_) => {
            return Err(error_response(
                StatusCode::BAD_REQUEST,
                "invalid_session_token_json",
            ))
        }
    };

    // 2. Build observed secure channel profile from config.
    let observed_profile = SecureChannelProfile {
        dns_fail_closed: cfg.dns_fail_closed,
        doh_pinned: cfg.doh_pinned,
        tls_pinned: cfg.tls_pinned,
        browserless: false,
    };

    let now = OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string());

    // For now, assume BCI is enabled when the proxy runs.
    let bcienabled = true;

    // 3. Validate SessionGuard.
    let guard = match SessionGuard::new(
        token,
        &cfg.expected_device_fingerprint,
        &observed_profile,
        bcienabled,
        &now,
    ) {
        Ok(g) => g,
        Err(e) => {
            let msg = match e {
                SessionGuardError::InvalidEnv(reason) => reason,
                SessionGuardError::Expired => "session_expired",
                SessionGuardError::RohViolation => "roh_violation",
                SessionGuardError::NeurorightsViolation(reason) => reason,
            };
            return Err(error_response(StatusCode::FORBIDDEN, msg));
        }
    };

    // Optional: enforce roles
    if !guard.has_role(&session::Role::Chat) {
        return Err(error_response(
            StatusCode::FORBIDDEN,
            "missing_chat_role_in_token",
        ));
    }

    // 4. Forward request to backend (strip original host, rewrite URI).
    let backend_uri = format!(
        "{}{}",
        cfg.backend_base_url,
        req.uri().path_and_query().map(|pq| pq.as_str()).unwrap_or("/")
    );

    let (parts, body) = req.into_parts();
    let mut new_req = Request::builder()
        .method(parts.method)
        .uri(backend_uri)
        .version(parts.version);

    // Copy headers except the original Host; you can filter more later.
    for (k, v) in parts.headers.iter() {
        if k.as_str().eq_ignore_ascii_case("host") {
            continue;
        }
        new_req = new_req.header(k, v);
    }

    let new_req = match new_req.body(body) {
        Ok(r) => r,
        Err(_) => return Err(error_response(StatusCode::INTERNAL_SERVER_ERROR, "build_error")),
    };

    match client.request(new_req).await {
        Ok(resp) => {
            // Optionally, strip or sanitize Set-Cookie headers here.
            Ok(resp)
        }
        Err(_) => Err(error_response(
            StatusCode::BAD_GATEWAY,
            "backend_unreachable",
        )),
    }
}

fn error_response(status: StatusCode, code: &str) -> Response<Body> {
    let body = serde_json::json!({
        "error": code,
        "roh_leq_03": true,
        "retrieval_only": true
    });
    let body_str = serde_json::to_string(&body).unwrap_or_else(|_| "{\"error\":\"encode\"}".into());
    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Body::from(body_str))
        .unwrap()
}
