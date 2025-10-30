use hauski_backend::validation::is_allowed_uri;

#[test]
fn uri_regex_behaves() {
    assert!(is_allowed_uri("qobuz:track:42"));
    assert!(is_allowed_uri("spotify:album:1"));
    assert!(is_allowed_uri("local:/foo/bar.flac"));
    assert!(!is_allowed_uri("qobuz:"));
    assert!(!is_allowed_uri("file:///tmp/x"));
    assert!(!is_allowed_uri(""));
}
