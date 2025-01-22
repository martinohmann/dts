//! Command line arguments for dts.

#[cfg(feature = "color")]
use crate::output::ColorChoice;
use crate::paging::PagingChoice;
use anyhow::{anyhow, Result};
use clap::{Args, Parser, ValueHint};
use clap_complete::Shell;
use dts::{de::DeserializeOptions, ser::SerializeOptions, Encoding, Sink, Source};
use regex::Regex;
use unescape::unescape;

/// Simple tool to transcode between different encodings.
///
/// The tool first deserializes data from the input into an internal representation which resembles
/// JSON. As an optional step certain transformations can be applied before serializing back into
/// the output encoding.
///
/// Refer to the documentation of the input, transform and output options below.
#[derive(Parser, Debug)]
#[command(
    name = "dts",
    version,
    after_help = "Hint: `dts -h` only provides a usage summary. Run `dts --help` for the full details to each flag."
)]
pub struct Options {
    /// Input sources.
    ///
    /// If multiple files are provided, the decoded data is read into an array. Input files many
    /// also be remote URLs. Data may also be provided on stdin. If stdin is used in combination
    /// with one or more input files, the data from stdin will be read into the first element of
    /// the resulting array.
    #[arg(name = "SOURCE", value_hint = ValueHint::AnyPath)]
    pub sources: Vec<Source>,

    /// Output sink. Can be specified multiple times. Defaults to stdout if omitted.
    ///
    /// It is possible to provide multiple output files if the data resembles an array. Each output
    /// file will receive an array element. The last output file collects the remaining elements if
    /// there are more elements than files.
    ///
    /// Passing '-' as filename or providing no output files will write the data to stdout instead.
    #[arg(short = 'O', long = "sink", value_name = "SINK", value_hint = ValueHint::FilePath)]
    pub sinks: Vec<Sink>,

    /// Options for deserializing the input.
    #[clap(flatten)]
    pub input: InputOptions,

    /// Options for data transformations performed after deserializing from the input encoding but
    /// before serializing back into the output encoding.
    #[clap(flatten)]
    pub transform: TransformOptions,

    /// Options for serializing the output.
    #[clap(flatten)]
    pub output: OutputOptions,

    /// If provided, outputs the completion file for the given shell.
    #[arg(value_enum, long, value_name = "SHELL", group = "generate-completion")]
    pub generate_completion: Option<Shell>,

    /// List available color themes and exit.
    #[cfg(feature = "color")]
    #[arg(long, conflicts_with = "generate-completion")]
    pub list_themes: bool,
}

/// Options that configure the behaviour of input deserialization.
#[derive(Args, Debug)]
pub struct InputOptions {
    /// Set the input encoding.
    ///
    /// If absent, dts will attempt to detect the encoding from the input file extension (if
    /// present) or from the first line of input.
    #[arg(value_enum, short = 'i', long, help_heading = "Input Options")]
    pub input_encoding: Option<Encoding>,

    /// Indicate that CSV input does not include a header row.
    ///
    /// If this flag is absent, the first line of CSV input is treated as headers and will be
    /// discarded.
    #[arg(long, help_heading = "Input Options")]
    pub csv_without_headers: bool,

    /// Use CSV headers as keys for the row columns.
    ///
    /// When reading CSV, this flag will deserialize the input into an array of maps with each
    /// field keyed by the corresponding header value. Otherwise, the input is deserialized into an
    /// array of arrays.
    #[arg(short = 'H', long, help_heading = "Input Options")]
    pub csv_headers_as_keys: bool,

    /// Custom delimiter for CSV input.
    #[arg(short = 'd', long, value_parser = parse_csv_delimiter, help_heading = "Input Options")]
    pub csv_input_delimiter: Option<u8>,

    /// Regex pattern to split text input at.
    #[arg(short = 's', long, help_heading = "Input Options")]
    pub text_split_pattern: Option<Regex>,

