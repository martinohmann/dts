//! Command line arguments for dts.

#[cfg(feature = "color")]
use crate::output::ColorChoice;
use crate::paging::PagingChoice;
use anyhow::{anyhow, Result};
use clap::{ArgSettings, Args, Parser, ValueHint};
use clap_generate::Shell;
use dts_core::{de::DeserializeOptions, ser::SerializeOptions, Encoding, Sink, Source};
use regex::Regex;
use unescape::unescape;

/// Simple tool to transcode between different encodings.
///
/// The tool first deserializes data from the input data into an internal representation which
/// resembles JSON. As an optional step certain transformations can be applied to the data before
/// serializing into the output encoding.
///
/// Refer to the input, transform and output options below.
#[derive(Parser, Debug)]
#[clap(
    name = "dts",
    version,
    after_help = "Hint: `dts -h` only provides a usage summary. Run `dts --help` for the full details to each flag.\n\nTo get help about the transformation expression syntax, run `dts --help-transform`."
)]
pub struct Options {
    /// Input sources.
    ///
    /// If multiple files are provides, the decoded data is read into an array. The input files
    /// many also be remote URLs. Data may also be provided on stdin. If stdin is used in
    /// combination with one or more input files, the data from stdin will be read into the first
    /// element of the resulting array.
    #[clap(name = "SOURCE", value_hint = ValueHint::AnyPath)]
    pub sources: Vec<Source>,

    /// Output sink. Can be specified multiple times. Defaults to stdout if omitted.
    ///
    /// It is possible to provide multiple output files if the data resembles an array. Each output
    /// file will receive an array element. The last output file collects the remaining elements if
    /// there are more elements than files.
    ///
    /// Passing '-' as filename or providing no output files will write the data to stdout instead.
    #[clap(short = 'O', long = "sink", value_name = "SINK", value_hint = ValueHint::FilePath)]
    #[clap(multiple_occurrences = true)]
    pub sinks: Vec<Sink>,

    /// Options for deserializing the input.
    #[clap(flatten)]
    pub input: InputOptions,

    /// Options for data transformations performed after deserializing from the input encoding but
    /// before serializing to the output encoding.
    #[clap(flatten)]
    pub transform: TransformOptions,

    /// Options for serializing the output.
    #[clap(flatten)]
    pub output: OutputOptions,

    /// If provided, outputs the completion file for the given shell.
    #[clap(arg_enum, long, value_name = "SHELL")]
    pub generate_completion: Option<Shell>,
}

/// Options that configure the behaviour of input deserialization.
#[derive(Args, Debug)]
#[clap(help_heading = "INPUT OPTIONS")]
pub struct InputOptions {
    /// Set the input encoding. If absent encoding will be detected from input file extension.
    #[clap(arg_enum, short = 'i', long, setting = ArgSettings::HidePossibleValues)]
    pub input_encoding: Option<Encoding>,

    /// Indicate that CSV input does not include a header row.
    ///
    /// If this flag is absent, the first line of CSV input is treated as headers and will be
    /// discarded.
    #[clap(long)]
    pub csv_without_headers: bool,

    /// Use CSV headers as keys for the row columns.
    ///
    /// When reading CSV, this flag will deserialize the input into an array of maps with each
    /// field keyed by the corresponding header value. Otherwise, the input is deserialized into an
    /// array of arrays.
    #[clap(short = 'H', long)]
    pub csv_headers_as_keys: bool,

    /// Custom delimiter for CSV input.
    #[clap(short = 'd', long, parse(try_from_str = parse_csv_delimiter))]
    pub csv_input_delimiter: Option<u8>,

    /// Regex pattern to split text input at.
    #[clap(short = 's', long)]
    pub text_split_pattern: Option<Regex>,

    /// Glob pattern for directories.
    ///
    /// Required if any of the input paths is a directory. Ignored otherwise.
    #[clap(long)]
    pub glob: Option<String>,

    /// Read input into a map keyed by file path of the origin file.
    ///
    /// If multiple input files or at least one directory is provided, this reads the result into
    /// a map keyed by file path instead of an array. If only one input file is provided, this
    /// option is ignored.
    #[clap(short = 'P', long)]
    pub file_paths: bool,

    /// Continue on errors that occur while reading or deserializing input data.
    ///
    /// If the flag is provided, `dts` will continue to read and deserialize the remaining input
    /// sources. For example, this is useful if you want to deserialize files using a glob pattern
    /// and one of the files is malformed. In this case a warning is logged to stderr and the
    /// source is skipped. This flag is ignored if input is read only from a single source that is
    /// not a directory.
    #[clap(short = 'C', long)]
    pub continue_on_error: bool,
}

