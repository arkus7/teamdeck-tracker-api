mod telemetry;

use actix_web::web::Data;
use actix_web::{guard, web, App, HttpResponse, HttpServer, Result};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_actix_web::{Request, Response};
use teamdeck_tracker_api::{create_schema, ApiSchema};
use crate::telemetry::{get_logs_subscriber, init_logs_subscriber};
use tracing_actix_web::{TracingLogger, TracingLoggerMiddleware};

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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let logs_subscriber = get_logs_subscriber("TeamdeckTimerAPI".into(), "info".into(), std::io::stdout);
    init_logs_subscriber(logs_subscriber);

    println!("Playground: http://localhost:8000");

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger)
            .app_data(Data::new(create_schema().clone()))
            .service(web::resource("/").guard(guard::Post()).to(index))
            .service(web::resource("/").guard(guard::Get()).to(index_playground))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
