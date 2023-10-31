mod error;
mod artifact;
mod resource;
mod command;

use artifact::Artifact;

fn main() {
    apis();
    let artifact = Artifact::new("opsman", 1, 1);
    let run_name = artifact::handle_artifact_creation(artifact).unwrap();
    println!("Running pipeline as {}", run_name);
}

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

//TODO: Add logging


#[cfg(test)]
mod tests {
    use crate::artifact::{ArtifactRequest, Artifact};
    use crate::artifact::pipeline;
    // POST   /api/v1/art with body in json as request
    #[test]
    fn test_artifact_creation() {
        let request = r#"{"name":"opsmanv1","total":1,"target":1,"refs":[{"name":"mock"}],"build":{"tasks":[{"name":"opsman-task1","spec":{"steps":[{"name":"step-collectdata","image":"ubuntu","script":"echo $(params.name)\necho with inputs: art_id: $(params.art_id)\tinst_id: $(params.inst_id) Done\nls -l /var\necho secret aws-route53\nls -l /var/aws-route53\ncat /var/aws-route53/user_id\ncat var/aws-route53/secret\necho secret pivnet\nls -l /var/pivnet\necho account gcp-environment\nls -l /var/gcp-environment","volumeMounts":[{"name":"aws-route53","mountPath":"/var/aws-route53"},{"name":"pivnet","mountPath":"/var/pivnet"},{"name":"gcp-environment","mountPath":"/var/gcp-environment"}]}],"params":[{"name":"name","type":"string","description":"The username"},{"name":"art_id","type":"string","description":"The artifact ID"},{"name":"inst_id","type":"string","description":"The instance ID"}],"volumes":[{"name":"aws-route53","secret":{"secretName":"sec-$(params.art_id)-aws-route53"}},{"name":"pivnet","secret":{"secretName":"sec-$(params.art_id)-pivnet"}},{"name":"gcp-environment","secret":{"secretName":"acnt-$(params.art_id)-$(params.inst_id)-gcp-environment"}}]},"paramValues":[{"name":"name","value":"John"},{"name":"art_id","value":"$(params.art_id)"},{"name":"inst_id","value":"$(params.inst_id)"}]}],"params":[{"name":"art_id","type":"string","description":"The artifact ID"},{"name":"inst_id","type":"string","description":"The instance ID"}],"secrets":[{"name":"aws-route53"},{"name":"pivnet"}],"accounts":[{"name":"gcp-environment"}]},"clean":{"tasks":[{"name":"task1","spec":{"steps":[{"name":"collect-data","image":"ubuntu","script":"echo $(params.name)\necho with art_id:"}],"params":[{"name":"name","type":"string","description":"The username"}]},"paramValues":[{"name":"name","value":"John"}]}],"params":[{"name":"art_id","type":"string","description":"The artifact ID"},{"name":"inst_id","type":"string","description":"The instance ID"}],"secrets":[{"name":"aws-route53"},{"name":"pivnet"}],"accounts":[{"name":"gcp-environment"}]}}"#;
        let artifact_request: ArtifactRequest = serde_json::from_str(request).expect("failed to deserialize the rollout request");
        let mut artifact = Artifact::from(artifact_request);
        assert_eq!(artifact.id, "opsmanv1");
        let len = artifact.rollout().expect("failed to rollout this artifact");
        assert_eq!(len, 1);
        let pipelines = pipeline::list("train").expect("Failed to list pipelines");
        assert!(pipelines.iter().any(|v| v.contains("build-opsman-")));
    }

    #[test]
    fn test_artifact_list() {

    }

    #[test]
    fn test_artifact_borrow() {

    }

    #[test]
    fn test_artfact_return() {

    }

    #[test]
    fn test_artifact_describe() {

    }

    #[test]
    fn test_artifact_update() {

    }

    #[test]
    fn test_artifact_destroy() {

    }

    #[test]
    fn test_secret_creation() {

    }

    #[test]
    fn test_secret_list() {

    }

    #[test]
    fn test_secret_show() {

    }

    #[test]
    fn test_secret_update() {

    }

    #[test]
    fn test_secret_destroy() {

    }

    #[test]
    fn test_account_creation() {

    }

    #[test]
    fn test_account_list() {

    }

    #[test]
    fn test_account_show() {

    }

    #[test]
    fn test_account_update() {

    }

    #[test]
    fn test_account_destroy() {

    }

}
