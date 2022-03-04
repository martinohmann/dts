# dts-core

[![Build Status](https://github.com/martinohmann/dts/workflows/ci/badge.svg)](https://github.com/martinohmann/dts/actions?query=workflow%3Aci)
[![docs.rs](https://img.shields.io/docsrs/dts-core)](https://docs.rs/dts-core)
![MIT License](https://img.shields.io/github/license/martinohmann/dts?color=blue)

This crate provides the `Serializer` and `Deserializer` implementations used
by [`dts`](https://github.com/martinohmann) internally to convert between
different input and output encodings. In additional to that it exposes the
data transformation functionality that `dts` uses to modify the input data
before serializing it.

**Note**: The API of `dts-core` is subject to change and there are no
stability guarantees.

## License

The source code of dts-core is released under the MIT License. See the bundled
LICENSE file for details.
