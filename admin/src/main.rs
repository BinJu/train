//! The API interface is responsebile to response the request from users. It save the data to DB,
//! and talk to other components such as engine and reconciller to fulfill the request.
//!
use actix_web::{post, delete, Result, web, App, middleware, HttpServer, HttpResponse, http::StatusCode};

use train_lib::bo::initialize_db_pool;

#[post("/api/v1/team")]
async fn team_create(art_id: web::Path<String>) -> Result<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::OK).body(art_id.into_inner()))
}

#[post("/api/v1/team/{team_id}")]
async fn team_update(art_id: web::Path<String>) -> Result<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::OK).body(art_id.into_inner()))
}

#[delete("/api/v1/team/{team_id}")]
async fn team_delete(art_id: web::Path<String>) -> Result<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::OK).body(art_id.into_inner()))
}

#[actix_web::main]
pub async fn main() -> std::io::Result<()>{
    env_logger::init();    
    log::info!("Starting API service at 3200");
    let pool = initialize_db_pool();

    HttpServer::new(move || {
       // let cors = actix_cors::Cors::default()
       //     .allowed_origin("http://localhost:8081")
       //     .allowed_methods(vec!["GET", "POST"]);
        App::new()
       //     .wrap(cors)
            .app_data(web::Data::new(pool.clone()))
            .wrap(middleware::Logger::default())
            .service(team_create)
            .service(team_update)
    })
    .bind(("0.0.0.0", 3201))?
    .run()
    .await
}
