use clap::Parser;
use eyre::Result;
use solidity_language_server::lsp::ForgeLsp;
use tower_lsp::{LspService, Server};

#[derive(Clone, Debug, Parser)]
#[command(
    name = "solar-language-server",
    version = env!("LONG_VERSION"),
    about = "A Solidity language server powered by Solar"
)]
pub struct SolarLspArgs {
    #[arg(long)]
    pub stdio: bool,
}

impl SolarLspArgs {
    pub async fn run(self) -> Result<()> {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        let (service, socket) =
            LspService::new(|client| ForgeLsp::new(client, /* use_solar */ true, /* use_solc */ false));
        Server::new(stdin, stdout, socket).serve(service).await;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let args = SolarLspArgs::parse();
    args.run().await
}
