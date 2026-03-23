mod client;
mod server;

use rmcp::ServiceExt;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let client_id = std::env::var("UNDERSTORY_CLIENT_ID")
        .expect("UNDERSTORY_CLIENT_ID environment variable is required");
    let client_secret = std::env::var("UNDERSTORY_CLIENT_SECRET")
        .expect("UNDERSTORY_CLIENT_SECRET environment variable is required");

    let client = client::UnderstoryClient::new(client_id, client_secret);
    let server = server::UnderstoryServer::new(client);

    let service = server.serve(rmcp::transport::io::stdio()).await?;
    service.waiting().await?;

    Ok(())
}
