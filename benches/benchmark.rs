use assert_cmd::Command;
use criterion::{criterion_group, criterion_main, Criterion};
use dts_core::key::*;
use dts_json::json;

fn benchmark_transform(c: &mut Criterion) {
    c.bench_function("expand_keys", |b| {
        b.iter(|| {
            let value = json!({
                "foo": [],
                "foo.bar[0]": "baz",
                "foo.bar[1]": "qux",
                "bar": {},
                "bar.qux": [],
                "bar.qux[0]": null,
                "bar.qux[10]": "qux",
                "qux": {},
                "qux.one": [],
                "qux.one[0]": {},
                "qux.one[0].two": 3,
                "qux.one[1]": {},
                "qux.one[3].four": [],
                "qux.one[3].four[0]": "five",
                "qux.one[30].four[1]": 6,
                "bam": [],
                "bam[0]": "a",
                "bam[1]": "b",
                "bam[5]": [],
                "bam[5][0]": "c",
                "bam[5][1]": "d",
                "bam[5][6]": "e",
                "adsf[\"foo\\\"-bar\"].baz[\"very\"][10].deep": 42,
                "adsf[\"foo\\\"-bar\"].buz[\"adsf\"].foo": null,
            });

            expand_keys(value)
        })
    });

    c.bench_function("flatten_keys", |b| {
        b.iter(|| {
            let value = json!({
                "foo": {
                    "bar": ["baz", "qux"]
                },
                "bar": {
                    "qux": [null, "qux"]
                },
                "qux": {
                    "one": [
                        {"two": 3},
                        {"four": ["five", 6]}
                    ]
                },
                "bam": ["a", "b", ["c", "d", "e"]]
            });

            flatten_keys(value, "data")
        })
    });

    c.bench_function("dts", |b| {
        b.iter(|| {
            Command::cargo_bin("dts")
                .unwrap()
                .arg("tests/fixtures")
                .args(&["--glob", "*", "-C", "-j", ".[]"])
                .assert()
                .success()
        })
    });
}

criterion_group!(benches, benchmark_transform);
criterion_main!(benches);
