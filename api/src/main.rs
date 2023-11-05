use std::convert::Infallible;
use hyper::{Body, Request, Response, server::Server};
use hyper::service::{make_service_fn, service_fn};
use train_lib::{error, artifact::{Artifact, ArtifactRequest}};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn 'static + std::error::Error + Send + Sync>>{
    env_logger::init();    
    log::info!("Starting API service.");
    let make_svc = make_service_fn(|_conn| {
        async { Ok::<_, Infallible>(service_fn(handler)) }
    });

    let addr = ([127,0,0,1], 3200).into();
    let server = Server::bind(&addr).serve(make_svc);

    log::info!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}

async fn handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let path = req.uri().path();
    match path {
        "/api/v1/art" => handler_artifact(req).await,
        "/api/v1/sec" => handler_secret(req).await,
        _ => Ok(Response::new(Body::from(format!("Hello World from: {}", path))))
    }
}

async fn handler_artifact(mut req: Request<Body>) -> Result<Response<Body>, Infallible> {
    match hyper::body::to_bytes(req.body_mut()).await {
        Ok(data) => {
            match save_artifact(&data).await {
                Ok(()) => Ok(Response::new(Body::from("Ok"))),
                Err(err) => Ok(Response::new(Body::from(format!("Fail to save artifact: {}", err))))

            }
        },
        Err(err) => {
            Ok(Response::new(Body::from(format!("Hello World Error: {}", err))))
        }
    }
}

async fn handler_secret(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from(format!("Hello World from: {}", req.uri().path()))))
}

async fn save_artifact(data: &[u8]) -> error::Result<()>{
    let str_data = std::str::from_utf8(data).map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("Failed to deserialize utf8: {err}")))?;
    
    let artifact_request: ArtifactRequest = serde_json::from_str(str_data).expect("failed to deserialize the artifact request");
    artifact_request.validate().expect("Failed to validate the request");
    let artifact = Artifact::from(artifact_request);
    artifact.save()
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
