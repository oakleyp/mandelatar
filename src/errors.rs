use actix_web::{
    error,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};

#[derive(Debug)]
pub enum UserError {
    ValidationError { message: String },
    InternalError,
}

impl std::fmt::Display for UserError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            UserError::ValidationError { message } => write!(f, "Validation Error: {}", message),
            UserError::InternalError => write!(f, "An internal server error occurred"),
        }
    }
}

impl error::ResponseError for UserError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            UserError::ValidationError { .. } => StatusCode::BAD_REQUEST,
            UserError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
