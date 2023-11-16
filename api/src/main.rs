use std::convert::Infallible;
use hyper::{Body, Request, Response, server::Server};
use hyper::service::{make_service_fn, service_fn};
use train_lib::artifact::dao::{ArtifactDao,connection};
use train_lib::queue;
use train_lib::{error, artifact::{Artifact, ArtifactRequest}};

const DEFAULT_REDIS_URL: &str = "redis://train-redis";
#[tokio::main]
pub async fn main() -> Result<(), Box<dyn 'static + std::error::Error + Send + Sync>>{
    env_logger::init();    
    log::info!("Starting API service.");
    let make_svc = make_service_fn(|_conn| {
        async { Ok::<_, Infallible>(service_fn(handler)) }
    });

    let addr = ([0,0,0,0], 3200).into();
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
                Ok(art_id) => {
                    match enqueue(&art_id) {
                        Ok(_) => Ok(Response::new(Body::from("Ok"))),
                        Err(err) => Ok(Response::new(Body::from(format!("Failed to enqueue the artifact: {}, with error: {}", art_id, err))))
                    }
                },
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

async fn save_artifact(data: &[u8]) -> error::Result<String>{
    let str_data = std::str::from_utf8(data).map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("Failed to deserialize utf8: {err}")))?;
    
    let mut artifact_request: ArtifactRequest = serde_json::from_str(str_data)?;
    artifact_request.validate().expect("Failed to validate the request");
    artifact_request.format()?;
    let artifact = Artifact::try_from(artifact_request)?;
    let mut conn = connection(DEFAULT_REDIS_URL).expect(&format!("Failed to open redis connection on {DEFAULT_REDIS_URL}"));
    let art_id = artifact.id.clone();
    ArtifactDao::save(artifact, &mut conn)?;
    Ok(art_id)
}

fn enqueue(art_id: &str) -> error::Result<()> {
    let queue = queue::Queue::new(queue::DEFAULT_QUEUE_NAME.to_owned());
    let mut conn = connection(DEFAULT_REDIS_URL)?;
    queue.enqueue(art_id, &mut conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_main_save_artifact() {
        let request = r#"{"name":"opsman-main","total":1,"target":1,"refs":[{"name":"mock"}],"build":{"tasks":[{"name":"opsman-task1","spec":{"steps":[{"name":"step-collectdata","image":"ubuntu","script":"echo $(params.name)\necho with inputs: art_id: $(params.art_id)\tinst_id: $(params.inst_id) Done\nls -l /var\necho secret aws-route53\nls -l /var/aws-route53\ncat /var/aws-route53/user_id\ncat var/aws-route53/secret\necho secret pivnet\nls -l /var/pivnet\necho account gcp-environment\nls -l /var/gcp-environment","volumeMounts":[{"name":"aws-route53","mountPath":"/var/aws-route53"},{"name":"pivnet","mountPath":"/var/pivnet"},{"name":"gcp-environment","mountPath":"/var/gcp-environment"}]}],"params":[{"name":"name","type":"string","description":"The username"},{"name":"art_id","type":"string","description":"The artifact ID"},{"name":"inst_id","type":"string","description":"The instance ID"}],"volumes":[{"name":"aws-route53","secret":{"secretName":"sec-$(params.art_id)-aws-route53"}},{"name":"pivnet","secret":{"secretName":"sec-$(params.art_id)-pivnet"}},{"name":"gcp-environment","secret":{"secretName":"acnt-$(params.art_id)-$(params.inst_id)-gcp-environment"}}]},"paramValues":[{"name":"name","value":"John"},{"name":"art_id","value":"$(params.art_id)"},{"name":"inst_id","value":"$(params.inst_id)"}]}],"params":[{"name":"art_id","type":"string","description":"The artifact ID"},{"name":"inst_id","type":"string","description":"The instance ID"}],"secrets":[{"name":"aws-route53"},{"name":"pivnet"}],"accounts":[{"name":"gcp-environment"}]},"clean":{"tasks":[{"name":"task1","spec":{"steps":[{"name":"collect-data","image":"ubuntu","script":"echo $(params.name)\necho with art_id:"}],"params":[{"name":"name","type":"string","description":"The username"}]},"paramValues":[{"name":"name","value":"John"}]}],"params":[{"name":"art_id","type":"string","description":"The artifact ID"},{"name":"inst_id","type":"string","description":"The instance ID"}],"secrets":[{"name":"aws-route53"},{"name":"pivnet"}],"accounts":[{"name":"gcp-environment"}]}}"#;
        let data = request.as_bytes();
        let mut conn = connection(DEFAULT_REDIS_URL).expect(&format!("Failed to open redis connection on {DEFAULT_REDIS_URL}"));
        ArtifactDao::delete("opsman-main", &mut conn).expect("Failed to delete opsman-main from DB");
        save_artifact(data).await.expect("Failed to save artifact");

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
