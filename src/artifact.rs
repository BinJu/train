pub mod manifest;
pub mod pipeline;

use crate::error;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

const DEFAULT_NAMESPACE: &str = "train";

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct ArtifactRequest<'a> {
    pub name: &'a str,
    #[serde(default)]
    pub total: u32,
    #[serde(default)]
    pub target: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refs: Option<Vec<ArtifactRef<'a>>>,
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
    pub secrets: Option<Vec<SecretRef<'a>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accounts: Option<Vec<AccountRef<'a>>>
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct SecretRef<'a> {
    pub name: &'a str
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct AccountRef<'a> {
    pub name: &'a str
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct ArtifactRef<'a> {
    pub name: &'a str
}

pub struct Artifact<'a> {
    pub id: &'a str,
    pub tags: HashMap<&'a str, &'a str>,
    pub total: u32,
    pub in_stock: Vec<Box::<Instance<'a>>>,
    pub target: u32,
    pub build: Rollout<'a>,
    pub clean: Rollout<'a>,
    pub art_refs: Vec<&'a Self>,
}

pub struct Instance<'a> {
    pub id: &'a str,
    pub art_id: &'a str,
    pub run_name: String,
    pub dirt: bool,
    //TODO: Tekton is based on async deploy, You may not get the final status immediately. There
    //must be some process/thread to keep them udpated.
    pub stat: InstanceStatus,
    pub results: Option<HashMap<&'a str, &'a str>> //TODO: abstract the result from pipeline run.
}

pub enum InstanceStatus {
    NotStarted,
    Running,
    Pending,
    Fail,
    Done
}

pub struct Rollout<'a> {
    pub name: String,
    pub accounts: Vec<AccountRef<'a>>,
    pub secrets: Vec<SecretRef<'a>>,
    pub art_refs: Vec<ArtifactRef<'a>>,
    pub params: Vec<manifest::Param<'a>>,
    pub tasks: Vec<manifest::TaskManifest<'a>>,
    pub results: Vec<manifest::ParamValue<'a>>
}

impl <'a>Artifact<'a> {
    pub fn new(art_id: &'a str, total: u32, target: u32) -> Self {
        Artifact {
            id: art_id,
            tags: HashMap::new(),
            total,
            target,
            in_stock: Vec::new(),
            build: Rollout {
                name: "build-".to_owned() + art_id,
                accounts: Vec::new(),
                secrets: Vec::new(),
                art_refs: Vec::new(),
                params: Vec::new(),
                tasks: Vec::new(),
                results: Vec::new()
            },
            clean: Rollout {
                name: "clean-".to_owned() + art_id,
                accounts: Vec::new(),
                secrets: Vec::new(),
                art_refs: Vec::new(),
                params: Vec::new(),
                tasks: Vec::new(),
                results: Vec::new()
            },
            art_refs: Vec::new()
        }
    }

    pub fn rollout(&'a mut self) -> error::Result<usize> {
        let diff = self.target as i32 - self.in_stock.len() as i32;
        let instances = if diff > 0 { //build
            self.build.run(diff)?
        } else {
            self.clean.run(-diff)?
        };
        self.in_stock.extend(instances);
        Ok(self.in_stock.len())
    }

    pub fn destroy(&self) -> error::Result<()> {
        Ok(())
    }
}

impl <'a> From<ArtifactRequest<'a>> for Artifact<'a> {
    fn from(value: ArtifactRequest<'a>) -> Self {
        Artifact {
            id: value.name,
            tags: HashMap::new(),
            total: value.total,
            target: value.target,
            in_stock: Vec::new(),
            art_refs: Vec::new(),
            build: Rollout {
                name: "build-".to_owned() + value.name,
                accounts: value.build.accounts.unwrap_or(Vec::new()),
                secrets: value.build.secrets.unwrap_or(Vec::new()),
                art_refs: value.refs.unwrap_or(Vec::new()),
                params: value.build.params.unwrap_or(Vec::new()),
                tasks: value.build.tasks,
                results: value.build.results.unwrap_or(Vec::new())
            },
            clean: Rollout {
                name: "clean-".to_owned() + value.name,
                accounts: Vec::new(),
                secrets: Vec::new(),
                art_refs: Vec::new(),
                params: value.clean.params.unwrap_or(Vec::new()),
                tasks: value.clean.tasks,
                results: value.clean.results.unwrap_or(Vec::new())
            }
        }
    }
}

