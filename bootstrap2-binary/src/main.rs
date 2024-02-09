#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let app = axum::Router::new().fallback(handle);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:0").await.unwrap();
    println!("{:?}", listener.local_addr());
    axum::serve(listener, app).await.unwrap();
}

async fn handle(req: axum::extract::Request) -> axum::response::Result<axum::response::Response> {
    let method = req.method().as_str().to_string();
    let path = req.uri().path().to_string();
    let body = axum::body::to_bytes(req.into_body(), 4096)
        .await
        .map_err(|e| e.to_string())?;
    println!("{method} {path} {}", String::from_utf8_lossy(&body));
    let mut out = axum::response::Response::new(
        axum::body::Body::from(format!(r#"{{
    "method": "{method}",
    "path": "{path}"
}}
"#,
        )),
    );
    out.headers_mut().insert(
        "content-type",
        axum::http::header::HeaderValue::from_static("application/json"),
    );

    Ok(out)
}