    /// Glob pattern for directories.
    ///
    /// Required if any of the input paths is a directory. Ignored otherwise.
    #[arg(long, help_heading = "Input Options")]
    pub glob: Option<String>,

    /// Read input into a map keyed by file path of the origin file.
    ///
    /// If multiple input files or at least one directory is provided, this reads the result into
    /// a map keyed by file path instead of an array. If only one input file is provided, this
    /// option is ignored.
    #[arg(short = 'P', long, help_heading = "Input Options")]
    pub file_paths: bool,

    /// Continue on errors that occur while reading or deserializing input data.
    ///
    /// If the flag is provided, `dts` will continue to read and deserialize the remaining input
    /// sources. For example, this is useful if you want to deserialize files using a glob pattern
    /// and one of the files is malformed. In this case a warning is logged to stderr and the
    /// source is skipped. This flag is ignored if input is read only from a single source that is
    /// not a directory.
    #[arg(short = 'C', long, help_heading = "Input Options")]
    pub continue_on_error: bool,

    /// Simplify input if the encoding supports it.
    ///
    /// Some encodings like HCL support partial expression evaluation, where an expression like
    /// `1 + 2` can be evaluated to `3`. This flag controls if input simplifications like this
    /// should be performed or not.
    #[arg(long, help_heading = "Input Options")]
    pub simplify: bool,
}

impl From<&InputOptions> for DeserializeOptions {
    fn from(opts: &InputOptions) -> Self {
        Self {
            csv_headers_as_keys: opts.csv_headers_as_keys,
            csv_without_headers: opts.csv_without_headers,
            csv_delimiter: opts.csv_input_delimiter,
            text_split_pattern: opts.text_split_pattern.clone(),
            simplify: opts.simplify,
        }
    }
}

/// Options that configure the behaviour of data transformation.
#[cfg(feature = "jaq")]
#[derive(Args, Debug)]
pub struct TransformOptions {
    /// A jq expression for transforming the input data.
    ///
    /// If the expression starts with an `@` it is treated as a local file path and the expression
    /// is read from there instead.
    ///
    /// See <https://stedolan.github.io/jq/manual/> for supported operators, filters and
    /// functions.
    #[arg(
        short = 'j',
        long = "jq",
        value_name = "EXPRESSION",
        help_heading = "Transform Options"
    )]
    pub jq_expression: Option<String>,
}

/// Options that configure the behaviour of data transformation.
#[cfg(not(feature = "jaq"))]
#[derive(Args, Debug)]
pub struct TransformOptions {
    /// A jq expression for transforming the input data.
    ///
    /// The usage of this flag requires the `jq` executable to be present in the `PATH`. You may
    /// also point `dts` to a different `jq` executable by setting the `DTS_JQ` environment
    /// variable.
    ///
    /// If the expression starts with an `@` it is treated as a local file path and the expression
    /// is read from there instead.
    ///
    /// See <https://stedolan.github.io/jq/manual/> for supported operators, filters and
    /// functions.
    #[arg(
        short = 'j',
        long = "jq",
        value_name = "EXPRESSION",
        help_heading = "Transform Options"
    )]
    pub jq_expression: Option<String>,
}

/// Options that configure the behaviour of output serialization.
#[derive(Args, Debug)]
pub struct OutputOptions {
    /// Set the output encoding.
    ///
    /// If absent, the encoding will be detected from the output file extension.
    ///
    /// If the encoding is not explicitly set and it cannot be inferred from the output file
    /// extension (or the output is stdout), the fallback is to encode output as JSON.
    #[arg(value_enum, short = 'o', long, help_heading = "Output Options")]
    pub output_encoding: Option<Encoding>,

