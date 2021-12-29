# dts-json

[![Build Status](https://github.com/martinohmann/dts/workflows/ci/badge.svg)](https://github.com/martinohmann/dts/actions?query=workflow%3Aci)
![MIT License](https://img.shields.io/github/license/martinohmann/dts?color=blue)

This crate provides the types used by
[`dts`](https://github.com/martinohmann/dts) internally for serialization and
deserialization. The most notable types are `Value`, `Number` and `Map`.

Furthermore it also includes a version of the `serde_json::json!` macro which
produces a `dts_json::Value` instead.

These types along with their original (de-)serialization implementation were
copied from [`serde_json`](https://docs.serde.rs/serde_json/) with some
features added and removed.

Copyright for code parts that were copied verbatim remains with the original
authors of [`serde_json`](https://github.com/serde-rs/json).

**Note**: If your are looking for a generic data type to represent arbitrary
JSON data it is advised to use `serde_json::Value` instead. The API of
`dts-json` is subject to change and there are no stability guarantees.

## Why?

Historically `dts` used `serde_json::Value` as internal representation of
arbitrary data. This worked well for some time but some newer use cases require
more control about the data type, it's trait implementations and method set.

Some of these issues were solved via extension traits and wrapper types but
this became cumbersome at one point and it became clear that using a custom
value type simplifies maintainability and extensibility. Also
`serde_json::Value` is very powerful but most of its advanced features are not
needed for `dts`.

## Notable differences to the `serde_json`

- `Value` and `Number` types are `Ord` and `Hash` to ease implementation of
  more sophisticated transformations in `dts`. For example, an efficient
  implementation to remove duplicates from a `Vec<Value>` requires `Value` to
  be `Hash` which is not the case for `serde_json::Value`.
- Public methods and specialized (de-)serializers not needed by `dts` were
  removed.
- No support for arbitrary precision numbers or raw value.
- The `Value` type received additional methods which may or may not be
  useful in the context of `dts` only.
- JSON (de-)serializers are not part of this crate. This really is only about
  the types to represent arbitrary JSON values.

## License

The source code of dts-json is released under the MIT License. See the bundled
LICENSE file for details.
