use clap::{command, Parser};
use commands::{console_command, server_command};
use tracing_subscriber::EnvFilter;

mod assets;
mod commands;
mod environment;
mod router;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser)]
enum Commands {
    #[command(about = "Start a Blox server")]
    Server {
        #[arg(short, long, default_value = "3000")]
        port: u16,
        #[arg(default_value = ".")]
        directory: String,
    },

    #[command(about = "Start a console")]
    Console {
        #[arg(default_value = ".")]
        directory: String,
    },
}

#[tokio::main]
async fn main() {
    let matches = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    match matches.command {
        Commands::Server { port, directory } => {
            server_command(port, directory)
                .await
                .expect("start command failed");
        }
        Commands::Console { directory } => {
            console_command(&directory)
                .await
                .expect("console command failed");
        }
    }
}