    /// Controls when to use colors.
    ///
    /// The default setting is `auto`, which means dts will try to guess when to use colors. For
    /// example, if dts is printing to a terminal, it will use colors. If it is redirected to a
    /// file or a pipe, it will suppress color output. Output is also not colored if the TERM
    /// environment variable isn't set or the terminal is `dumb`.
    ///
    /// Use color `always` to enforce coloring.
    #[cfg(feature = "color")]
    #[arg(
        value_enum,
        long,
        value_name = "WHEN",
        default_value = "auto",
        env = "DTS_COLOR",
        help_heading = "Output Options"
    )]
    pub color: ColorChoice,

    /// Controls the color theme to use.
    ///
    /// See --list-themes for available color themes.
    #[cfg(feature = "color")]
    #[arg(long, env = "DTS_THEME", help_heading = "Output Options")]
    pub theme: Option<String>,

    /// Controls when to page output.
    ///
    /// The default setting is `auto`. dts will try to guess when to page output when `auto` is
    /// enabled. For example, if the output does fit onto the screen it may not be paged depending
    /// on the pager in use.
    ///
    /// Use `always` to enforce paging even if the output fits onto the screen.
    #[arg(
        value_enum,
        long,
        value_name = "WHEN",
        default_value = "auto",
        env = "DTS_PAGING",
        help_heading = "Output Options"
    )]
    pub paging: PagingChoice,

    /// Controls the output pager to use.
    ///
    /// By default the pager configured via the `PAGER` environment variable will be used. The
    /// fallback is `less`.
    #[arg(long, env = "DTS_PAGER", help_heading = "Output Options")]
    pub pager: Option<String>,

    /// Emit output data in a compact format.
    ///
    /// This will disable pretty printing for encodings that support it.
    #[arg(short = 'c', long, help_heading = "Output Options")]
    pub compact: bool,

    /// Add a trailing newline to the output.
    #[arg(short = 'n', long, help_heading = "Output Options")]
    pub newline: bool,

    /// Use object keys of the first item as CSV headers.
    ///
    /// When the input is an array of objects and the output encoding is CSV, the field names of
    /// the first object will be used as CSV headers. Field values of all following objects will be
    /// matched to the right CSV column based on their key. Missing fields produce empty columns
    /// while excess fields are ignored.
    #[arg(short = 'K', long, help_heading = "Output Options")]
    pub keys_as_csv_headers: bool,

    /// Custom delimiter for CSV output.
    #[arg(short = 'D', long, value_parser = parse_csv_delimiter, help_heading = "Output Options")]
    pub csv_output_delimiter: Option<u8>,

    /// Custom separator to join text output with.
    #[arg(short = 'J', long, value_parser = parse_unescaped, help_heading = "Output Options")]
    pub text_join_separator: Option<String>,

    /// Treat output arrays as multiple YAML documents.
    ///
    /// If the output is an array and the output format is YAML, treat the array members as
    /// multiple YAML documents that get written to the same file.
    #[arg(long, help_heading = "Output Options")]
    pub multi_doc_yaml: bool,

    /// Overwrite output files if they exist.
    #[arg(long)]
    pub overwrite: bool,
}

impl From<&OutputOptions> for SerializeOptions {
    fn from(opts: &OutputOptions) -> Self {
        Self {
            compact: opts.compact,
            newline: opts.newline,
            keys_as_csv_headers: opts.keys_as_csv_headers,
            csv_delimiter: opts.csv_output_delimiter,
            text_join_separator: opts.text_join_separator.clone(),
            multi_doc_yaml: opts.multi_doc_yaml,
        }
    }
}

fn parse_csv_delimiter(s: &str) -> Result<u8> {
    let unescaped = parse_unescaped(s)?;
    let bytes = unescaped.as_bytes();

    if bytes.len() == 1 {
        Ok(bytes[0])
    } else {
        Err(anyhow!("expected single byte delimiter"))
    }
}

fn parse_unescaped(s: &str) -> Result<String> {
    unescape(s).ok_or_else(|| anyhow!("string contains invalid escape sequences: `{}`", s))
}
