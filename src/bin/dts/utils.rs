//! Misc utilities.

use std::path::PathBuf;

/// Resolves a shell command to a binary and its args.
///
/// The purpose of doing this instead of passing the path to the program directly to Command::new
/// is that Command::new will hand relative paths to CreateProcess on Windows, which will
/// implicitly search the current working directory for the executable. This could be undesirable
/// for security reasons.
///
/// Returns `None` if `s` is not a valid shell command (e.g. mismatching quotes). On windows it
/// also returns `None` if the binary cannot be found in `PATH`.
pub fn resolve_cmd<S: AsRef<str>>(s: S) -> Option<(PathBuf, Vec<String>)> {
    shell_words::split(s.as_ref()).ok().and_then(|parts| {
        parts.split_first().and_then(|(cmd, args)| {
            grep_cli::resolve_binary(cmd)
                .ok()
                .map(|bin_path| (bin_path, args.to_vec()))
        })
    })
}
