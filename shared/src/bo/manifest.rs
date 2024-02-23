use serde::{Serialize, Deserialize};
use crate::error;
use std::collections::HashMap;

pub const TEKTON_DEV_V1: &str = "tekton.dev/v1";

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct TaskManifest {
    pub name: String,
    pub spec: TaskSpec,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "paramValues", deserialize = "paramValues"))]
    pub param_values: Option<Vec<ParamValue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "runAfter", deserialize = "runAfter"))]
    pub run_after: Option<Vec<String>>
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct Task<'a> {
    #[serde(rename(serialize = "apiVersion", deserialize = "apiVersion"))]
    pub api_version: &'a str,
    pub kind: &'a str,
    pub metadata: Metadata,
    pub spec: TaskSpec
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub name: String
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct TaskSpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub results: Option<Vec<TaskResult>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params : Option<Vec<Param>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "stepTemplate", deserialize = "stepTemplate"))]
    pub step_template: Option<StepTemplate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volumes: Option<Vec<Volume>>,
    pub steps: Vec<TaskStep>,
    #[serde(skip_serializing_if = "Option::is_none")]
    // TODO:
    // securityContext:
    //  privileged: true
    pub sidecars: Option<Vec<TaskStep>>
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct StepTemplate {
    pub env: Vec<TaskStepEnvKV>
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct Volume {
    pub name: String,
    #[serde(flatten)]
    pub volume_type: VolumeType
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum VolumeType {
    #[serde(rename(serialize = "hostPath", deserialize = "hostPath"))]
    HostPath(HostPath),
    #[serde(rename(serialize = "emptyDir", deserialize = "emptyDir"))]
    EmptyDir(String),
    #[serde(rename(serialize = "configMap", deserialize = "configMap"))]
    ConfigMap(ConfigMapRef),
    #[serde(rename(serialize = "secret", deserialize = "secret"))]
    Secret(SecretRef)
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct ConfigMapRef {
    pub name: String
}
#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct HostPath {
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "type", deserialize = "type"))]
    tpe: Option<String>
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct SecretRef {
    #[serde(rename(serialize = "secretName", deserialize = "secretName"))]
    secret_name: String
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub name: String,
    pub description: String
}

// TODO: Add environment and step template
#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct TaskStep {
    pub name: String,
    pub image: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<Vec<TaskStepEnvKV>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "volumeMounts", deserialize = "volumeMounts"))]
    pub volume_mounts: Option<Vec<VolumeMount>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "computeResource", deserialize = "computeResource"))]
    pub compute_resources: Option<ComputeResource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<String>, //e.g.: 5s
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "onError", deserialize = "onError"))]
    pub on_error: Option<String>, //continue or stopAndFail
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "stdoutConfig", deserialize = "stdoutConfig"))]
    pub stdout_config: Option<OutputPath>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "stderrConfig", deserialize = "stderrConfig"))]
    pub stderr_config: Option<OutputPath>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "securityContext", deserialize = "securityContext"))]
    pub security_context: Option<SecurityContext>
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    pub privileged: bool
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct TaskStepEnvKV {
    pub name: String,
    #[serde(flatten)]
    pub value: EnvValue
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum EnvValue {
    #[serde(rename(serialize = "value", deserialize = "value"))]
    Value(String),
    #[serde(rename(serialize = "sercretKeyRef", deserialize = "secretKeyRef"))]
    SecretKeyRef(SecretKeyRef)
}

impl <'a>Default for EnvValue {
    fn default() -> Self {
        EnvValue::Value(String::new())
    }
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct SecretKeyRef {
    pub name: String,
    pub key: String
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct ComputeResource {
    pub requests: ResourceDescription,
    pub limits: ResourceDescription
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResourceDescription {
    cpu: String,
    mem: String
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct OutputPath {
    path: String
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct VolumeMount {
    pub name: String,
    #[serde(rename(serialize = "mountPath", deserialize = "mountPath"))]
    pub mount_path: String
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tpe: Option<String>,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct Pipeline<'a> {
    #[serde(rename(serialize = "apiVersion", deserialize = "apiVersion"))]
    pub api_version: &'a str,
    pub kind: &'a str,
    pub metadata: Metadata,
    pub spec: PipelineSpec
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct PipelineSpec {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params : Option<Vec<Param>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub results: Option<Vec<ParamValue>>,
    pub tasks: Vec<TaskDef>
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct TaskDef {
    pub name: String,
    #[serde(rename(serialize = "taskRef", deserialize = "taskRef"))]
    pub task_ref: TaskRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "runAfter", deserialize = "runAfter"))]
    pub run_after: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Vec<ParamValue>>,

}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct TaskRef {
    pub name: String
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct ParamValue {
    pub name: String,
    pub value: String
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct Manifest<'a>{
    pub pipeline: Pipeline<'a>,
    pub tasks: Vec<Task<'a>> 
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct Secret<'a> {
    #[serde(rename(serialize = "apiVersion", deserialize = "apiVersion"))]
    pub api_version: &'a str,
    pub kind: &'a str,
    pub metadata: SecretMetadata<'a>,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub tpe: &'a str,
    #[serde(rename(serialize = "stringData", deserialize = "stringData"))]
    pub string_data: HashMap<&'a str, &'a str>
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct SecretMetadata<'a> {
    pub name: String,
    pub namespace: &'a str
}

impl Default for VolumeType {
    fn default() -> Self {
        VolumeType::EmptyDir(String::from("{}"))
    }
}

impl <'a>Manifest<'a> {
    pub fn to_yaml(&self) -> error::Result<String> {
        let mut buff = String::new();
        for task in &self.tasks {
            buff += "---\n";
            buff += &serde_yaml::to_string(&task)?;
        }

        buff += "---\n";
        buff += &serde_yaml::to_string(&self.pipeline)?;
        Ok(buff)
    }
}

impl <'a> Secret<'a> {
    pub fn new(name: String, namespace: &'a str, kvs: HashMap<&'a str, &'a str>) -> Self {
        Secret {
            api_version: "v1",
            kind: "Secret",
            metadata: SecretMetadata {
                name,
                namespace
            },
            tpe: "Opaque",
            string_data: kvs
        }
    }
}

#[cfg(test)]
mod tests {
}
