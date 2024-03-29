use train_lib::error;

use actix_web::{get, post, patch, put, delete, Result, web, App, middleware, HttpServer, HttpResponse, http::StatusCode};

use train_lib::bo::{ArtifactOps,artifact::ArtifactRequest, ConnectionPool, initialize_db_pool};

//TODO: Think about the timing that enqueue the artifact 
// 1. api call after the artifact creation.
// 2. api call after the target number or the total number was changed.
// 3. After the deploy fail. (enter maintainance once a artifact has more than 5 failed). Which may
//    require another crate `worker`, and which sych the status of each instance.
// 4. api call after the artifact delete.

#[post("/api/v1/sched/{art_id}")]
async fn art_sched(pool: web::Data<ConnectionPool>, art_id: web::Path<i32>) -> Result<HttpResponse> {
    if let Ok(mut conn) = pool.get() {
        //TODO: schedule the artifact
        let artifact = ArtifactOps::load_by_id(&mut conn, art_id.into_inner()).unwrap();
        log::info!("received schedule request for art: {}", artifact.name);
        Ok(HttpResponse::build(StatusCode::OK).into())
    } else {
        Ok(HttpResponse::build(StatusCode::BAD_REQUEST).body("Out of database bandwith"))
    }
}

#[post("/api/v1/poll/{art_id}")]
async fn art_poll(pool: web::Data<ConnectionPool>, art_id: web::Path<i32>) -> Result<HttpResponse> {
    if let Ok(mut conn) = pool.get() {
        //TODO: schedule the artifact
        let artifact = ArtifactOps::load_by_id(&mut conn, art_id.into_inner()).unwrap();
        log::info!("received poll request for art: {}", artifact.name);
        Ok(HttpResponse::build(StatusCode::OK).into())
    } else {
        Ok(HttpResponse::build(StatusCode::BAD_REQUEST).body("Out of database bandwith"))
    }
}

async fn background() -> error::Result<()> {
    //TODO:: loop to get the job done
    Ok(())
}

#[actix_web::main]
pub async fn main() -> std::io::Result<()>{
    env_logger::init();    
    log::info!("Starting internal scheduler service at 3201");
    let pool = initialize_db_pool();

    actix_rt::spawn(background());

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(middleware::Logger::default())
            .service(art_sched)
            .service(art_poll)
    })
    .bind(("0.0.0.0", 3202))?
    .run()
    .await
}

/*
fn main() {
    env_logger::init();
    //TODO:
    // 1. scheduler should read the redis list (from the head), if there is not item read, pending.
    // 2. the api should push the new artifact to the end of the list.
    log::info!("start scheduler");
    loop {
        if let Err(err) = scheduler::process(&queue) {
            log::warn!("Fail to dequeue artifact with the error: {}", err);
        }

        // Once a item(the artifact record id) is read
        // Read the hashset 'artifact:{record_id}:total' to total
        // Read the hashset 'artifact:{record_id}:target' to target
        // Read the hashset 'artifact:{art_id}:instance' to get the instance list
        // Read the hashset 'artifact:{art_id}:instance:{inst_id}' to get the instance info.
        // Iterate each of the instances of 'artifact:{art_id}:instance:{inst_id}' of 'artifact:{art_id}:instance', get the status of these instances
        // Calculate the numbers of instances that is under 'succ'
        // The number to be deploy:
        //  buff = total - ready - in_proc - fail, need = target - ready - in_proc
        // to_deploy = min(buff, need)
        // Deploy '{to_deploy}' copies to tekton by calling rollout method of 'Artifact'

    }
}
*/
