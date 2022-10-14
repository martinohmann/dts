# dts

[![Build Status](https://github.com/martinohmann/dts/workflows/ci/badge.svg)](https://github.com/martinohmann/dts/actions?query=workflow%3Aci)
![MIT License](https://img.shields.io/github/license/martinohmann/dts?color=blue)

A simple tool to _**deserialize**_ data from an input encoding, _**transform**_
it and _**serialize**_ it back into an output encoding.

Uses [`jq`](https://stedolan.github.io/jq/) for data transformation and
requires rust >= 1.56.0.

## Installation

Check out the [releases page](https://github.com/martinohmann/dts/releases)
for prebuilt versions of `dts`.

Statically-linked binaries are also available: look for archives with `musl` in
the filename.

### From crates.io

```sh
cargo install dts
```

### From source

Clone the repository and run:

```sh
cargo install --locked --path .
```

## Usage

```sh
dts [<source>...] [-j <jq-expression>] [-O <sink>...]
```

For a full list of available flags consult the help:

```sh
dts --help
```

## Examples

Convert YAML to TOML:

```sh
dts input.yaml -o toml
```

Load all YAML files from sub directories and merge them into one:

```sh
dts . --glob '**/*.yaml' output.yaml
```

Transform the input data using a [`jq`](https://stedolan.github.io/jq/) expression:

```sh
dts tests/fixtures/example.json -j '.users | map(select(.age < 30))'
```

Use `jq` filter expression from a file:

```sh
dts tests/fixtures/example.json -j @my-filter.jq
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
to have colors at all. See the [feature flags](#feature-flags) section below.

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
- HCL _(deserialize, serialize only supports HCL attributes)_

## Feature flags

To build `dts` without its default features enabled, run:

```sh
cargo build --no-default-features --release
```

The following feature flags are available:

* `color`: Enables support for colored output. This feature is enabled by
  default.

  If you just want to disable colors by default with the option to enable them
  conditionally, you can also set the [`NO_COLOR`](https://no-color.org/)
  environment variable or set `DTS_COLOR=never`.

* `jaq`: Use [`jaq-core`](https://docs.rs/jaq-core/latest/jaq_core/) to
  process transformation filters instead of shelling out to `jq`. This feature
  is enabled by default.

## License

The source code of dts is released under the MIT License. See the bundled
LICENSE file for details.
