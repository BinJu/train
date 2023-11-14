use serde::{Serialize, Deserialize};
use crate::error;
use std::collections::HashMap;

pub const TEKTON_DEV_V1: &str = "tekton.dev/v1";

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct TaskManifest<'a> {
    pub name: String,
    pub spec: TaskSpec<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "paramValues", deserialize = "paramValues"))]
    pub param_values: Option<Vec<ParamValue<'a>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "runAfter", deserialize = "runAfter"))]
    pub run_after: Option<Vec<&'a str>>
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct Task<'a> {
    #[serde(rename(serialize = "apiVersion", deserialize = "apiVersion"))]
    pub api_version: &'a str,
    pub kind: &'a str,
    pub metadata: Metadata,
    pub spec: TaskSpec<'a>
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub name: String
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct TaskSpec<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub results: Option<Vec<TaskResult<'a>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params : Option<Vec<Param<'a>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "stepTemplate", deserialize = "stepTemplate"))]
    pub step_template: Option<StepTemplate<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volumes: Option<Vec<Volume<'a>>>,
    pub steps: Vec<TaskStep<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    // TODO:
    // securityContext:
    //  privileged: true
    pub sidecars: Option<Vec<TaskStep<'a>>>
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct StepTemplate<'a> {
    pub env: Vec<TaskStepEnvKV<'a>>
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct Volume<'a> {
    pub name: &'a str,
    #[serde(flatten)]
    pub volume_type: VolumeType<'a>
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum VolumeType<'a> {
    #[serde(rename(serialize = "hostPath", deserialize = "hostPath"))]
    HostPath(HostPath<'a>),
    #[serde(rename(serialize = "emptyDir", deserialize = "emptyDir"))]
    EmptyDir(&'a str),
    #[serde(rename(serialize = "configMap", deserialize = "configMap"))]
    ConfigMap(ConfigMapRef<'a>),
    #[serde(rename(serialize = "secret", deserialize = "secret"))]
    Secret(SecretRef<'a>)
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct ConfigMapRef<'a> {
    pub name: &'a str
}
#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct HostPath<'a> {
    path: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "type", deserialize = "type"))]
    tpe: Option<&'a str>
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct SecretRef<'a> {
    #[serde(rename(serialize = "secretName", deserialize = "secretName"))]
    secret_name: &'a str
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct TaskResult<'a> {
    pub name: &'a str,
    pub description: &'a str
}

