use regex::Regex;

lazy_static::lazy_static! {
    /// Erlaubte URI-Schemata: qobuz:, spotify:, local:
    /// Mindestens ein weiteres Zeichen hinter dem Schema verlangt.
    static ref URI_RE: Regex = Regex::new(r"^(?i:(qobuz|spotify|local))[:/].+").unwrap();
}

pub fn is_allowed_uri(uri: &str) -> bool {
    URI_RE.is_match(uri)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ok_schemes() {
        assert!(is_allowed_uri("qobuz:track:123"));
        assert!(is_allowed_uri("spotify:track:123"));
        assert!(is_allowed_uri("local:/music/foo.flac"));
        assert!(is_allowed_uri("LOCAL:/x"));
    }
    #[test]
    fn rejects_empty_and_plain() {
        assert!(!is_allowed_uri(""));
        assert!(!is_allowed_uri("file:///tmp/x")); // nicht freigeschaltet
        assert!(!is_allowed_uri("qobuz:"));       // nichts dahinter
    }
}
