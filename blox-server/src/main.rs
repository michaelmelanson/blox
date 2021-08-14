use std::{
    convert::Infallible,
    sync::{Arc, Mutex},
};

use blox_assets::{AssetError, AssetManager};
use hyper::{
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use router::request_asset_path;
use tracing::{error, info, instrument, metadata::LevelFilter};

mod assets;
mod router;

use assets::{program::BloxProgram, template::Template};
use blox_interpreter::{execute_program, Scope, Value};

#[tokio::main]
async fn main() {
    let matches = clap::App::new("blox")
        .version("1.0")
        .author("Michael Melanson<michael@michaelmelanson.net")
        .subcommand(
            clap::SubCommand::with_name("start")
                .about("Start a Blox server")
                .arg(
                    clap::Arg::with_name("PORT")
                        .help("the port to bind to")
                        .short("p")
                        .long("port")
                        .default_value("3000")
                        .takes_value(true),
                )
                .arg(
                    clap::Arg::with_name("DIRECTORY")
                        .help("base path for the application")
                        .default_value(".")
                        .index(1),
                ),
        )
        .get_matches();

    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .init();

    if let Some(matches) = matches.subcommand_matches("start") {
        subcommand_run(matches).await.expect("run command failed");
    } else {
        println!("{}", matches.usage.unwrap());
    }
}

async fn subcommand_run<'a>(matches: &'a clap::ArgMatches<'a>) -> Result<(), anyhow::Error> {
    let path = matches.value_of("DIRECTORY").unwrap();
    let cache = AssetManager::new(path)?;
    let cache = Arc::new(Mutex::new(cache));

    let port = matches
        .value_of("PORT")
        .map(|s| u16::from_str_radix(s, 10).expect("port must be a number"))
        .expect("no port given");

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));

    let make_svc = make_service_fn(|_socket: &AddrStream| {
        let cache = cache.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                let cache = cache.clone();
                async move { handle_request(req, cache).await }
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    info!("Listening on http://{:?}", addr);
    server.await?;

    Ok(())
}

#[instrument(skip(request, assets), fields(method, uri))]
pub async fn handle_request(
    request: Request<Body>,
    assets: Arc<Mutex<AssetManager>>,
) -> anyhow::Result<Response<Body>> {
    let method = request.method();
    let uri = request.uri();

    tracing::Span::current()
        .record("method", &method.as_str())
        .record("uri", &uri.path_and_query().unwrap().to_string().as_str());

    let (path, bindings) = request_asset_path(request.method(), request.uri())?;

    let mut assets = assets.lock().unwrap();

    let mut scope = Scope::default();
    for (name, value) in bindings {
        scope.insert_binding(name.clone(), Value::String(value))
    }

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
                    return Ok(Response::new(Body::from(error.to_string())));
                }
            }
        }
    }

    match assets.load::<Template>(&path) {
        Ok(template) => match template.render(&scope) {
            Ok(body) => Ok(Response::new(Body::from(body))),
            Err(error) => {
                error!(
                    error = error.to_string().as_str(),
                    "Error processing template"
                );
                Ok(Response::new(Body::from(error.to_string())))
            }
        },

        Err(error) => {
            error!(
                error = error.to_string().as_str(),
                "Error while running handler"
            );

            Ok(Response::new(Body::from(error.to_string())))
        }
    }
}
