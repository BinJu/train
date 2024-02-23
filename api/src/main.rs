//! The API interface is responsebile to response the request from users. It save the data to DB,
//! and talk to other components such as engine and reconciller to fulfill the request.
//!
use actix_web::{get, post, patch, put, delete, Result, web, App, middleware, HttpServer, HttpResponse, http::StatusCode};
use actix_web_httpauth::extractors::bearer::BearerAuth;

use train_lib::bo::{ArtifactOps,artifact::ArtifactRequest, ConnectionPool, initialize_db_pool};
use train_lib::scheduler::{Executable, DefaultExecutor};

/// Create the artifact.
/// User need to have the bearer token in the header. if the token does not match the token, the
/// request will be rejected.
/// We may consider to issue tokens with lifespan to avoid the leakage of the team token. Or the
/// team tokens have to be renewed in the given time.
/// For the `ArtifactRequest`, please see the sample json file under `asset` folder.
/// Return 200 if the `ArtifactRequest` is accepted. A notice message will be sent to `engine` to
/// schedule the artifact.
/// Return 400 if the artifact is malformed.
///
#[post("/api/v1/art")]
async fn art_create(auth: BearerAuth, pool: web::Data<ConnectionPool>, data: web::Json<ArtifactRequest>) -> Result<HttpResponse> {
    // Validate the request
    let token = auth.token();
    if let Ok(mut conn) = pool.get() {
        let art_id = ArtifactOps::create(&mut conn, token, data.into_inner())?;
        // notify the engine that the new art is ready.
        let client = awc::Client::new();
        let res = client.post(format!("http://scheduler.train.svc.cluster.local/api/v1/sched/{art_id}")).send().await;
        match res {
            Ok(r) => {
                if r.status() != awc::http::StatusCode::OK {
                    log::warn!("WARN: failed to notify scheduler");
                }
            },
            Err(e) => {
                log::warn!("WARN: failed to notify scheduler. error: {}", e);
            }
        }

        Ok(HttpResponse::build(StatusCode::OK).into())
    } else {
        Ok(HttpResponse::build(StatusCode::BAD_REQUEST).body("Out of database bandwith"))
    }
}

/// Update the artifact
///
#[patch("/api/v1/art/{art_id}")]
async fn art_update(auth: BearerAuth, pool: web::Data<ConnectionPool>, data: web::Json<ArtifactRequest>) -> Result<HttpResponse> {
    // Validate the request
    let token = auth.token();
    if let Ok(mut conn) = pool.get() {
        ArtifactOps::update(&mut conn, token, data.into_inner())?;
        Ok(HttpResponse::build(StatusCode::OK).into())
    } else {
        Ok(HttpResponse::build(StatusCode::BAD_REQUEST).body("Out of database bandwith"))
    }
}


#[get("/api/v1/art")]
async fn art_list() -> Result<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::OK).body(""))
}

#[get("/api/v1/art/{art_id}")]
async fn art_show(art_id: web::Path<String>) -> Result<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::OK).body(art_id.into_inner()))
}
#[delete("/api/v1/art/{art_id}")]
async fn art_delete(art_id: web::Path<String>) -> Result<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::OK).body(art_id.into_inner()))
}

#[put("/api/v1/art/{art_id}/borrow")]
async fn art_borrow(art_id: web::Path<String>) -> Result<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::OK).body(art_id.into_inner()))
}

#[put("/api/v1/art/{art_id}/return")]
async fn art_return(art_id: web::Path<String>) -> Result<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::OK).body(art_id.into_inner()))
}
#[put("/api/v1/art/{art_id}/pause")]
async fn art_pause(art_id: web::Path<String>) -> Result<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::OK).body(art_id.into_inner()))
}

#[put("/api/v1/art/{art_id}/resume")]
async fn art_resume(art_id: web::Path<String>) -> Result<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::OK).body(art_id.into_inner()))
}

#[post("/api/v1/sec")]
async fn secret_create(art_id: web::Path<String>) -> Result<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::OK).body(art_id.into_inner()))
}

#[patch("/api/v1/sec/{sec_id}")]
async fn secret_update(art_id: web::Path<String>) -> Result<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::OK).body(art_id.into_inner()))
}

#[get("/api/v1/sec/{sec_id}")]
async fn secret_show(art_id: web::Path<String>) -> Result<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::OK).body(art_id.into_inner()))
}

#[get("/api/v1/sec")]
async fn secret_list(art_id: web::Path<String>) -> Result<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::OK).body(art_id.into_inner()))
}

#[delete("/api/v1/sec/{sec_id}")]
async fn secret_delete(art_id: web::Path<String>) -> Result<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::OK).body(art_id.into_inner()))
}


#[post("/api/v1/acnt")]
async fn account_create(art_id: web::Path<String>) -> Result<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::OK).body(art_id.into_inner()))
}

#[patch("/api/v1/acnt/{acnt_id}")]
async fn account_update(art_id: web::Path<String>) -> Result<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::OK).body(art_id.into_inner()))
}

#[get("/api/v1/acnt/{acnt_id}")]
async fn account_show(art_id: web::Path<String>) -> Result<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::OK).body(art_id.into_inner()))
}

