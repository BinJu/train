use std::fmt::Display;

pub type Result<T> = std::result::Result<T, GeneralError>;

#[derive(Debug)]
pub enum GeneralError {
    IoError(std::io::Error),
    ProcessError(String),
    PipelineError(String),
    RedisError(redis::RedisError),
    SerdeJsonError(serde_json::Error),
    SerdeYamlError(serde_yaml::Error)
}

pub fn new_process_error<E: AsRef<str>>(err: E) -> GeneralError {
    let error_info: String = err.as_ref().to_string();
    GeneralError::ProcessError(error_info)
}

pub fn new_pipeline_error<E: AsRef<str>>(err: E) -> GeneralError {
    let error_info: String = err.as_ref().to_string();
    GeneralError::PipelineError(error_info)
}


impl From<serde_json::Error> for GeneralError {
    fn from(err: serde_json::Error) -> Self {
        GeneralError::SerdeJsonError(err)
    }
}

impl From<serde_yaml::Error> for GeneralError {
    fn from(err: serde_yaml::Error) -> Self {
        GeneralError::SerdeYamlError(err)
    }

}

impl From<std::io::Error> for GeneralError {
    fn from(err: std::io::Error) -> Self {
        GeneralError::IoError(err)
    }
}

impl From<redis::RedisError> for GeneralError {
    fn from(err: redis::RedisError) -> Self {
        GeneralError::RedisError(err)
    }
}

impl Display for GeneralError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::IoError(err) => f.write_fmt(format_args!("IoError: {}", err))?,
            Self::ProcessError(desc) => f.write_fmt(format_args!("ProcessError: {}", desc))?,
            Self::PipelineError(desc) => f.write_fmt(format_args!("PipelineError: {}", desc))?,
            Self::RedisError(desc) => f.write_fmt(format_args!("RedisError: {}", desc))?,
            Self::SerdeJsonError(err) => f.write_fmt(format_args!("SerdeJsonError: {}", err))?,
            Self::SerdeYamlError(err) => f.write_fmt(format_args!("SerdeYamlError: {}", err))?
        };
        Ok(())
    }
}

impl std::error::Error for GeneralError {}
