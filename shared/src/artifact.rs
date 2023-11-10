pub mod manifest;
pub mod pipeline;
pub mod instance;
pub mod artifact_dao;
pub mod instance_dao;

use crate::error;
use chrono::{DateTime,  Local};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use instance::{Instance, InstanceStatus};

const DEFAULT_NAMESPACE: &str = "train";
pub const DEFAULT_REDIS_URL: &str = "redis://127.0.0.1";

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct ArtifactRequest<'a> {
    pub name: &'a str,
    #[serde(default)]
    pub total: u32,
    #[serde(default)]
    pub target: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refs: Option<Vec<ArtifactRef>>,
    pub build: DeployUnit<'a>,
    pub clean: DeployUnit<'a>
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct DeployUnit<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Vec<manifest::Param<'a>>>,
    pub tasks: Vec<manifest::TaskManifest<'a>>,
    pub results: Option<Vec<manifest::ParamValue<'a>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secrets: Option<Vec<SecretRef>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accounts: Option<Vec<AccountRef>>
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct SecretRef {
    pub name: String
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct AccountRef {
    pub name: String
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct ArtifactRef {
    pub name: String
}

#[derive(Debug, PartialEq, Clone)]
pub struct Artifact {
    pub id: String,
    pub tags: HashMap<String, String>,
    pub total: u32,
    pub target: u32,
    pub build: Rollout,
    pub clean: Rollout
}

#[derive(Debug, PartialEq, Clone)]
pub struct Rollout {
    pub name: String,
    pub stats: ArtifactStatus,
    pub last_sched: DateTime<Local>,
    pub accounts: Vec<AccountRef>,
    pub secrets: Vec<SecretRef>,
    pub art_refs: Vec<ArtifactRef>,
    pub manifest: String
}

#[derive(Debug, PartialEq, Clone)]
pub enum ArtifactStatus {
    NotScheduled,
    Running,
    PendingAccount,
    PendingArtRef,
    Fail,
    Done
}

impl Artifact {
    pub fn new(art_id: &str, total: u32, target: u32) -> Self {
        Artifact {
            id: art_id.to_owned(),
            tags: HashMap::new(),
            total,
            target,
            build: Rollout {
                name: "build-".to_owned() + art_id,
                stats: ArtifactStatus::NotScheduled,
                last_sched: "2012-12-12T12:12:12Z".parse::<DateTime<Local>>().expect("Failed to parse datetime string to last_sched"),
                accounts: Vec::new(),
                secrets: Vec::new(),
                art_refs: Vec::new(),
                manifest: String::new()
            },
            clean: Rollout {
                name: "clean-".to_owned() + art_id,
                stats: ArtifactStatus::NotScheduled,
                last_sched: "2012-12-12T12:12:12Z".parse::<DateTime<Local>>().expect("Failed to parse datetime string to last_sched"),
                accounts: Vec::new(),
                secrets: Vec::new(),
                art_refs: Vec::new(),
                manifest: String::new()
            }
        }
    }
}

impl <'a>ArtifactRequest<'a> {
    pub fn validate(&self) -> error::Result<()> {
        //TODO: Need to validate each of the items from the customer. Including: accounts, secrets,
        //art_refs, params, tasks, results for both of the creation and deletion.
        Ok(())
    }
}
impl <'a> TryFrom<ArtifactRequest<'a>> for Artifact {
    type Error = error::GeneralError;
    fn try_from(value: ArtifactRequest<'a>) -> Result<Self, Self::Error> {
        let build_name = "build-".to_owned() + value.name;
        let manifest_build = to_manifest_with_optional_args(&build_name, value.build.tasks, value.build.params, value.build.results);
        let manifest_build_yaml = manifest_build.to_yaml()?;

        let clean_name = "clean-".to_owned() + value.name;
        let manifest_clean = to_manifest_with_optional_args(&clean_name, value.clean.tasks, value.clean.params, value.clean.results);
        let manifest_clean_yaml = manifest_clean.to_yaml()?;
        Ok(Artifact {
            id: value.name.to_owned(),
            tags: HashMap::new(),
            total: value.total,
            target: value.target,
            build: Rollout {
                name: "build-".to_owned() + value.name,
                stats: ArtifactStatus::NotScheduled,
                last_sched: "2012-12-12T12:12:12Z".parse::<DateTime<Local>>().expect("Failed to parse datetime string to last_sched"),
                accounts: value.build.accounts.unwrap_or(Vec::new()),
                secrets: value.build.secrets.unwrap_or(Vec::new()),
                art_refs: value.refs.unwrap_or(Vec::new()),
                manifest: manifest_build_yaml
            },
            clean: Rollout {
                name: "clean-".to_owned() + value.name,
                stats: ArtifactStatus::NotScheduled,
                last_sched: "2012-12-12T12:12:12Z".parse::<DateTime<Local>>().expect("Failed to parse datetime string to last_sched"),
                accounts: Vec::new(),
                secrets: Vec::new(),
                art_refs: Vec::new(),
                manifest: manifest_clean_yaml
            }
        })
    }
}

impl Rollout {
    pub fn run(&mut self, copies: i32) -> error::Result<Vec<Instance>> {
        //TODO: The accounts and art_ref not ready will cause an error, then mark the artifact
        //status to be pending, this should be rescheduled by another module `reconciller`.
        //Update the last_sched
        self.last_sched = Local::now();
        let mut result = Vec::new();
        let secrets = self.prepare_secrets()?;
        Self::apply_secrets(&secrets)?;
        // to remove the instance.
        // Check the ArtRefs 
        // Apply secret
        // Make sure the manifest is updated
        pipeline::apply(self.manifest.clone(), DEFAULT_NAMESPACE)?;
        for _i in 0..copies {
            // Prepare refs
            let refs = self.prepare_refs()?;
            Self::apply_secrets(&refs)?;
            // Prepare accounts
            let accounts = self.prepare_accounts()?;
            Self::apply_secrets(&accounts)?;

            //TODO: Generate the instance ID
            let inst_id = "warn-ma20";
            let arg_art_id = format!("art_id={}", self.name);
            let arg_inst_id = format!("inst_id={}", inst_id);
            let params: Vec<&str> = vec![&arg_art_id, &arg_inst_id];
            let run_name = pipeline::run(&self.name, DEFAULT_NAMESPACE, params)?;
            result.push(Instance{
                id: "warn-ma20".to_owned(),
                art_id: self.name.clone(),
                run_name,
                dirt: false,
                stat: InstanceStatus::Running,
                results: None
            });
        }
        Ok(result)
    }

    fn prepare_refs(&self) -> error::Result<Vec<manifest::Secret>> {
        //TODO: This function is going to claim the artifact that the artifact depends on, and then
        // serialize the json result value as string if it is not a string
        Ok(vec![manifest::Secret::new("ref-opsman-warn-ma20", DEFAULT_NAMESPACE, HashMap::from([("user_id", "user123456"), ("secret", "{\"user_secret\": \"value1\"}")]))])
    }

    fn prepare_secrets(&self) -> error::Result<Vec<manifest::Secret>> {
        //TODO: This is a mock function need to be implemented
        Ok(vec![
           manifest::Secret::new("sec-opsman-aws-route53", DEFAULT_NAMESPACE, HashMap::from([("user_id", "user123456"), ("secret", "user_secret")])),
           manifest::Secret::new("sec-opsman-pivnet", DEFAULT_NAMESPACE, HashMap::from([("user_id", "pivnet123456"), ("pivnet-token", "user-token-123456")]))
        ])
    }

    fn prepare_accounts(&self) -> error::Result<Vec<manifest::Secret>> {
        //TODO: This is a mock function need to be implemented
        Ok(vec![manifest::Secret::new("acnt-opsman-warn-ma20-gcp-environment", DEFAULT_NAMESPACE, HashMap::from([("user_id", "user123456"), ("secret", "user_secret")]))])
    }

    fn apply_secrets(secrets: &Vec<manifest::Secret>) -> error::Result<()> {
        let mut buff = String::new();
        for sec in secrets {
            buff.push_str("---\n");
            let sec_yaml = serde_yaml::to_string(sec)?;
            buff.push_str(&sec_yaml);
        }
        pipeline::apply(buff, DEFAULT_NAMESPACE)?;
        Ok(())
    }
}

impl Default for ArtifactStatus {
    fn default() -> Self {
        ArtifactStatus::NotScheduled
    }
}

fn to_manifest_with_optional_args<'a>(name: &'a str, tasks: Vec<manifest::TaskManifest<'a>>, params: Option<Vec<manifest::Param<'a>>>, results: Option<Vec<manifest::ParamValue<'a>>>) -> manifest::Manifest<'a> {
    let params = params.unwrap_or(Vec::new());
    let results = results.unwrap_or(Vec::new());
    to_manifest(name, tasks, params, results)
}

fn to_manifest<'a>(name: &'a str, tasks: Vec<manifest::TaskManifest<'a>>, params: Vec<manifest::Param<'a>>, results: Vec<manifest::ParamValue<'a>>) -> manifest::Manifest<'a> {
    let task_refs: Vec<manifest::TaskDef> = tasks.iter().map(|v|manifest::TaskDef{name: v.name.clone(), task_ref: manifest::TaskRef{name: v.name.clone()}, run_after: v.run_after.clone(), params: v.param_values.clone()}).collect();
    let mut task_defs = Vec::<manifest::Task>::new();
    for task_def in tasks {
        task_defs.push(manifest::Task{
            api_version: manifest::TEKTON_DEV_V1,
            kind: "Task",
            metadata: manifest::Metadata {
                name: task_def.name
            },
            spec: task_def.spec.clone()
        })
    }
    manifest::Manifest {
        pipeline: manifest::Pipeline {
            api_version: "tekton.dev/v1",
            kind: "Pipeline",
            metadata: manifest::Metadata { name },
            spec: manifest::PipelineSpec {
                params: if params.len() > 0 { Some(params.clone())} else {None},
                results: if results.len() > 0 { Some(results.clone())} else {None},
                tasks: task_refs
            }
        },
        tasks: task_defs
    }
}

impl ArtifactRef {
    pub fn get_data(&self) -> Option<Vec<manifest::ParamValue>> {
        Some(vec![manifest::ParamValue{name: "abckl", value: "value1"}]) //TODO: How to save the results
    }
}

impl SecretRef {
    pub fn get_data(&self) -> Option<manifest::Secret> {
        Some(manifest::Secret::new(&self.name, DEFAULT_NAMESPACE, HashMap::from([("k1", "v1")])))
    }
}

impl AccountRef {
    pub fn get_data(&self) -> Option<manifest::Secret> {
        Some(manifest::Secret::new(&self.name, DEFAULT_NAMESPACE, HashMap::from([("k1", "v1")])))
    }
}

impl ToString for ArtifactStatus {
    fn to_string(&self) -> String {
        match self {
            Self::NotScheduled => "NotScheduled",
            Self::Running => "Running",
            Self::PendingAccount => "PendingAccount",
            Self::PendingArtRef => "PendingArtRef",
            Self::Fail => "Fail",
            Self::Done => "Done"
        }.to_owned()
    }
}

impl <R: AsRef<str>> From<R> for ArtifactStatus {
    fn from(value: R) -> Self {
        let val_ref = value.as_ref();
        match val_ref {
            "Running" => Self::Running,
            "PendingAccount" => Self::PendingAccount,
            "PendingArfRef" => Self::PendingArtRef,
            "Fail" => Self::Fail,
            "Done" => Self::Done,
            _ => Self::NotScheduled
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time;
    #[test]
    fn test_rollout() {
        pipeline::delete_run("--all", "train").unwrap();
        let request = r#"{"name":"arttest","total":1,"target":1,"refs":[{"name":"mock"}],"build":{"tasks":[{"name":"arttest-task1","spec":{"steps":[{"name":"step1","image":"ubuntu","script":"echo $(params.name)\necho with art_id:"}],"params":[{"name":"name","type":"string","description":"The username"}]},"paramValues":[{"name":"name","value":"John"}]}],"params":[{"name":"art_id","type":"string","description":"The artifact ID"},{"name":"inst_id","type":"string","description":"The instance ID"}],"secrets":[{"name":"aws-route53"},{"name":"pivnet"}],"accounts":[{"name":"gcp-environment"}]},"clean":{"tasks":[{"name":"task1","spec":{"steps":[{"name":"step1","image":"ubuntu","script":"echo $(params.name)\necho with art_id:"}],"params":[{"name":"name","type":"string","description":"The username"}]},"paramValues":[{"name":"name","value":"John"}]}],"params":[{"name":"art_id","type":"string","description":"The artifact ID"},{"name":"inst_id","type":"string","description":"The instance ID"}],"secrets":[{"name":"aws_route53"},{"name":"pivnet"}],"accounts":[{"name":"gcp_environment"}]}}"#;

        let mut request: ArtifactRequest = serde_json::from_str(request).expect("Failed to deserialize the payload to ArtifactRequest object");
        request.name = "opsman-warn-ma20";

        let mut artifact = Artifact::try_from(request).expect("Failed to deserialize artifact from artifact request");
        //let manifest_yaml = artifact.build.manifest;
        //pipeline::apply(manifest_yaml, DEFAULT_NAMESPACE).expect("Fail to apply manifest to kubernetets");
        let instances = artifact.build.run(1).expect("Failed to roll out the artifct");
        //let params: Vec<&str> = vec!["art_id=opsman", "inst_id=warn-ma20"];
        //let run_name = pipeline::run(&artifact.build.name, "train", params).unwrap();
        let pipelines = pipeline::list("train").expect("failed to list the pipelines");
        assert!(pipelines.len() >= 1);
        assert!(pipelines.iter().any(|x| x == "build-opsman-warn-ma20"));
        std::thread::sleep(time::Duration::from_millis(20));
        let logs = pipeline::logs(&instances[0].run_name, "train").expect("Failed to acquire logs");
        println!("### logs: \n{}", logs);
        assert!(logs.contains("John"));
        //pipeline::delete_run("--all", "train").unwrap();
    }
}
