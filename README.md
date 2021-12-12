# dts

[![Build Status](https://github.com/martinohmann/dts/workflows/ci/badge.svg)](https://github.com/martinohmann/dts/actions?query=workflow%3Aci)
![GitHub](https://img.shields.io/github/license/martinohmann/dts?color=orange)

A simple tool to _**deserialize**_ data from an input encoding, _**transform**_
it and _**serialize**_ it back into an output encoding.

Requires rust >= 1.56.0.

## Installation from source

This is the only install option available right now.

Clone the repository and run:

```sh
cargo install --path .
```

## Usage

```sh
dts [<source>...] [-t <transform-options>] [-O <sink>...]
```

For a full list of available flags and transform options consult the help:

```sh
dts --help
```

## Examples

Convert YAML to TOML and remove empty values:

```sh
dts input.yaml -t remove-empty-values -o toml
```

Load all YAML files from sub directories, flatten nested arrays and merge them into one:

```sh
dts . --glob '**/*.yaml' -t flatten-arrays output.yaml
```

Run a JSONPath filter on the input data:

```sh
dts tests/fixtures/example.json -t jsonpath='$.users[?(@.age < 30)]'
```

Combine multiple transformation options (multiple usages of the same option possible):

```sh
# This uses the single char forms of the transformation options
dts tests/fixtures/example.json -t d='some_key',m,j='[*]',f,j='[*].id'
```

Read data from stdin:

```sh
echo '{"foo": {"bar": "baz"}}' | dts -i json -tF,r -o yaml
```

## Supported Encodings

Right now `dts` supports the following encodings:

- JSON
- YAML
- TOML
- JSON5 _(deserialize only)_
- CSV
- QueryString
- XML
- Text
- Gron
- HCL _(deserialize only)_

## License

The source code of dts is released under the MIT License. See the bundled
LICENSE file for details.
