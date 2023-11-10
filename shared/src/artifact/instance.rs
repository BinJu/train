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
    Fail,
    Done
}

pub struct InstanceNumbers {
    pub running: u32,
    pub fail: u32,
    pub done: u32
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
            "Fail" => InstanceStatus::Fail,
            "Done" => InstanceStatus::Done,
                _ => InstanceStatus::Unknown
        }
    }
}

impl ToString for InstanceStatus {
    fn to_string(&self) -> String {
        match self {
            Self::Unknown => "Unknown",
            Self::Running => "Running",
            Self::Fail => "Fail",
            Self::Done => "Done"
        }.to_owned()
    }
}
