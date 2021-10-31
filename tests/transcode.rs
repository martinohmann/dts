use anyhow::Result;
use std::path::Path;

use trnscd::{
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
    ser.serialize(&mut output, value)?;
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
            DeserializerBuilder::new(Json).build(),
            SerializerBuilder::new(Yaml).build(),
        )
        .unwrap(),
        fixture("tests/fixtures/simple.yaml"),
    );

    assert_eq!(
        transcode(
            fixture("tests/fixtures/simple.yaml"),
            DeserializerBuilder::new(Yaml).build(),
            SerializerBuilder::new(Json).build(),
        )
        .unwrap(),
        "{\"foo\":\"bar\"}".to_string(),
    );

    assert_eq!(
        transcode(
            fixture("tests/fixtures/simple.yaml"),
            DeserializerBuilder::new(Yaml).build(),
            SerializerBuilder::new(Json)
                .pretty(true)
                .newline(true)
                .build(),
        )
        .unwrap(),
        fixture("tests/fixtures/simple.pretty.json"),
    );

    assert_eq!(
        transcode(
            fixture("tests/fixtures/simple.yaml"),
            DeserializerBuilder::new(Yaml).build(),
            SerializerBuilder::new(Json).newline(true).build(),
        )
        .unwrap(),
        fixture("tests/fixtures/simple.json"),
    );
}

#[test]
fn test_transcode_csv() {
    assert_eq!(
        transcode(
            "row00,row01\nrow10,row11",
            DeserializerBuilder::new(Csv).build(),
            SerializerBuilder::new(Json).build(),
        )
        .unwrap(),
        "[[\"row00\",\"row01\"],[\"row10\",\"row11\"]]",
    );

    assert_eq!(
        transcode(
            "header00,header01\nrow00,row01\nrow10,row11",
            DeserializerBuilder::new(Csv).headers(true).build(),
            SerializerBuilder::new(Json).build(),
        ).unwrap(),
        "[{\"header00\":\"row00\",\"header01\":\"row01\"},{\"header00\":\"row10\",\"header01\":\"row11\"}]",
    );
}

#[test]
fn test_transcode_tsv() {
    assert_eq!(
        transcode(
            "row00\trow01\nrow10\trow11",
            DeserializerBuilder::new(Tsv).build(),
            SerializerBuilder::new(Json).build(),
        )
        .unwrap(),
        "[[\"row00\",\"row01\"],[\"row10\",\"row11\"]]",
    );

    assert_eq!(
        transcode(
            "header00\theader01\nrow00\trow01\nrow10\trow11",
            DeserializerBuilder::new(Tsv).headers(true).build(),
            SerializerBuilder::new(Json).build(),
        ).unwrap(),
        "[{\"header00\":\"row00\",\"header01\":\"row01\"},{\"header00\":\"row10\",\"header01\":\"row11\"}]",
    );
}

#[test]
fn test_deserialize_errors() {
    assert!(transcode(
        "invalidjson",
        DeserializerBuilder::new(Json).build(),
        SerializerBuilder::new(Yaml).build(),
    )
    .is_err());
}
