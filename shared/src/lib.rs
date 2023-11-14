pub mod error;
pub mod artifact;
pub mod resource;
pub mod queue;
mod command;

#[cfg(test)]
mod tests {
    use crate::artifact::dao::ArtifactDao;
    use crate::artifact::{ArtifactRequest, Artifact};
    // POST   /api/v1/art with body in json as request
    #[test]
    fn test_artifact_creation() {
        let request = r#"{"name":"opsman-lib","total":1,"target":1,"refs":[{"name":"mock"}],"build":{"tasks":[{"name":"opsman-task1","spec":{"steps":[{"name":"step-collectdata","image":"ubuntu","script":"echo $(params.name)\necho with inputs: art_id: $(params.art_id)\tinst_id: $(params.inst_id) Done\nls -l /var\necho secret aws-route53\nls -l /var/aws-route53\ncat /var/aws-route53/user_id\ncat var/aws-route53/secret\necho secret pivnet\nls -l /var/pivnet\necho account gcp-environment\nls -l /var/gcp-environment","volumeMounts":[{"name":"aws-route53","mountPath":"/var/aws-route53"},{"name":"pivnet","mountPath":"/var/pivnet"},{"name":"gcp-environment","mountPath":"/var/gcp-environment"}]}],"params":[{"name":"name","type":"string","description":"The username"},{"name":"art_id","type":"string","description":"The artifact ID"},{"name":"inst_id","type":"string","description":"The instance ID"}],"volumes":[{"name":"aws-route53","secret":{"secretName":"sec-$(params.art_id)-aws-route53"}},{"name":"pivnet","secret":{"secretName":"sec-$(params.art_id)-pivnet"}},{"name":"gcp-environment","secret":{"secretName":"acnt-$(params.art_id)-$(params.inst_id)-gcp-environment"}}]},"paramValues":[{"name":"name","value":"John"},{"name":"art_id","value":"$(params.art_id)"},{"name":"inst_id","value":"$(params.inst_id)"}]}],"params":[{"name":"art_id","type":"string","description":"The artifact ID"},{"name":"inst_id","type":"string","description":"The instance ID"}],"secrets":[{"name":"aws-route53"},{"name":"pivnet"}],"accounts":[{"name":"gcp-environment"}]},"clean":{"tasks":[{"name":"task1","spec":{"steps":[{"name":"collect-data","image":"ubuntu","script":"echo $(params.name)\necho with art_id:"}],"params":[{"name":"name","type":"string","description":"The username"}]},"paramValues":[{"name":"name","value":"John"}]}],"params":[{"name":"art_id","type":"string","description":"The artifact ID"},{"name":"inst_id","type":"string","description":"The instance ID"}],"secrets":[{"name":"aws-route53"},{"name":"pivnet"}],"accounts":[{"name":"gcp-environment"}]}}"#;
        let artifact_request: ArtifactRequest = serde_json::from_str(request).expect("failed to deserialize the rollout request");
        artifact_request.validate().expect("Failed to validate the request");
        let artifact = Artifact::try_from(artifact_request).expect("failed to deserialize the request to artifact");
        let mut conn = redis::Client::open("redis://127.0.0.1").unwrap().get_connection().unwrap();
        ArtifactDao::delete("opsman-lib", &mut conn).expect("Failed to delete artifact:opsman-lib from DB");
        ArtifactDao::save(artifact, &mut conn).expect("Failed to save artifact");
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
