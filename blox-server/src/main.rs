use std::sync::{Arc, Mutex};

use blox_assets::{types::AssetPath, AssetError, AssetManager};
use blox_language::ast::Identifier;
use clap::{command, Parser};
use std::net::SocketAddr;
use tracing_subscriber::EnvFilter;

use http_body_util::Full;
use hyper::{body::Bytes, server::conn::http1, service::service_fn, Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use router::request_asset_path;
use tracing::{debug, error, info, instrument, metadata::LevelFilter};

mod assets;
mod router;

use assets::{program::BloxProgram, static_file::StaticFile, template::Template};
use blox_interpreter::{execute_program, Scope, Value};

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
    Start {
        #[arg(short, long, default_value = "3000")]
        port: u16,
        #[arg(default_value = ".")]
        directory: String,
    },
}

#[tokio::main]
async fn main() {
    let matches = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_max_level(LevelFilter::TRACE)
        .init();

    match matches.command {
        Commands::Start { port, directory } => {
            start(port, directory).await.expect("start command failed");
        }
    }
}

async fn start(port: u16, path: String) -> Result<(), anyhow::Error> {
    let cache = AssetManager::new(&path)?;
    let cache = Arc::new(Mutex::new(cache));

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr).await?;
    info!("Listening on http://{:?}", addr);

    loop {
        let (tcp, _) = listener.accept().await?;
        let io = TokioIo::new(tcp);

        let cache = cache.clone();

        tokio::task::spawn(async move {
            let service = service_fn(|req: Request<hyper::body::Incoming>| async {
                handle_request(req, cache.clone()).await
            });

            if let Err(err) = http1::Builder::new()
                // .timer(TokioTimer::new())
                .serve_connection(io, service)
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}

#[instrument(skip(request, assets), fields(method, uri))]
pub async fn handle_request(
    request: Request<hyper::body::Incoming>,
    assets: Arc<Mutex<AssetManager>>,
) -> anyhow::Result<Response<Full<Bytes>>> {
    let method = request.method();
    let uri = request.uri();

    tracing::Span::current()
        .record("method", &method.as_str())
        .record("uri", &uri.path_and_query().unwrap().to_string().as_str());

    let (path, bindings) = request_asset_path(request.method(), request.uri())?;

    info!(?path, "Requesting asset");

    let mut assets = assets.lock().unwrap();

    let mut scope = Scope::default();
    for (name, value) in bindings {
        scope.insert_binding(&Identifier(name.clone()), Value::String(value))
    }

    debug!(?path, "Loading asset");
    match path {
        AssetPath::Route(ref _vec) => {
            match assets.load::<BloxProgram>(&path) {
                Ok(program) => {
                    execute_program(&program.into(), &mut scope)?;
                }

                Err(error) => {
                    match error.downcast_ref::<AssetError>() {
                        // ignore this, just means there's no Blox code
                        Some(AssetError::NoMatchingExtension(_, _)) => {}

                        _ => {
                            error!(error = error.to_string().as_str(), "Parse error:");
                            return Ok(Response::new(error.to_string().into()));
                        }
                    }
                }
            }

            match assets.load::<Template>(&path) {
                Ok(template) => match template.render(&scope) {
                    Ok(body) => Ok(Response::new(body.into())),
                    Err(error) => {
                        error!(
                            error = error.to_string().as_str(),
                            "Error processing template"
                        );
                        Ok(Response::new(error.to_string().into()))
                    }
                },

                Err(error) => {
                    error!(
                        error = error.to_string().as_str(),
                        "Error while running handler"
                    );

                    Ok(Response::new(error.to_string().into()))
                }
            }
        }
        AssetPath::Static(_) => {
            let asset = assets.load::<StaticFile>(&path)?;
            Ok(asset.into())
        }
        AssetPath::Layout(_) => todo!("layout routes"),
    }
}
