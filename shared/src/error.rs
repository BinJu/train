use std::fmt::Display;

use actix_web::HttpResponse;

pub type Result<T> = std::result::Result<T, GeneralError>;

#[derive(Debug)]
pub enum GeneralError {
    Error(String),
    IoError(std::io::Error),
    ProcessError(String),
    PendingArtRef,
    PendingAccount,
    PipelineError(String),
    RedisError(redis::RedisError),
    SerdeJsonError(serde_json::Error),
    SerdeYamlError(serde_yaml::Error),
    DBError(diesel::result::Error)
}

#[inline]
pub fn error(msg: &str) -> GeneralError {
    GeneralError::Error(msg.to_owned())
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

impl From<String> for GeneralError {
    fn from(value: String) -> Self {
        GeneralError::Error(value)
    }
}

impl From<&str> for GeneralError {
    fn from(value: &str) -> Self {
        GeneralError::Error(value.to_owned())
    }
}

impl From<diesel::result::Error> for GeneralError {
    fn from(value: diesel::result::Error) -> Self {
        GeneralError::DBError(value)
    }
}

impl actix_web::error::ResponseError for GeneralError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::from_u16(500).unwrap()
    }

    fn error_response(&self) -> HttpResponse<actix_http::body::BoxBody> {
        //TODO: return true response
        HttpResponse::build(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR).body(format!("{}", self)) 
    }
}

impl Display for GeneralError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Error(err) => f.write_fmt(format_args!("Error: {}", err))?,
            Self::IoError(err) => f.write_fmt(format_args!("IoError: {}", err))?,
            Self::ProcessError(desc) => f.write_fmt(format_args!("ProcessError: {}", desc))?,
            Self::PendingArtRef => f.write_fmt(format_args!("PendingArtRef"))?,
            Self::PendingAccount => f.write_fmt(format_args!("PendingAccount"))?,
            Self::PipelineError(desc) => f.write_fmt(format_args!("PipelineError: {}", desc))?,
            Self::RedisError(desc) => f.write_fmt(format_args!("RedisError: {}", desc))?,
            Self::SerdeJsonError(err) => f.write_fmt(format_args!("SerdeJsonError: {}", err))?,
            Self::SerdeYamlError(err) => f.write_fmt(format_args!("SerdeYamlError: {}", err))?,
            Self::DBError(err) => f.write_fmt(format_args!("DB Error: {}", err))?
        };
        Ok(())
    }
}

impl std::error::Error for GeneralError {}