impl From<&InputOptions> for DeserializeOptions {
    fn from(opts: &InputOptions) -> Self {
        Self {
            csv_headers_as_keys: opts.csv_headers_as_keys,
            csv_without_headers: opts.csv_without_headers,
            csv_delimiter: opts.csv_input_delimiter,
            text_split_pattern: opts.text_split_pattern.clone(),
        }
    }
}

/// Options that configure the behaviour of data transformation.
#[derive(Args, Debug)]
#[clap(help_heading = "TRANSFORM OPTIONS")]
pub struct TransformOptions {
    /// An expression containing one or more transformation functions.
    ///
    /// See --help-transform to get a list of possible transformation functions, their arguments.
    #[clap(short = 't', long = "transform", value_name = "EXPRESSION")]
    #[clap(multiple_occurrences = true, number_of_values = 1)]
    pub expressions: Vec<String>,

    /// Displays help and usage examples for the transformation expressions and available
    /// functions and exit.
    #[clap(long = "help-transform", conflicts_with = "generate-completion")]
    pub print_help: bool,
}

/// Options that configure the behaviour of output serialization.
#[derive(Args, Debug)]
#[clap(help_heading = "OUTPUT OPTIONS")]
pub struct OutputOptions {
    /// Set the output encoding. If absent encoding will be detected from output file extension.
    ///
    /// If the encoding is not explicitly set and it cannot be inferred from the output file
    /// extension (or the output is stdout), the fallback is to encode output as JSON.
    #[clap(arg_enum, short = 'o', long, setting = ArgSettings::HidePossibleValues)]
    pub output_encoding: Option<Encoding>,

    /// Controls when to use colors.
    ///
    /// The default setting is `auto`, which means dts will try to guess when to use colors. For
    /// example, if dts is printing to a terminal, then it will use colors, but if it is redirected
    /// to a file or a pipe, then it will suppress color output. Output is also not colored if the
    /// TERM environment variable isn't set or the terminal is `dumb`.
    ///
    /// Use color `always` to enforce coloring.
    #[cfg(feature = "color")]
    #[clap(arg_enum, long, value_name = "WHEN")]
    #[clap(default_value = "auto", env = "DTS_COLOR")]
    pub color: ColorChoice,

    /// Controls the color theme to use.
    ///
    /// See --list-themes for available color themes.
    #[cfg(feature = "color")]
    #[clap(long, env = "DTS_THEME")]
    pub theme: Option<String>,

    /// List available color themes and exit.
    #[cfg(feature = "color")]
    #[clap(long, conflicts_with = "generate-completion")]
    pub list_themes: bool,

    /// Controls when to page output.
    ///
    /// The default setting is `auto`, which means dts will try to guess when to page output. For
    /// example, if the output does fit onto the screen it may not be paged depending on the pager
    /// in use.
    ///
    /// Use paging `always` to enforce paging even if the output fits onto the screen.
    #[clap(arg_enum, long, value_name = "WHEN")]
    #[clap(default_value = "auto", env = "DTS_PAGING")]
    pub paging: PagingChoice,

    /// Controls the output pager to use.
    ///
    /// By default the pager configured via the `PAGER` environment variable will be used. The
    /// fallback is `less`.
    #[clap(long, env = "DTS_PAGER")]
    pub pager: Option<String>,

    /// Emit output data in a compact format.
    ///
    /// This will disable pretty printing for encodings that support it.
    #[clap(short = 'c', long)]
    pub compact: bool,

    /// Add a trailing newline to the output.
    #[clap(short = 'n', long)]
    pub newline: bool,

    /// Use object keys of the first item as CSV headers.
    ///
    /// When the input is an array of objects and the output encoding is CSV, the field names of
    /// the first object will be used as CSV headers. Field values of all following objects will be
    /// matched to the right CSV column based on their key. Missing fields produce empty columns
    /// while excess fields are ignored.
    #[clap(short = 'K', long)]
    pub keys_as_csv_headers: bool,

    /// Custom delimiter for CSV output.
    #[clap(short = 'D', long, parse(try_from_str = parse_csv_delimiter))]
    pub csv_output_delimiter: Option<u8>,

    /// Custom separator to join text output with.
    #[clap(short = 'J', long, parse(try_from_str = parse_unescaped))]
    pub text_join_separator: Option<String>,

    /// Overwrite output files if they exist.
    #[clap(long)]
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
        }
    }
}

fn parse_csv_delimiter(s: &str) -> Result<u8> {
    let unescaped = parse_unescaped(s)?;
    let bytes = unescaped.as_bytes();

    if bytes.len() == 1 {
        Ok(bytes[0])
    } else {
        Err(anyhow!("Expected single byte delimiter"))
    }
}

fn parse_unescaped(s: &str) -> Result<String> {
    unescape(s).ok_or_else(|| anyhow!("String contains invalid escape sequences: `{}`", s))
}
