use anyhow::Result;
use std::path::Path;

use dts::{
    de::{Deserializer, DeserializerBuilder},
    ser::{Serializer, SerializerBuilder},
    Encoding,
};

use Encoding::*;

fn transcode<T>(input: T, de: Deserializer, ser: Serializer) -> Result<String>
where
    T: AsRef<[u8]>,
{
    let mut input = input.as_ref();
    let value = de.deserialize(&mut input)?;
    let mut output: Vec<u8> = Vec::new();
    ser.serialize(&mut output, &value)?;
    Ok(std::str::from_utf8(&output)?.to_owned())
}

fn fixture<P: AsRef<Path>>(path: P) -> String {
    std::fs::read_to_string(path).unwrap()
}

#[test]
fn test_transcode() {
    assert_eq!(
        transcode(
            fixture("tests/fixtures/simple.json"),
            DeserializerBuilder::new().build(Json),
            SerializerBuilder::new().build(Yaml),
        )
        .unwrap(),
        fixture("tests/fixtures/simple.yaml"),
    );

    assert_eq!(
        transcode(
            fixture("tests/fixtures/simple.yaml"),
            DeserializerBuilder::new().build(Yaml),
            SerializerBuilder::new().build(Json),
        )
        .unwrap(),
        r#"{"foo":"bar"}"#.to_string(),
    );

    assert_eq!(
        transcode(
            fixture("tests/fixtures/simple.yaml"),
            DeserializerBuilder::new().build(Yaml),
            SerializerBuilder::new()
                .pretty(true)
                .newline(true)
                .build(Json),
        )
        .unwrap(),
        fixture("tests/fixtures/simple.pretty.json"),
    );

    assert_eq!(
        transcode(
            fixture("tests/fixtures/simple.yaml"),
            DeserializerBuilder::new().build(Yaml),
            SerializerBuilder::new().newline(true).build(Json),
        )
        .unwrap(),
        fixture("tests/fixtures/simple.json"),
    );
}

#[test]
fn test_transcode_csv() {
    assert_eq!(
        transcode(
            "header00,header01\nrow00,row01\nrow10,row11",
            DeserializerBuilder::new().build(Csv),
            SerializerBuilder::new().build(Json),
        )
        .unwrap(),
        r#"[["row00","row01"],["row10","row11"]]"#,
    );

    assert_eq!(
        transcode(
            "row00,row01\nrow10,row11",
            DeserializerBuilder::new()
                .csv_without_headers(true)
                .build(Csv),
            SerializerBuilder::new().build(Json),
        )
        .unwrap(),
        r#"[["row00","row01"],["row10","row11"]]"#,
    );

    assert_eq!(
        transcode(
            "header00,header01\nrow00,row01\nrow10,row11",
            DeserializerBuilder::new()
                .csv_headers_as_keys(true)
                .build(Csv),
            SerializerBuilder::new().build(Json),
        )
        .unwrap(),
        r#"[{"header00":"row00","header01":"row01"},{"header00":"row10","header01":"row11"}]"#,
    );

    assert_eq!(
        transcode(
            r#"[{"header00":"row00","header01":"row01"},{"header00":"row10","header01":"row11"}]"#,
            DeserializerBuilder::new().build(Json),
            SerializerBuilder::new()
                .keys_as_csv_headers(true)
                .build(Csv),
        )
        .unwrap(),
        "header00,header01\nrow00,row01\nrow10,row11\n",
    );

    assert_eq!(
        transcode(
            r#"[{"header00":"row00","header01":"row01"},{"header00":"row10","other":"row11"}]"#,
            DeserializerBuilder::new().build(Json),
            SerializerBuilder::new()
                .keys_as_csv_headers(true)
                .build(Csv),
        )
        .unwrap(),
        "header00,header01\nrow00,row01\nrow10,\n",
    );
}

#[test]
fn test_transcode_tsv() {
    assert_eq!(
        transcode(
            "header00\theader01\nrow00\trow01\nrow10\trow11",
            DeserializerBuilder::new().build(Tsv),
            SerializerBuilder::new().build(Json),
        )
        .unwrap(),
        r#"[["row00","row01"],["row10","row11"]]"#,
    );

    assert_eq!(
        transcode(
            "header00\theader01\nrow00\trow01\nrow10\trow11",
            DeserializerBuilder::new()
                .csv_headers_as_keys(true)
                .build(Tsv),
            SerializerBuilder::new().build(Json),
        )
        .unwrap(),
        r#"[{"header00":"row00","header01":"row01"},{"header00":"row10","header01":"row11"}]"#,
    );
}

#[test]
fn test_transcode_query_string() {
    assert_eq!(
        transcode(
            "&foo=bar&baz[0]=qux&baz[1]=foo&",
            DeserializerBuilder::new().build(QueryString),
            SerializerBuilder::new().build(Json),
        )
        .unwrap(),
        r#"{"baz":["qux","foo"],"foo":"bar"}"#,
    );

    assert_eq!(
        transcode(
            r#"{"baz":["qux","foo"],"foo":"bar"}"#,
            DeserializerBuilder::new().build(Json),
            SerializerBuilder::new().build(QueryString),
        )
        .unwrap(),
        "baz[0]=qux&baz[1]=foo&foo=bar",
    );
}

#[test]
fn test_deserialize_errors() {
    assert!(transcode(
        "invalidjson",
        DeserializerBuilder::new().build(Json),
        SerializerBuilder::new().build(Yaml),
    )
    .is_err());
}

#[test]
fn test_serialize_errors() {
    assert!(transcode(
        r#"[["header00","row00","header01","row01"],["header00","row10","header01","row11"]]"#,
        DeserializerBuilder::new().build(Json),
        SerializerBuilder::new()
            .keys_as_csv_headers(true)
            .build(Csv),
    )
    .is_err());

    assert!(transcode(
        r#"{"header00":"row00","header01":"row01"}"#,
        DeserializerBuilder::new().build(Json),
        SerializerBuilder::new()
            .keys_as_csv_headers(true)
            .build(Csv),
    )
    .is_err());
}
