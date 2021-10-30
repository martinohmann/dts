use std::path::PathBuf;
use trnscd::{detect_encoding, Encoding};

#[test]
fn test_encoding_from_path() {
    assert_eq!(Encoding::from_path("foo.yaml"), Some(Encoding::Yaml));
    assert_eq!(Encoding::from_path("foo.yml"), Some(Encoding::Yaml));
    assert_eq!(Encoding::from_path("foo.json"), Some(Encoding::Json));
    assert_eq!(Encoding::from_path("foo.json5"), Some(Encoding::Json5));
    assert_eq!(Encoding::from_path("foo.ron"), Some(Encoding::Ron));
    assert_eq!(Encoding::from_path("foo.toml"), Some(Encoding::Toml));
    assert_eq!(Encoding::from_path("foo.hjson"), Some(Encoding::Hjson));
    assert_eq!(Encoding::from_path("foo.bak"), None);
    assert_eq!(Encoding::from_path("foo"), None);
}

#[test]
fn test_detect_encoding() {
    assert_eq!(detect_encoding::<PathBuf>(None, None), None);
    assert_eq!(
        detect_encoding::<PathBuf>(Some(Encoding::Yaml), None),
        Some(Encoding::Yaml)
    );
    assert_eq!(
        detect_encoding(Some(Encoding::Yaml), Some("foo.json")),
        Some(Encoding::Yaml)
    );
    assert_eq!(
        detect_encoding(None, Some("foo.json")),
        Some(Encoding::Json)
    );
    assert_eq!(detect_encoding(None, Some("foo.bak")), None);
}
