# dts

[![Build Status](https://github.com/martinohmann/dts/workflows/ci/badge.svg)](https://github.com/martinohmann/dts/actions?query=workflow%3Aci)
![MIT License](https://img.shields.io/github/license/martinohmann/dts?color=blue)

A simple tool to _**deserialize**_ data from an input encoding, _**transform**_
it and _**serialize**_ it back into an output encoding.

Requires rust >= 1.56.0.

## Installation from source

This is the only install option available right now.

Clone the repository and run:

```sh
cargo install --locked --path .
```

## Usage

```sh
dts [<source>...] [-t <transform-options>] [-O <sink>...]
```

For a full list of available flags consult the help:

```sh
dts --help
```

The help for the transformation expression syntax and available functions along
with their documentation can be printed via:

```sh
dts --help-transform
```

## Examples

Convert YAML to TOML and remove empty values:

```sh
dts input.yaml -t remove_empty_values -o toml
```

Load all YAML files from sub directories, flatten nested arrays and merge them into one:

```sh
dts . --glob '**/*.yaml' -t flatten output.yaml
```

Select a subset of the input data via JSONPath query:

```sh
dts tests/fixtures/example.json -t 'select("$.users[?(@.age < 30)]")'
```

Mutate just a subset of the input data:

```sh
dts tests/fixtures/example.json -t 'mutate("$.users[?(@.age > 30)]", sort().flatten())'
```

Combine multiple transformation options:

```sh
dts tests/fixtures/example.json -t "delete_keys('some_key').deep_merge.select('[*]').flatten.select('[*].id')"
```

Read data from stdin:

```sh
echo '{"foo": {"bar": "baz"}}' | dts -i json -o yaml
```

## Output colors and themes

`dts` supports output coloring and syntax highlighting. The coloring behaviour
can be controlled via the `--color` flag or `DTS_COLOR` environment variable.

If the default theme used for syntax highlighting does not suit you, you can
change it via the `--theme` flag or `DTS_THEME` environment variable.

Available themes can be listed via:

```sh
dts --list-themes
```

**Hint**: The `color` feature can be disabled at compile time if you don't want
to have colors at all. See the [Cargo feature](#cargo-features) section below.

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

## Cargo features

Support for colored output is provided by the `color` feature which is enabled
by default. The feature increases binary size and may be disabled via:

```sh
cargo build --no-default-features --release
```

If you just want to disable colors by default with the option to enable them
conditionally, you can also set the [`NO_COLOR`](https://no-color.org/)
environment variable or set `DTS_COLOR=never`.

## License

The source code of dts is released under the MIT License. See the bundled
LICENSE file for details.
