#[tokio::main]
async fn main() -> anyhow::Result<()> {
    sopho::initialize_logging()?;
    sopho::run().await
}
