use anyhow::Result;

use trnscd::{
    de::{DeserializeOptions, Deserializer},
    ser::{SerializeOptions, Serializer},
    Encoding,
};

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

#[test]
fn test_transcode() {
    use Encoding::*;

    assert_transcode("{\"foo\":\"bar\"}", "---\nfoo: bar\n", Json, Yaml);
    assert_transcode("---\nfoo: bar\n", "{\"foo\":\"bar\"}", Yaml, Json);
    assert_transcode_opts(
        "---\nfoo: bar\n",
        "{\n  \"foo\": \"bar\"\n}",
        Yaml,
        Json,
        DeserializeOptions::default(),
        SerializeOptions {
            pretty: true,
            newline: false,
        },
    );
    assert_transcode_opts(
        "---\nfoo: bar\n",
        "{\"foo\":\"bar\"}\n",
        Yaml,
        Json,
        DeserializeOptions::default(),
        SerializeOptions {
            pretty: false,
            newline: true,
        },
    );
}
