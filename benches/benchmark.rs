use criterion::{criterion_group, criterion_main, Criterion};
use dts::transform::*;
use serde_json::json;

fn benchmark_transform(c: &mut Criterion) {
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
            deep_merge(&json!([{"foo": "bar"},
                       {"foo": {"bar": "baz"}, "bar": [1], "qux": null},
                       {"foo": {"bar": "qux"}, "bar": [2], "baz": 1},
                       {"bar": {"bar": "baz", "bam": "boo"}, "bar": [null, 1], "qux": null},
                       {"bar": {"bar": "qux", "buz": "foo"}, "bar": [2], "baz": 1}]))
        })
    });
}

criterion_group!(benches, benchmark_transform);
criterion_main!(benches);
