#![feature(unboxed_closures)]
mod telemetry;

use crate::telemetry::{get_logs_subscriber, init_logs_subscriber};
use actix_cors::Cors;

use actix_web::web::Data;
use actix_web::{guard, web, App, HttpRequest, HttpResponse, HttpServer, Result};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};
use teamdeck_tracker_api::{auth::token::AccessToken, create_schema, ApiSchema};
use tracing_actix_web::TracingLogger;

async fn index(
    schema: web::Data<ApiSchema>,
    req: GraphQLRequest,
    http_req: HttpRequest,
) -> GraphQLResponse {
    let mut query: async_graphql::Request = req.into_inner();

    let auth_token = dbg!(get_token(http_req));
    let access_token = dbg!(auth_token.and_then(|t| AccessToken::verify(&t).ok()));

    if let Some(token) = access_token {
        let resource_id = token.resource_id();
        query = query.data(token).data(resource_id);
    }

    schema.execute(query).await.into()
}

fn get_token(req: HttpRequest) -> Option<String> {
    let authorization_header = req.headers().get(AUTHORIZATION);
    if let Some(value) = authorization_header {
        let contents = value.to_str().unwrap_or("");
        let token = contents.split_whitespace().last().map(|t| t.to_string());
        token
    } else {
        None
    }
}

async fn index_playground() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(
            GraphQLPlaygroundConfig::new("/").subscription_endpoint("/"),
        )))
}

#[derive(Debug, Serialize, Deserialize)]
struct GoogleSignInQuery {
    code: String,
}

async fn google_signin_redirect(query: web::Query<GoogleSignInQuery>) -> Result<HttpResponse> {
    // TODO: Add some HTML template for displaying the code in more user friendly way
    Ok(HttpResponse::Ok().body(serde_json::to_string(&query.0)?))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let port = std::env::var("PORT").ok().unwrap_or_else(|| "8000".into());
    let logs_subscriber =
        get_logs_subscriber("TeamdeckTimerAPI".into(), "info".into(), std::io::stdout);
    init_logs_subscriber(logs_subscriber);

    // println!("Playground: http://localhost:8000");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .send_wildcard();

        App::new()
            .wrap(cors)
            .wrap(TracingLogger::default())
            .app_data(Data::new(create_schema()))
            .service(web::resource("/").guard(guard::Post()).to(index))
            .service(web::resource("/").guard(guard::Get()).to(index_playground))
            .service(
                web::resource("/google/redirect")
                    .guard(guard::Get())
                    .to(google_signin_redirect),
            )
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
