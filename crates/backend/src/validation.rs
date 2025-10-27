use regex::Regex;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("unsupported URI scheme: {0}")]
    UnsupportedScheme(String),
    #[error("invalid URI: {0}")]
    Invalid(String),
}

pub fn validate_uri(uri: &str) -> Result<(), ValidationError> {
    // erlaubte Schemata: qobuz:, spotify:, local:
    let re = Regex::new(r"^(qobuz|spotify|local):.+$").unwrap();
    if !re.is_match(uri) {
        if let Some((scheme, _)) = uri.split_once(':') {
            return Err(ValidationError::UnsupportedScheme(scheme.into()));
        }
        return Err(ValidationError::Invalid(uri.into()));
    }
    Ok(())
}

pub fn validate_uris(uris: &[String]) -> Result<(), ValidationError> {
    for u in uris {
        validate_uri(u)?;
    }
    Ok(())
}
