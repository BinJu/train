use chrono::{DateTime,  Local};
use diesel::PgConnection;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use super::{dao, manifest};
use crate::error;

const DEFAULT_NAMESPACE: &'static str = "train";

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct ArtifactRequest {
    pub name: String,
    #[serde(default)]
    pub total: i32,
    #[serde(default)]
    pub target: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refs: Option<Vec<ArtifactRef>>,
    pub build: DeployUnit,
    pub clean: DeployUnit
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct DeployUnit {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Vec<manifest::Param>>,
    pub tasks: Vec<manifest::TaskManifest>,
    pub results: Option<Vec<manifest::ParamValue>>,
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
    pub total: i32,
    pub target: i32,
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
    Failed,
    Succeeded
}


impl Artifact {
    pub fn new(art_id: &str, total: i32, target: i32) -> Self {
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

impl ArtifactRequest {
    pub fn format(&mut self) -> error::Result<()> {
        for task in &mut self.build.tasks {
            task.name = format!("{}-{}", self.name, task.name);
        }
        Ok(())
    }
}

impl  TryFrom<ArtifactRequest> for Artifact {
    type Error = error::GeneralError;
    fn try_from(value: ArtifactRequest) -> Result<Self, Self::Error> {
        let build_name = "build-".to_owned() + &value.name;
        let manifest_build = to_manifest_with_optional_args(&build_name, value.build.tasks, value.build.params, value.build.results);
        let manifest_build_yaml = manifest_build.to_yaml()?;

        let clean_name = "clean-".to_owned() + &value.name;
        let manifest_clean = to_manifest_with_optional_args(&clean_name, value.clean.tasks, value.clean.params, value.clean.results);
        let manifest_clean_yaml = manifest_clean.to_yaml()?;
        Ok(Artifact {
            id: value.name.to_owned(),
            tags: HashMap::new(),
            total: value.total,
            target: value.target,
            build: Rollout {
                name: value.name.to_owned(),
                stats: ArtifactStatus::NotScheduled,
                last_sched: "2012-12-12T12:12:12Z".parse::<DateTime<Local>>().expect("Failed to parse datetime string to last_sched"),
                accounts: value.build.accounts.unwrap_or(Vec::new()),
                secrets: value.build.secrets.unwrap_or(Vec::new()),
                art_refs: value.refs.unwrap_or(Vec::new()),
                manifest: manifest_build_yaml
            },
            clean: Rollout {
                name: value.name.to_owned(),
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
    pub fn run(&mut self, _copies: i32) -> error::Result<Vec<String>> {
        //TODO: The accounts and art_ref not ready will cause an error, then mark the artifact
        //status to be pending, this should be rescheduled by another module `reconciller`.
        //Update the last_sched
        /*self.last_sched = Local::now();
        let mut result = Vec::new();
        let secrets = self.prepare_secrets(&self.name)?;
        log::info!("applying secrets");
        Self::apply_secrets(&secrets)?;
        log::info!("applied.");
        // to remove the instance.
        // Check the ArtRefs 
        // Apply secret
        // Make sure the manifest is updated
        log::info!("applying manifest: {}", self.manifest);
        pipeline::apply(self.manifest.clone(), DEFAULT_NAMESPACE)?;
        log::info!("applied.");
        let pipeline_prefix = if copies >= 0 { "build" } else { copies = -copies; "clean" };
        for _i in 0..copies {
            let inst_id = format!("{}-{}", naming::word(None), naming::random_id());
            // Prepare refs
            let refs = self.prepare_refs(&self.name, &inst_id)?;
            Self::apply_secrets(&refs)?;
            // Prepare accounts
            let accounts = self.prepare_accounts(&self.name, &inst_id)?;
            Self::apply_secrets(&accounts)?;

            let arg_art_id = format!("art_id={}", self.name);
            let arg_inst_id = format!("inst_id={}", inst_id);
            let params: Vec<&str> = vec![&arg_art_id, &arg_inst_id];
            let run_name = pipeline::run(&format!("{}-{}", pipeline_prefix, self.name), DEFAULT_NAMESPACE, &params)?;
            result.push(Instance{
                id: inst_id,
                art_id: self.name.clone(),
                run_name,
                dirt: false,
                stat: InstanceStatus::Running,
                results: None
            });
        }
        */
        let result = Vec::new();
        Ok(result)
    }

    pub fn validate(&self) -> error::Result<()> {
        Ok(())
    }

    pub fn format(&mut self) -> error::Result<()> {
        Ok(())
    }

    fn prepare_refs(&self, art_id: &str, inst_id: &str) -> error::Result<Vec<manifest::Secret>> {
        //TODO: This function is going to claim the artifact that the artifact depends on, and then
        // serialize the json result value as string if it is not a string
        Ok(vec![manifest::Secret::new(format!("ref-{}-{}", art_id, inst_id), DEFAULT_NAMESPACE, HashMap::from([("user_id", "user123456"), ("secret", "{\"user_secret\": \"value1\"}")]))])
    }

    fn prepare_secrets(&self, art_id: &str) -> error::Result<Vec<manifest::Secret>> {
        //TODO: This is a mock function need to be implemented
        Ok(vec![
           manifest::Secret::new(format!("sec-{}-aws-route53", art_id), DEFAULT_NAMESPACE, HashMap::from([("user_id", "user123456"), ("secret", "user_secret")])),
           manifest::Secret::new(format!("sec-{}-pivnet", art_id), DEFAULT_NAMESPACE, HashMap::from([("user_id", "pivnet123456"), ("pivnet-token", "user-token-123456")]))
        ])
    }

    fn prepare_accounts(&self, art_id: &str, inst_id: &str) -> error::Result<Vec<manifest::Secret>> {
        //TODO: This is a mock function need to be implemented
        Ok(vec![manifest::Secret::new(format!("acnt-{}-{}-gcp-environment", art_id, inst_id), DEFAULT_NAMESPACE, HashMap::from([("user_id", "user123456"), ("secret", "user_secret")]))])
    }

    fn apply_secrets(secrets: &Vec<manifest::Secret>) -> error::Result<()> {
        let mut buff = String::new();
        for sec in secrets {
            buff.push_str("---\n");
            let sec_yaml = serde_yaml::to_string(sec)?;
            buff.push_str(&sec_yaml);
        }
        //pipeline::apply(buff, DEFAULT_NAMESPACE)?;
        Ok(())
    }
}

impl Default for ArtifactStatus {
    fn default() -> Self {
        ArtifactStatus::NotScheduled
    }
}

fn to_manifest_with_optional_args(name: &str, tasks: Vec<manifest::TaskManifest>, params: Option<Vec<manifest::Param>>, results: Option<Vec<manifest::ParamValue>>) -> manifest::Manifest {
    let params = params.unwrap_or(Vec::new());
    let results = results.unwrap_or(Vec::new());
    to_manifest(name, tasks, params, results)
}

fn to_manifest(name: &str, tasks: Vec<manifest::TaskManifest>, params: Vec<manifest::Param>, results: Vec<manifest::ParamValue>) -> manifest::Manifest {
    let task_refs: Vec<manifest::TaskDef> = tasks.iter().map(|v|manifest::TaskDef{name: v.name.clone(), task_ref: manifest::TaskRef{name: v.name.clone()}, run_after: v.run_after.clone(), params: v.param_values.clone()}).collect();
    let mut task_defs = Vec::<manifest::Task>::new();
    for task_def in &tasks {
        task_defs.push(manifest::Task{
            api_version: manifest::TEKTON_DEV_V1,
            kind: "Task",
            metadata: manifest::Metadata {
                name: task_def.name.clone()
            },
            spec: task_def.spec.clone()
        })
    }
    manifest::Manifest {
        pipeline: manifest::Pipeline {
            api_version: "tekton.dev/v1",
            kind: "Pipeline",
            metadata: manifest::Metadata { name: name.to_owned() },
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
        Some(vec![manifest::ParamValue{name: String::from("abckl"), value: String::from("value1")}]) //TODO: How to save the results
    }
}

impl SecretRef {
    pub fn get_data(&self) -> Option<manifest::Secret> {
        Some(manifest::Secret::new(self.name.clone(), DEFAULT_NAMESPACE, HashMap::from([("k1", "v1")])))
    }
}

impl AccountRef {
    pub fn get_data(&self) -> Option<manifest::Secret> {
        Some(manifest::Secret::new(self.name.clone(), DEFAULT_NAMESPACE, HashMap::from([("k1", "v1")])))
    }
}

impl ToString for ArtifactStatus {
    fn to_string(&self) -> String {
        match self {
            Self::NotScheduled => "NotScheduled",
            Self::Running => "Running",
            Self::PendingAccount => "PendingAccount",
            Self::PendingArtRef => "PendingArtRef",
            Self::Failed => "Failed",
            Self::Succeeded=> "Succeeded"
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
            "Failed" => Self::Failed,
            "Succeeded" => Self::Succeeded,
            _ => Self::NotScheduled
        }
    }
}

pub trait Validable {
    fn validate(&mut self) -> error::Result<()>;
}

pub struct ArtifactValidator<'a> {
    pub conn: &'a mut PgConnection,
    pub artifact: &'a mut ArtifactRequest
}

impl <'a>Validable for ArtifactValidator<'a> {
    fn validate(&mut self) -> error::Result<()> {
        if let Some(refs) = &self.artifact.refs {
            let mut err_msg = String::new();
            for art_ref in refs {
                //TODO: check if the reference exists
                if !dao::ArtifactDao::exist_name(self.conn, &art_ref.name)? {
                    err_msg += &format!("Unable to find the artifact: {}\n", art_ref.name);
                }
            }
            if !err_msg.is_empty() {
                return Err(error::error(&err_msg));
            }
        }

        if let Some(accounts) = &self.artifact.build.accounts {
            let mut err_msg = String::new();
            for account in accounts {
                if !dao::AccountDao::exist_name(self.conn, &account.name)? {
                    err_msg += &format!("Unable to find the account: {}\n", account.name);
                }

                //TODO: check if the account exists
                //TODO: check if an account is available
            }
            if !err_msg.is_empty() {
                return Err(error::error(&err_msg));
            }
        }

        if let Some(accounts) = &self.artifact.clean.accounts {
            let mut err_msg = String::new();
            for account in accounts {
                if !dao::AccountDao::exist_name(self.conn, &account.name)? {
                    err_msg += &format!("Unable to find the account: {}\n", account.name);
                }

                //TODO: check if the account exists
                //TODO: check if an account is available
            }
            if !err_msg.is_empty() {
                return Err(error::error(&err_msg));
            }
        }

        if let Some(secrets) = &self.artifact.build.secrets {
            let mut err_msg = String::new();
            for secret in secrets {
                if !dao::SecretDao::exist_name(self.conn, &secret.name)? {
                    err_msg += &format!("Unable to find the secret: {}\n", secret.name);
                }
            }
            if !err_msg.is_empty() {
                return Err(error::error(&err_msg));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time;
    /*
    #[test]
    fn test_rollout() {
        pipeline::delete_run("--all", "train").unwrap();
        let request = r#"{"name":"opsman-art","total":1,"target":1,"refs":[{"name":"mock"}],"build":{"tasks":[{"name":"arttest-task1","spec":{"steps":[{"name":"step1","image":"ubuntu","script":"echo $(params.name)\necho with art_id:"}],"params":[{"name":"name","type":"string","description":"The username"}]},"paramValues":[{"name":"name","value":"John"}]}],"params":[{"name":"art_id","type":"string","description":"The artifact ID"},{"name":"inst_id","type":"string","description":"The instance ID"}],"secrets":[{"name":"aws-route53"},{"name":"pivnet"}],"accounts":[{"name":"gcp-environment"}]},"clean":{"tasks":[{"name":"task1","spec":{"steps":[{"name":"step1","image":"ubuntu","script":"echo $(params.name)\necho with art_id:"}],"params":[{"name":"name","type":"string","description":"The username"}]},"paramValues":[{"name":"name","value":"John"}]}],"params":[{"name":"art_id","type":"string","description":"The artifact ID"},{"name":"inst_id","type":"string","description":"The instance ID"}],"secrets":[{"name":"aws_route53"},{"name":"pivnet"}],"accounts":[{"name":"gcp_environment"}]}}"#;

        let mut request: ArtifactRequest = serde_json::from_str(request).expect("Failed to deserialize the payload to ArtifactRequest object");
        request.format().expect("Failed to format the artifact request");

        let mut artifact = Artifact::try_from(request).expect("Failed to deserialize artifact from artifact request");
        //let manifest_yaml = artifact.build.manifest;
        //pipeline::apply(manifest_yaml, DEFAULT_NAMESPACE).expect("Fail to apply manifest to kubernetets");
        let instances = artifact.build.run(1).expect("Failed to roll out the artifct");
        //let params: Vec<&str> = vec!["art_id=opsman", "inst_id=warn-ma20"];
        //let run_name = pipeline::run(&artifact.build.name, "train", params).unwrap();
        let pipelines = pipeline::list("train").expect("failed to list the pipelines");
        assert!(pipelines.len() >= 1);
        assert!(pipelines.iter().any(|x| x == "build-opsman-warn-ma20"));
        std::thread::sleep(time::Duration::from_secs(1));
        let logs = pipeline::logs(&instances[0].run_name, "train").expect("Failed to acquire logs");
        println!("### logs: \n{}", logs);
        assert!(logs.contains("John"));
        //pipeline::delete_run("--all", "train").unwrap();
    }
    */
}
