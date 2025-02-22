use anyhow::Result;
use axum::Router;
use sloppy::Sloppy;

pub mod nostr;
pub mod sloppy;
pub mod unleashed;

#[tokio::main]
async fn main() -> Result<()> {
    let app = Router::new().route("/", axum::routing::get(|| async { "Hello, World!" }));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    let server_task = tokio::spawn(async move { axum::serve(listener, app).await });

    let mut sloppy = Sloppy::new().await;
    let sloppy_task = tokio::spawn(async move { sloppy.run_survival_loop().await });

    tokio::select! {
        result = server_task => {
            if let Err(e) = result {
                eprintln!("Server error: {}", e);
                std::process::exit(1);
            }
        }
        result = sloppy_task => {
            if let Err(e) = result {
                eprintln!("Sloppy error: {}", e);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
