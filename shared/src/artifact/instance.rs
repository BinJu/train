use std::collections::HashMap;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Instance {
    pub id: String,
    pub art_id: String,
    pub run_name: String,
    pub dirt: bool,
    //TODO: Tekton is based on async deploy, You may not get the final status immediately. There
    //must be some process/thread to keep them udpated.
    pub stat: InstanceStatus,
    pub results: Option<HashMap<String,String>>//TODO: abstract the result from pipeline run.
}

#[derive(Debug,  PartialEq, Clone)]
pub enum InstanceStatus {
    Unknown,
    Running,
    Failed(String),
    Succeeded
}

pub struct InstanceNumbers {
    pub running: u32,
    pub fail: u32,
    pub done_clean: u32,
    pub done_dirt: u32
}

impl Default for InstanceStatus {
    fn default() -> Self {
        InstanceStatus::Unknown
    }
}

impl <S:AsRef<str>>From<S> for InstanceStatus {
    fn from(value: S) -> Self {
        match value.as_ref() {
            "Running" => InstanceStatus::Running,
            "Succeeded" => InstanceStatus::Succeeded,
            "" => InstanceStatus::Unknown,
            _ => InstanceStatus::Failed(value.as_ref().to_owned()),
        }
    }
}

impl ToString for InstanceStatus {
    fn to_string(&self) -> String {
        match self {
            Self::Unknown => "Unknown".to_owned(),
            Self::Running => "Running".to_owned(),
            Self::Failed(reason) => format!("Fail: {}", reason),
            Self::Succeeded => "Succeeded".to_owned()
        }
    }
}