#[get("/api/v1/acnt")]
async fn account_list(art_id: web::Path<String>) -> Result<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::OK).body(art_id.into_inner()))
}

#[delete("/api/v1/acnt/{acnt_id}")]
async fn account_delete(art_id: web::Path<String>) -> Result<HttpResponse> {
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
            .service(art_create)
            .service(art_list)
            .service(art_show)
            .service(art_update)
            .service(art_delete)
            .service(art_borrow)
            .service(art_return)
            .service(art_pause)
            .service(art_resume)
            .service(secret_list)
            .service(secret_show)
            .service(secret_create)
            .service(secret_update)
            .service(secret_delete)
            .service(account_list)
            .service(account_show)
            .service(account_create)
            .service(account_update)
            .service(account_delete)
    })
    .bind(("0.0.0.0", 3200))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use diesel::Connection;
    use train_lib::{bo::{get_connection, TeamOps}, error};

    use super::*;
    use actix_web::test;

    fn init_test() {
        let mut conn = get_connection();
        conn.transaction(|conn| {
            if TeamOps::find_team_by_name_for_update(conn, "admin".to_owned()).is_err() {
                TeamOps::create(conn, String::from("admin"), None);
            };
            Result::<_, error::GeneralError>::Ok(())
        }).unwrap();
    }

    fn clean_test() {
        let mut conn = get_connection();
        conn.transaction(|conn| {
            train_lib::bo::tests::clean(conn);
            Result::<_, error::GeneralError>::Ok(())
        }).unwrap();
    }

    #[actix_web::test]
    async fn test_create_artifact() {
        dotenvy::dotenv().ok();
        init_test();
        env_logger::try_init_from_env(env_logger::Env::new().default_filter_or("info")).ok();

        let pool = initialize_db_pool();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .wrap(middleware::Logger::default())
                .service(art_create)
        )
        .await;

        // send something that isn't a UUID to `get_user`
        let file = std::fs::File::open("../asset/sample-artifact-request.json").unwrap();

        let mut json_data: ArtifactRequest = serde_json::from_reader(file).expect("Fail to parse the json ArtifactRequest");
        json_data.name = String::from("test-create-artifact");
        let req = test::TestRequest::post().uri("/api/v1/art").set_json(&json_data).insert_header(("Authorization", "Bearer 123456")).to_request();
        let res = test::call_service(&app, req).await;
        println!("response: {:?}", res.response());
        assert_eq!(res.status(), StatusCode::OK);
        clean_test();
    }

    #[actix_web::test]
    async fn test_create_artifact_invalid_json() {
        dotenvy::dotenv().ok();
        init_test();
        env_logger::try_init_from_env(env_logger::Env::new().default_filter_or("info")).ok();

        let pool = initialize_db_pool();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .wrap(middleware::Logger::default())
                .service(art_create)
        )
        .await;

        // send something that isn't a UUID to `get_user`
        // let file = std::fs::File::open("../asset/sample-artifact-request-invalid.json").unwrap();

        // let json_data: ArtifactRequest = serde_json::from_reader(file).expect("Fail to parse the json ArtifactRequest");
        let json_data = serde_json::json!({ "name": "test-bad-artifact", "total": "1", "target":"1" });
        let req = test::TestRequest::post().uri("/api/v1/art").set_json(&json_data).insert_header(("Authorization", "Bearer 123456")).to_request();
        let res = test::call_service(&app, req).await;
        println!("response: {:?}", res);
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        clean_test();
    }
}
/*
fn apis() {
    println!("Starting API!");
    println!("Creating artifact API: POST   /api/v1/art?req=ARTIFACT_REQUEST");
    println!("List artifact API:     GET    /api/v1/arts[?id=ARTIFACT_ID]");
    println!("Get artifact API:      PUT    /api/v1/art?action=get");
    println!("Return artifact API:   PUT    /api/v1/art?action=return");
    println!("Show artifact API:     GET    /api/v1/art?id=12345");
    println!("Delete artifact API:   DELETE /api/v1/art?id=12345");

    println!("====================");

    println!("Creating resource API: POST   /api/v1/res?manifest=MANIFEST");
    println!("List resource API:     GET    /api/v1/resources");
    println!("Get resource API:      PUT    /api/v1/res?action=get");
    println!("Return resource API:   PUT    /api/v1/res?action=return");
    println!("Show resource API:     GET    /api/v1/res?id=12345");
    println!("Delete resource API:   DELETE /api/v1/res?id=12345");

    println!("====================");

    println!("Admin clean artifact  POST /api/v1/admin/clean?filter=art");
    println!("Admin clean resource  POST /api/v1/admin/clean?filter=res");
    println!("Admin clean all       POST /api/v1/admin/clean?filter=all");

    println!("Admin create user     POST   /api/v1/admin/user");
    println!("Admin delete user     DELETE /api/v1/admin/user");

    println!("Admin create workload ending cluster POST   /api/v1/admin/cluster");
    println!("Admin delete workload engine cluster DELETE /api/v1/admin/cluster");
}
*/
