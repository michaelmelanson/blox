use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use blox_assets::{types::AssetPath, AssetError, AssetManager};
use blox_interpreter::{execute_program, EvaluationContext, Scope, Value};
use blox_language::ast::Identifier;
use http_body_util::Full;
use hyper::{body::Bytes, server::conn::http1, service::service_fn, Request, Response};
use hyper_util::rt::{TokioIo, TokioTimer};
use tokio::net::TcpListener;
use tracing::{debug, error, info, instrument};

use crate::{
    assets::{program::BloxProgram, static_file::StaticFile, template::Template},
    environment::BloxEnvironment,
    router::request_asset_path,
};

pub async fn server_command(port: u16, path: String) -> Result<(), anyhow::Error> {
    let assets = AssetManager::new(&path)?;
    let assets = Arc::new(Mutex::new(assets));

    let environment = BloxEnvironment::new(assets.clone());
    environment.start();

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr).await?;
    info!("Listening on http://{:?}", addr);

    loop {
        let (tcp, _) = listener.accept().await?;
        let io = TokioIo::new(tcp);

        let assets = assets.clone();

        tokio::task::spawn(async move {
            let service = service_fn(|req: Request<hyper::body::Incoming>| async {
                handle_request(req, assets.clone()).await
            });

            if let Err(err) = http1::Builder::new()
                .timer(TokioTimer::new())
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

    let scope = Arc::new(Scope::default());
    for (name, value) in bindings {
        scope.insert_binding(&Identifier(name.clone()), Value::String(value))
    }

    let mut context = EvaluationContext::new(".".to_string(), &scope);

    debug!(?path, "Loading asset");
    match path {
        AssetPath::Route(ref _vec) => {
            match assets.load::<BloxProgram>(&path) {
                Ok(program) => {
                    execute_program(&program.into(), &mut context)?;
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
        path => {
            error!(?path, "Unsupported asset type");
            Ok(Response::new("Unsupported asset type".into()))
        }
    }
}
