use std::{
    collections::HashMap,
    convert::Infallible,
    sync::{Arc, Mutex},
};

use ast::Identifier;
use blox_assets::AssetManager;
use hyper::{
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use tracing::{error, info, info_span, metadata::LevelFilter};

#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(pub program);

mod assets;
mod ast;

use assets::template::Template;
use blox_assets::types::AssetPath;

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

pub async fn handle_request(
    request: Request<Body>,
    assets: Arc<Mutex<AssetManager>>,
) -> anyhow::Result<Response<Body>> {
    let method = request.method();
    let uri = request.uri();
    let span = info_span!("request");
    let _enter = span.enter();

    info!(
        method = method.as_str(),
        uri = uri.to_string().as_str(),
        "Request info:"
    );

    let path = AssetPath::new(match method {
        &hyper::Method::GET => format!("routes/{}/index", uri.path().trim_matches('/')),
        _ => unimplemented!(),
    });

    let mut assets = assets.lock().unwrap();

    match assets.load::<ast::Program>(&path) {
        Ok(program) => {
            let mut scope = Scope::default();
            execute_program(&program, &mut scope)?;

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

        Err(error) => {
            error!(error = error.to_string().as_str(), "Parse error:");

            Ok(Response::new(Body::from(error.to_string())))
        }
    }
}

#[derive(Default, Debug)]
pub struct Scope {
    bindings: HashMap<Identifier, Value>,
}

impl Scope {
    pub fn child(&self) -> Self {
        Scope {
            bindings: self.bindings.clone(),
        }
    }
}

fn execute_program(program: &ast::Program, scope: &mut Scope) -> Result<(), Infallible> {
    for statement in &program.block.statements {
        match statement {
            ast::Statement::Binding { lhs, rhs } => {
                if let Some(value) = evaluate_expression(rhs, &scope) {
                    scope.bindings.insert(lhs.clone(), value);
                } else {
                    unimplemented!();
                }
            }

            ast::Statement::FunctionCall(_call) => {
                unimplemented!()
            }
        }
    }

    Ok::<_, Infallible>(())
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
    Number(i64),
    String(String),
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Value::Number(number) => number.to_string(),
            Value::String(string) => string.clone(),
        }
    }
}

fn evaluate_expression(expression: &ast::Expression, scope: &Scope) -> Option<Value> {
    match expression {
        ast::Expression::Term(term) => evaluate_expression_term(term, scope),
        ast::Expression::Operator { lhs, operator, rhs } => {
            let lhs_value = evaluate_expression_term(lhs, scope);
            let rhs_value = evaluate_expression_term(rhs, scope);

            match operator {
                ast::Operator::Add => match (lhs_value, rhs_value) {
                    (Some(Value::String(lhs)), Some(Value::String(rhs))) => {
                        Some(Value::String(lhs + &rhs))
                    }
                    _ => None,
                },
            }
        }
    }
}

fn evaluate_expression_term(term: &ast::ExpressionTerm, scope: &Scope) -> Option<Value> {
    match term {
        ast::ExpressionTerm::Identifier(identifier) => {
            scope.bindings.get(identifier).clone().map(|x| x.clone())
        }
        ast::ExpressionTerm::Literal(ast::Literal::Number(number)) => Some(Value::Number(*number)),
        ast::ExpressionTerm::Literal(ast::Literal::String(string)) => {
            Some(Value::String(string.clone()))
        }
        ast::ExpressionTerm::Expression(expression) => evaluate_expression(expression, scope),
    }
}