// TODO: Add environment and step template
#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct TaskStep<'a> {
    pub name: &'a str,
    pub image: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<&'a str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<Vec<TaskStepEnvKV<'a>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "volumeMounts", deserialize = "volumeMounts"))]
    pub volume_mounts: Option<Vec<VolumeMount<'a>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "computeResource", deserialize = "computeResource"))]
    pub compute_resources: Option<ComputeResource<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<&'a str>, //e.g.: 5s
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "onError", deserialize = "onError"))]
    pub on_error: Option<&'a str>, //continue or stopAndFail
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "stdoutConfig", deserialize = "stdoutConfig"))]
    pub stdout_config: Option<OutputPath<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "stderrConfig", deserialize = "stderrConfig"))]
    pub stderr_config: Option<OutputPath<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "securityContext", deserialize = "securityContext"))]
    pub security_context: Option<SecurityContext>
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    pub privileged: bool
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct TaskStepEnvKV<'a> {
    pub name: &'a str,
    #[serde(flatten)]
    pub value: EnvValue<'a>
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum EnvValue<'a> {
    #[serde(rename(serialize = "value", deserialize = "value"))]
    Value(&'a str),
    #[serde(rename(serialize = "sercretKeyRef", deserialize = "secretKeyRef"))]
    SecretKeyRef(SecretKeyRef<'a>)
}

impl <'a>Default for EnvValue<'a> {
    fn default() -> Self {
        EnvValue::Value("")
    }
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct SecretKeyRef<'a> {
    pub name: &'a str,
    pub key: &'a str
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct ComputeResource<'a> {
    pub requests: ResourceDescription<'a>,
    pub limits: ResourceDescription<'a>
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct ResourceDescription<'a> {
    cpu: &'a str,
    mem: &'a str
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct OutputPath<'a> {
    path: &'a str
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct VolumeMount<'a> {
    pub name: &'a str,
    #[serde(rename(serialize = "mountPath", deserialize = "mountPath"))]
    pub mount_path: &'a str
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct Param<'a> {
    pub name: &'a str,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tpe: Option<&'a str>,
    pub description: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<&'a str>,
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct Pipeline<'a> {
    #[serde(rename(serialize = "apiVersion", deserialize = "apiVersion"))]
    pub api_version: &'a str,
    pub kind: &'a str,
    pub metadata: Metadata,
    pub spec: PipelineSpec<'a>
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct PipelineSpec<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params : Option<Vec<Param<'a>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub results: Option<Vec<ParamValue<'a>>>,
    pub tasks: Vec<TaskDef<'a>>
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct TaskDef<'a> {
    pub name: String,
    #[serde(rename(serialize = "taskRef", deserialize = "taskRef"))]
    pub task_ref: TaskRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "runAfter", deserialize = "runAfter"))]
    pub run_after: Option<Vec<&'a str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Vec<ParamValue<'a>>>,

}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct TaskRef {
    pub name: String
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct ParamValue<'a> {
    pub name: &'a str,
    pub value: &'a str
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

impl <'a>Default for VolumeType<'a> {
    fn default() -> Self {
        VolumeType::EmptyDir("{}")
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
/*
pub fn art_system_tasks(mut manifest: Manifest) -> error::Result<Manifest> {
    let pipeline_params = vec![
        Param {
            name: "art_id",
            tpe: Some("string"),
            description: "The artifact ID that is allocated from the system",
            default: None
        },
        Param {
            name: "art_name",
            tpe: Some("string"),
            description: "The artifact name that is allocated from the system",
            default: None
        },
        Param {
            name: "meta_ref",
            tpe: Some("string"),
            description: "The metadata that the artifact relies on",
            default: None
        },
    ];

    manifest.pipeline.spec.params = Some(pipeline_params.clone());

    let param_values = vec![
        ParamValue {
            name: "art_id",
            value: "$(params.art_id)"
        },
        ParamValue {
            name: "art_name",
            value: "$(params.art_name)"
        },
        ParamValue {
            name: "meta_ref",
            value: "$(params.meta_ref)"
        },
    ];

    for p in &mut manifest.pipeline.spec.tasks {
        let mut overloaded_param_values = param_values.clone();
        if let Some(task_param_values) = &mut p.params {
            overloaded_param_values.extend_from_slice(&task_param_values);
        }
        p.params = Some(overloaded_param_values);
    }

    for p in &mut manifest.tasks {
        let mut overloaded_params = pipeline_params.clone();
        if let Some(task_param_values) = &mut p.spec.params {
            overloaded_params.extend_from_slice(&task_param_values);
        }
        p.spec.params = Some(overloaded_params);
    }
    Ok(manifest)
}
*/

#[cfg(test)]
mod tests {

/*    #[test]
    fn test_art_system_tasks() {
        let manifest = Manifest {
            pipeline: Pipeline{
                    api_version: TEKTON_DEV_V1,
                    kind: "Pipeline",
                    metadata: Metadata{name: "task1"},
                    spec: PipelineSpec {
                        tasks: vec![
                            TaskDef {
                                name: "task1",
                                task_ref: TaskRef { name: "task_def_1" },
                                ..Default::default()
                            },
                            TaskDef {
                                name: "task2",
                                task_ref: TaskRef { name: "task_def_2" },
                                params: Some(vec![
                                    ParamValue {
                                        name: "city",
                                        value: "Toronto"
                                    }
                                ]),
                                ..Default::default()
                            }
                        ],
                        ..Default::default()
                    }
            },
            tasks: vec![
                Task {
                    api_version: TEKTON_DEV_V1,
                    kind: "Task",
                    metadata: Metadata{name: "task_def_1"},
                    spec: TaskSpec { steps: vec![TaskStep{name: "task1_step1", image: "ubuntu", ..Default::default()}], ..Default::default() }
                },
                Task {
                    api_version: TEKTON_DEV_V1,
                    kind: "Task",
                    metadata: Metadata{name: "task_def_2"},
                    spec: TaskSpec { steps: vec![TaskStep{name: "task2_step1", image: "ubuntu", ..Default::default()}], params: Some(vec![Param{name: "city", description: "city name", ..Default::default()}]), ..Default::default() }
                }
            ]
        };

        let processed_manifest = art_system_tasks(manifest).expect("Failed to process the manifest");
        assert!(processed_manifest.pipeline.spec.params.is_some());
        let pipeline_params = processed_manifest.pipeline.spec.params.unwrap();
        assert_eq!(pipeline_params.len(), 3);
        assert!(vec!["art_id", "art_name", "meta_ref"].contains(&&*pipeline_params[0].name));
        assert!(vec!["art_id", "art_name", "meta_ref"].contains(&&*pipeline_params[1].name));
        assert!(vec!["art_id", "art_name", "meta_ref"].contains(&&*pipeline_params[2].name));
        
        let params = &processed_manifest.pipeline.spec.tasks[0].params;
        assert!(params.is_some());
        if let Some(ref p) = params {
            assert_eq!(p.len(), 3);
        }

        let params = &processed_manifest.pipeline.spec.tasks[1].params;
        assert!(params.is_some());
        if let Some(ref p) = params {
            assert_eq!(p.len(), 4);
        }

        let params = &processed_manifest.tasks[0].spec.params;
        assert!(params.is_some());
        if let Some(ref p) = params {
            assert_eq!(p.len(), 3);
        }

        let params = &processed_manifest.tasks[1].spec.params;
        assert!(params.is_some());
        if let Some(ref p) = params {
            assert_eq!(p.len(), 4);
        }

    }
*/
}
