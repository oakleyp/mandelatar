#[derive(Debug, Clone, PartialEq)]
pub enum InvalidPostProcessConfig {
    Default { message: String },
}

impl std::fmt::Display for InvalidPostProcessConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InvalidPostProcessConfig::Default { message } => {
                write!(f, "Failed to process image query params: {}", message)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImageProcessingError {
    Default { message: String },
}

impl std::fmt::Display for ImageProcessingError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ImageProcessingError::Default { message } => {
                write!(f, "Failed to process image: {}", message)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImagePostProcessingError {
    Default { message: String },
}

impl std::fmt::Display for ImagePostProcessingError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ImagePostProcessingError::Default { message } => {
                write!(f, "Failed to perform image post-processing: {}", message)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImageLoaderError {
    FailedLoad { message: String },
}

impl std::fmt::Display for ImageLoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ImageLoaderError::FailedLoad { message } => {
                write!(f, "Failed to load image: {}", message)
            }
        }
    }
}
