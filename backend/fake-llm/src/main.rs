use clap::Parser;
use fake_llm::FakeLlmServer;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;

#[derive(Parser)]
struct Arguments {
    #[arg(long)]
    port: u16,
    #[arg(long)]
    fixtures: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let arguments = Arguments::parse();
    let server = FakeLlmServer::start_on(
        SocketAddr::from((Ipv4Addr::LOCALHOST, arguments.port)),
        &arguments.fixtures,
    )
    .await?;
    println!("fake LLM listening on {}", server.base_url());
    tokio::signal::ctrl_c().await?;
    Ok(())
}
