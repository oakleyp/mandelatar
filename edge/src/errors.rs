use std::fmt;
use worker::{Error as WorkerError, Response, Result as WorkerResult};

#[derive(Debug)]
pub enum UserError {
    ValidationError { message: String },
    InternalError,
    WorkerError { error: WorkerError },
}

impl std::fmt::Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            UserError::ValidationError { message } => write!(f, "Validation Error: {}", message),
            UserError::InternalError => write!(f, "An internal server error occurred"),
            UserError::WorkerError { error } => write!(f, "Worker error: {}", error),
        }
    }
}

pub trait ResponseError: fmt::Debug + fmt::Display {
    fn error_response(&self) -> WorkerResult<Response>;
    fn status_code(&self) -> u16;
}

impl ResponseError for UserError {
    fn error_response(&self) -> WorkerResult<Response> {
        Response::error(self.to_string(), self.status_code())
    }

    fn status_code(&self) -> u16 {
        match *self {
            UserError::ValidationError { .. } => 400,
            UserError::InternalError => 500,
            UserError::WorkerError { .. } => 500,
        }
    }
}

impl From<WorkerError> for UserError {
    fn from(error: WorkerError) -> Self {
        UserError::WorkerError { error }
    }
}
