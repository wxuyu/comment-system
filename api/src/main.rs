use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let app = blogcomment_api::create_app().await;

    // Vercel injects $PORT; locally BIND_ADDR works too.
    let bind = std::env::var("PORT")
        .or_else(|_| std::env::var("BIND_ADDR"))
        .unwrap_or_else(|_| "3000".to_string());
    let addr: SocketAddr = if bind.contains(':') {
        bind.parse().expect("invalid BIND_ADDR/PORT")
    } else {
        format!("0.0.0.0:{}", bind).parse().expect("invalid PORT")
    };
    println!("BlogComment API listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
