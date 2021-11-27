use criterion::{criterion_group, criterion_main, Criterion};
use dts::transform::*;
use serde_json::json;

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

            flatten_keys(&value, "data")
        })
    });

    c.bench_function("deep_merge", |b| {
        b.iter(|| {
            deep_merge(json!([{"foo": "bar"},
                       {"foo": {"bar": "baz"}, "bar": [1], "qux": null},
                       {"foo": {"bar": "qux"}, "bar": [2], "baz": 1},
                       {"bar": {"bar": "baz", "bam": "boo"}, "bar": [null, 1], "qux": null},
                       {"bar": {"bar": "qux", "buz": "foo"}, "bar": [2], "baz": 1}]))
        })
    });
}

criterion_group!(benches, benchmark_transform);
criterion_main!(benches);
