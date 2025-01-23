//! Utilities to syntax highlight output.

use crate::{
    output::ColorChoice,
    paging::{PagingChoice, PagingConfig},
    utils::resolve_cmd,
};
use bat::{assets::HighlightingAssets, config::Config, controller::Controller, Input, PagingMode};
use dts::Encoding;
use std::io::{self, Write};
use std::path::Path;
use termcolor::{ColorSpec, StandardStream, WriteColor};

/// ColoredStdoutWriter writes data to stdout and may or may not colorize it.
pub struct ColoredStdoutWriter<'a> {
    highlighter: SyntaxHighlighter<'a>,
    encoding: Encoding,
    theme: Option<&'a str>,
    buf: Vec<u8>,
}

impl<'a> ColoredStdoutWriter<'a> {
    /// Creates a new `ColoredStdoutWriter` which colorizes output based on the provided `Encoding`
    /// and `theme`, and prints it using the provided `SyntaxHighlighter`.
    pub fn new(
        highlighter: SyntaxHighlighter<'a>,
        encoding: Encoding,
        theme: Option<&'a str>,
    ) -> Self {
        ColoredStdoutWriter {
            highlighter,
            encoding,
            theme,
            buf: Vec::with_capacity(256),
        }
    }
}

impl io::Write for ColoredStdoutWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buf.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        let buf = std::mem::take(&mut self.buf);

        if buf.is_empty() {
            return Ok(());
        }

        self.highlighter.print(self.encoding, &buf, self.theme)
    }
}

impl Drop for ColoredStdoutWriter<'_> {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

impl From<PagingChoice> for PagingMode {
    fn from(choice: PagingChoice) -> Self {
        match choice {
            PagingChoice::Always => PagingMode::Always,
            PagingChoice::Auto => PagingMode::QuitIfOneScreen,
            PagingChoice::Never => PagingMode::Never,
        }
    }
}

/// Can syntax highlight a buffer and then print the result to stdout.
pub struct SyntaxHighlighter<'a> {
    assets: HighlightingAssets,
    paging_config: PagingConfig<'a>,
}

impl<'a> SyntaxHighlighter<'a> {
    /// Creates a new `SyntaxHighlighter`.
    pub fn new(paging_config: PagingConfig<'a>) -> Self {
        SyntaxHighlighter {
            assets: HighlightingAssets::from_binary(),
            paging_config,
        }
    }

    /// Returns an iterator over all supported color themes.
    fn themes(&self) -> impl Iterator<Item = &str> {
        self.assets.themes()
    }

    /// Returns the color theme that should be used to color the output. Checks if the requested
    /// theme is available, otherwise fall back to `base16` as default.
    fn pick_theme(&self, theme: Option<&str>) -> String {
        theme
            .and_then(|requested| {
                let requested = requested.to_lowercase();
                self.themes()
                    .find(|known| known.to_lowercase() == requested)
                    .map(|theme| theme.to_owned())
            })
            .unwrap_or_else(|| String::from("base16"))
    }

    /// Returns a suitable output pager.
    fn pager(&self) -> String {
        // Since we are using `bat` to do the syntax highlighting for us we have to ensure that the
        // pager is not `bat` itself. In this case we'll just fall back to using the default pager.
        let pager = self.paging_config.pager();

        if let Some((pager_bin, _)) = resolve_cmd(&pager) {
            if !pager_bin.ends_with("bat") && !pager_bin.ends_with("bat.exe") {
                return pager;
            }
        }

        self.paging_config.default_pager()
    }

    /// Highlights `buf` using the given `Encoding` hint and `theme` and prints the result to
    /// stdout.
    pub fn print(&self, encoding: Encoding, buf: &[u8], theme: Option<&str>) -> io::Result<()> {
        let pager = self.pager();
        let paging_choice = self.paging_config.paging_choice();
        let theme = self.pick_theme(theme);

        let config = Config {
            colored_output: true,
            true_color: true,
            pager: Some(&pager),
            paging_mode: paging_choice.into(),
            theme,
            ..Default::default()
        };

        let pseudo_filename = Path::new("out").with_extension(encoding.as_str());
        let inputs = vec![Input::from_bytes(buf).name(pseudo_filename).into()];

        let ctrl = Controller::new(&config, &self.assets);
        ctrl.run(inputs, None).map_err(io::Error::other)?;
        Ok(())
    }
}

/// Prints available themes to stdout.
pub fn print_themes(color_choice: ColorChoice) -> io::Result<()> {
    let example = include_bytes!("assets/example.json");
    let highlighter = SyntaxHighlighter::new(PagingConfig::default());

    if color_choice.should_colorize() {
        let max_len = highlighter.themes().map(str::len).max().unwrap_or(0);

        let mut stdout = StandardStream::stdout(color_choice.into());

        for theme in highlighter.themes() {
            stdout.set_color(ColorSpec::new().set_bold(true))?;
            write!(&mut stdout, "{:1$}", theme, max_len + 2)?;
            stdout.reset()?;

            highlighter.print(Encoding::Json, example, Some(theme))?;
        }
    } else {
        for theme in highlighter.themes() {
            println!("{}", theme);
        }
    }

    Ok(())
}
