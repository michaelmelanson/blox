use std::{collections::HashMap, convert::Infallible, io::Read, path};

use ast::{HttpPathPart, Identifier, Literal};
use hyper::{
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};

#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(pub program);

mod ast;

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
                    clap::Arg::with_name("PROGRAM")
                        .help("path to the program to run")
                        .required(true)
                        .index(1),
                ),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("start") {
        subcommand_run(matches).await.expect("run command failed");
    } else {
        println!("{}", matches.usage.unwrap());
    }
}

async fn subcommand_run<'a>(matches: &'a clap::ArgMatches<'a>) -> Result<(), anyhow::Error> {
    let path = matches.value_of("PROGRAM").unwrap();

    let mut file = std::fs::File::open(path)?;
    let mut code = String::new();
    file.read_to_string(&mut code)?;

    let program = program::ProgramParser::new()
        .parse(&code)
        .expect("parse error");

    let port = matches
        .value_of("PORT")
        .map(|s| u16::from_str_radix(s, 10).expect("port must be a number"))
        .expect("no port given");

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));

    let make_svc = make_service_fn(|socket: &AddrStream| {
        let program = program.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                let program = program.clone();
                async move { handle_request(req, &program).await }
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{:?}", addr);
    server.await?;

    Ok(())
}

pub async fn handle_request(request: Request<Body>, program: &ast::Program) -> Result<Response<Body>, Infallible> {
    
    for decl in &program.declarations {
        match decl {
            ast::Declaration::Endpoint(endpoint) => {
                if let Some(matches) = match_request(&endpoint, &request) {
                    return execute_endpoint(&endpoint, &request, &matches);
                }
            }
        }
    }

    Ok::<_, Infallible>(Response::new(Body::from("Not found")))
}

fn match_request(endpoint: &ast::EndpointDeclaration, request: &Request<Body>) -> Option<HashMap<ast::Identifier, Value>> {
    let verbs_match = match (request.method(), &endpoint.verb) {
        (&hyper::Method::GET, &ast::HttpVerb::Get) => true,
        (&hyper::Method::POST, &ast::HttpVerb::Post) => true,
        (&hyper::Method::PUT, &ast::HttpVerb::Put) => true,
        (&hyper::Method::PATCH, &ast::HttpVerb::Patch) => true,
        (&hyper::Method::DELETE, &ast::HttpVerb::Delete) => true,
        _ => false
    };

    if !verbs_match {
        return None
    }

    let path_parts = request.uri().path().strip_prefix("/").unwrap().split("/").collect::<Vec<_>>();
    let endpoint_parts = &endpoint.path.parts;

    if path_parts.len() != endpoint_parts.len() {
        return None
    }

    let mut matches = HashMap::new();

    for (path_part, endpoint_part) in path_parts.iter().zip(endpoint_parts) {
        match endpoint_part {
            HttpPathPart::Literal(literal) => {
                if literal != *path_part {
                    return None;
                }
            },

            HttpPathPart::Variable(identifier) => {
                matches.insert(identifier.clone(), Value::String(path_part.to_string()));
            }
        }
    }

    Some(matches)
}

fn execute_endpoint(endpoint: &ast::EndpointDeclaration, request: &Request<Body>, matches: &HashMap<ast::Identifier, Value>) -> Result<Response<Body>, Infallible> {
    let mut bindings = matches.clone();

    let mut response = Response::new(Body::empty());
    
    for statement in &endpoint.block.statements {
        match statement {
            ast::BlockStatement::Binding { lhs, rhs } => {
                if let Some(value) = evaluate_expression(rhs, &bindings) {
                    bindings.insert(lhs.clone(), value);
                } else {
                    unimplemented!();
                }
            },

            ast::BlockStatement::FunctionCall(call) => {
                match &call.ident.0 {
                    x if x == "render" => {
                        let mut mime_type = None;
                        let mut body_text = None;
                        let mut status = 200;

                        for argument in &call.arguments {
                            if argument.ident.0 == "text" {
                                match evaluate_expression(&argument.value, &bindings) {
                                    Some(Value::String(string)) => { 
                                        mime_type = Some("text/plain".to_string());
                                        body_text = Some(string)
                                    },
                                    Some(Value::Number(number)) => {
                                        mime_type = Some("text/plain".to_string());
                                        body_text = Some(number.to_string())
                                    },
                                    None => unimplemented!()
                                }
                            } else if argument.ident.0 == "status" {
                                match evaluate_expression(&argument.value, &bindings) {
                                    Some(Value::String(string)) => unimplemented!(),
                                    Some(Value::Number(number)) => {
                                        status = number;
                                    },
                                    None => unimplemented!()
                                }
                            }
                        }

                        response = Response::new(Body::from(body_text.unwrap()));
                    },
                    other => unimplemented!("function {}", other)
                }
            }
        }
    }

    Ok::<_, Infallible>(response)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
    Number(i64),
    String(String)
}

fn evaluate_expression(expression: &ast::Expression, bindings: &HashMap<Identifier, Value>) -> Option<Value> {
    match expression {
        ast::Expression::Term(term) => evaluate_expression_term(term, bindings),
        ast::Expression::Operator { lhs, operator, rhs } => {
            let lhs_value = evaluate_expression_term(lhs, bindings);
            let rhs_value = evaluate_expression_term(rhs, bindings);

            match operator {
                ast::Operator::Add => match (lhs_value, rhs_value) {
                    (Some(Value::String(lhs)), Some(Value::String(rhs))) => Some(Value::String(lhs + &rhs)),
                    _ => None
                }
            }
        }
    }
}

fn evaluate_expression_term(term: &ast::ExpressionTerm, bindings: &HashMap<Identifier, Value>) -> Option<Value> {
    match term {
        ast::ExpressionTerm::Identifier(identifier) => bindings.get(identifier).clone().map(|x| x.clone()),
        ast::ExpressionTerm::Literal(ast::Literal::Number(number)) => Some(Value::Number(*number)),
        ast::ExpressionTerm::Literal(ast::Literal::String(string)) => Some(Value::String(string.clone())),
        ast::ExpressionTerm::Expression(expression) => evaluate_expression(expression, bindings)
    }
}