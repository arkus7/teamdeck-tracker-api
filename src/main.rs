mod telemetry;

use crate::telemetry::{get_logs_subscriber, init_logs_subscriber};
use actix_web::web::Data;
use actix_web::{guard, web, App, HttpResponse, HttpServer, Result};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_actix_web::{Request, Response};
use serde::{Deserialize, Serialize};
use teamdeck_tracker_api::{create_schema, ApiSchema};
use tracing_actix_web::TracingLogger;

async fn index(schema: web::Data<ApiSchema>, req: Request) -> Response {
    schema.execute(req.into_inner()).await.into()
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
        App::new()
            .wrap(TracingLogger)
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
