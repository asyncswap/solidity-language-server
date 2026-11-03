#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

use clap::Parser;
use eyre::Result;
use solidity_language_server::invariants::InvariantService;
use solidity_language_server::lsp::ForgeLsp;
use tower_lsp::{LspService, Server};

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum CompletionMode {
    Full,
    Fast,
}

#[derive(Clone, Debug, Parser)]
#[command(
    version = env!("LONG_VERSION"),
    about = "solidity-language-server, a Solidity LSP powered by foundry"
)]
pub struct LspArgs {
    #[arg(long)]
    pub stdio: bool,
    #[arg(long)]
    pub use_solar: bool,
    /// Fall back to forge build for AST generation instead of solc.
    /// By default, the LSP uses solc directly for faster AST generation.
    /// This flag forces the forge-based pipeline.
    #[arg(long)]
    pub use_forge: bool,
    /// Deprecated: scope-aware completions are now always enabled.
    /// This flag is a no-op and will be removed in a future release.
    #[arg(long, value_enum, hide = true)]
    pub completion_mode: Option<CompletionMode>,
    /// Validate every response against the LSP specification and the open
    /// document. Violations are printed to stderr and returned as errors.
    #[arg(long)]
    pub check_invariants: bool,
}

impl LspArgs {
    pub async fn run(self) -> Result<()> {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        let use_solc = !self.use_forge;
        let (service, socket) =
            LspService::new(|client| ForgeLsp::new(client, self.use_solar, use_solc));
        let server = Server::new(stdin, stdout, socket);
        if self.check_invariants {
            server.serve(InvariantService::new(service)).await;
        } else {
            server.serve(service).await;
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    let args = LspArgs::parse();
    args.run().await
}