impl <'a>Rollout<'a> {
    pub fn run(&'a mut self, copies: i32) -> error::Result<Vec<Box<Instance<'a>>>> {
        let mut result = Vec::new();
        let secrets = self.prepare_secrets()?;
        Self::apply_secrets(&secrets)?;
        // to remove the instance.
        // Check the ArtRefs 
        // Apply secret
        // Make sure the manifest is updated
        let rollout: &Rollout = self;
        let manifest = manifest::Manifest::from(rollout);
        let manifest_yaml = manifest.to_yaml()?;
        pipeline::apply(manifest_yaml, DEFAULT_NAMESPACE)?;
        for _i in 0..copies {
            // Prepare accounts
            let accounts = self.prepare_accounts()?;
            Self::apply_secrets(&accounts)?;
            // Prepare refs
            let refs = self.prepare_refs()?;
            Self::apply_secrets(&refs)?;
            let params: Vec<&str> = vec!["art_id=opsman", "inst_id=warn-ma20"];
            let run_name = pipeline::run(&rollout.name, DEFAULT_NAMESPACE, params)?;
            result.push(Box::new(Instance{
                id: "warn-ma20",
                art_id: &self.name,
                run_name,
                dirt: false,
                stat: InstanceStatus::Running,
                results: None
            }));
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

impl <'a>From<&'a Rollout<'a>> for manifest::Manifest<'a> {
    fn from(value: &'a Rollout<'a>) -> manifest::Manifest<'a> {
        let task_refs: Vec<manifest::TaskDef> = value.tasks.iter().map(|v|manifest::TaskDef{name: v.name.clone(), task_ref: manifest::TaskRef{name: v.name.clone()}, run_after: v.run_after.clone(), params: v.param_values.clone()}).collect();
        let mut tasks = Vec::<manifest::Task>::new();
        for task_def in &value.tasks {
            tasks.push(manifest::Task{
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
                metadata: manifest::Metadata { name: &value.name},
                spec: manifest::PipelineSpec {
                    params: if value.params.len() > 0 { Some(value.params.clone())} else {None},
                    results: if value.results.len() > 0 { Some(value.results.clone())} else {None},
                    tasks: task_refs
                }
            },
            tasks
        }
    }
}

impl <'a> ArtifactRef<'a> {
    pub fn get_data(&self) -> Option<Vec<manifest::ParamValue<'a>>> {
        Some(vec![manifest::ParamValue{name: "abckl", value: "value1"}]) //TODO: How to save the results
    }
}

impl <'a> SecretRef<'a> {
    pub fn get_data(&self) -> Option<manifest::Secret<'a>> {
        Some(manifest::Secret::new(self.name, DEFAULT_NAMESPACE, HashMap::from([("k1", "v1")])))
    }
}

impl <'a> AccountRef<'a> {
    pub fn get_data(&self) -> Option<manifest::Secret<'a>> {
        Some(manifest::Secret::new(self.name, DEFAULT_NAMESPACE, HashMap::from([("k1", "v1")])))
    }
}

pub fn handle_artifact_creation<'a>(artifact: Artifact<'a>) -> error::Result<String> {
    //TODO: secrets must be set to the kubenetes cluster for the user. format: secret_userid
    //TODO: accounts must be set as param/env to the tekton pipeline.
    println!("Creating the artifact with: id={}", artifact.id);
    // 
    /*let request_name = artifact.create.request.name;
    let request: manifest::RolloutRequest = artifact.create.request;

    let manifest = build_tekton_manifest(request)?;
    //let manifest = manifest::art_system_tasks(manifest)?;
    apply_artifact(&manifest, "train")?;
    // prepair the artreference.
    // prepair the secret
    // prepair the account
    let art_id = format!("art_id={}", artifact.id);
    let params: Vec<&str> = vec![&art_id, "torontowarm"];
    let run_name = pipeline::run(request_name, "train", params)?;
    Ok(run_name)*/
    Ok(String::new())
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

        let artifact = Artifact::from(request);
        let manifest = manifest::Manifest::from(&artifact.build);
        // let manifest = manifest::art_system_tasks(manifest).expect("Failed to apply system tasks to the RolloutRquest");
        let manifest_yaml = manifest.to_yaml().expect("Fail to serialize to manifest yaml");
        pipeline::apply(manifest_yaml, DEFAULT_NAMESPACE).expect("Fail to apply manifest to kubernetets");
        let params: Vec<&str> = vec!["art_id=opsman", "inst_id=warn-ma20"];
        let run_name = pipeline::run(&manifest.pipeline.metadata.name, "train", params).unwrap();
        let pipelines = pipeline::list("train").expect("failed to list the pipelines");
        assert!(pipelines.len() >= 1);
        assert_eq!(pipelines[0], "build-opsman-warn-ma20");
        std::thread::sleep(time::Duration::from_secs(1));
        let logs = pipeline::logs(&run_name, "train").expect("Failed to acquire logs");
        println!("### logs: \n{}", logs);
        assert!(logs.contains("John"));
        //pipeline::delete_run("--all", "train").unwrap();
    }
}
