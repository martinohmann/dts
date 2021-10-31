use anyhow::Result;
use std::path::Path;

use trnscd::{
    de::{DeserializeOptions, Deserializer},
    ser::{SerializeOptions, Serializer},
    Encoding,
};

use Encoding::*;

fn transcode<T>(
    input: T,
    in_enc: Encoding,
    out_enc: Encoding,
    de_opts: DeserializeOptions,
    ser_opts: SerializeOptions,
) -> Result<Vec<u8>>
where
    T: AsRef<[u8]>,
{
    let mut input = input.as_ref();
    let de = Deserializer::new(in_enc, de_opts);
    let value = de.deserialize(&mut input)?;
    let ser = Serializer::new(out_enc, ser_opts);
    let mut output: Vec<u8> = Vec::new();
    ser.serialize(&mut output, value)?;
    Ok(output)
}

fn assert_transcode_opts<T>(
    input: T,
    expected: T,
    in_enc: Encoding,
    out_enc: Encoding,
    de_opts: DeserializeOptions,
    ser_opts: SerializeOptions,
) where
    T: AsRef<[u8]>,
{
    let output = transcode(input, in_enc, out_enc, de_opts, ser_opts).unwrap();

    assert_eq!(
        std::str::from_utf8(&output).unwrap(),
        std::str::from_utf8(expected.as_ref()).unwrap()
    )
}

fn assert_transcode<T>(input: T, expected: T, in_enc: Encoding, out_enc: Encoding)
where
    T: AsRef<[u8]>,
{
    assert_transcode_opts(
        input,
        expected,
        in_enc,
        out_enc,
        DeserializeOptions::default(),
        SerializeOptions::default(),
    );
}

fn fixture<P: AsRef<Path>>(path: P) -> String {
    std::fs::read_to_string(path).unwrap()
}

#[test]
fn test_transcode() {
    assert_transcode(
        fixture("tests/fixtures/simple.json"),
        fixture("tests/fixtures/simple.yaml"),
        Json,
        Yaml,
    );
    assert_transcode(
        fixture("tests/fixtures/simple.yaml"),
        "{\"foo\":\"bar\"}".to_string(),
        Yaml,
        Json,
    );
    assert_transcode(
        "row00,row01\nrow10,row11",
        "[[\"row00\",\"row01\"],[\"row10\",\"row11\"]]",
        Csv,
        Json,
    );
    assert_transcode_opts(
        fixture("tests/fixtures/simple.yaml"),
        fixture("tests/fixtures/simple.pretty.json"),
        Yaml,
        Json,
        DeserializeOptions::default(),
        SerializeOptions {
            pretty: true,
            newline: true,
        },
    );
    assert_transcode_opts(
        fixture("tests/fixtures/simple.yaml"),
        fixture("tests/fixtures/simple.json"),
        Yaml,
        Json,
        DeserializeOptions::default(),
        SerializeOptions {
            pretty: false,
            newline: true,
        },
    );
}

#[test]
fn test_transcode_csv() {
    assert_transcode(
        "row00,row01\nrow10,row11",
        "[[\"row00\",\"row01\"],[\"row10\",\"row11\"]]",
        Csv,
        Json,
    );

    assert_transcode_opts(
        "header00,header01\nrow00,row01\nrow10,row11",
        "[{\"header00\":\"row00\",\"header01\":\"row01\"},{\"header00\":\"row10\",\"header01\":\"row11\"}]",
        Csv,
        Json,
        DeserializeOptions {
            all_documents: false,
            headers: true,
        },
        SerializeOptions::default(),
    );
}

#[test]
fn test_transcode_tsv() {
    assert_transcode(
        "row00\trow01\nrow10\trow11",
        "[[\"row00\",\"row01\"],[\"row10\",\"row11\"]]",
        Tsv,
        Json,
    );

    assert_transcode_opts(
        "header00\theader01\nrow00\trow01\nrow10\trow11",
        "[{\"header00\":\"row00\",\"header01\":\"row01\"},{\"header00\":\"row10\",\"header01\":\"row11\"}]",
        Tsv,
        Json,
        DeserializeOptions {
            all_documents: false,
            headers: true,
        },
        SerializeOptions::default(),
    );
}

#[test]
fn test_deserialize_errors() {
    assert!(transcode(
        "invalidjson",
        Json,
        Yaml,
        DeserializeOptions::default(),
        SerializeOptions::default(),
    )
    .is_err());
}
