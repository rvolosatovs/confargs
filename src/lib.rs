// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
#![deny(
    clippy::all,
    absolute_paths_not_starting_with_crate,
    deprecated_in_future,
    missing_copy_implementations,
    missing_debug_implementations,
    noop_method_call,
    rust_2018_compatibility,
    rust_2018_idioms,
    rust_2021_compatibility,
    single_use_lifetimes,
    trivial_bounds,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_code,
    unreachable_patterns,
    unreachable_pub,
    unstable_features,
    unused,
    unused_import_braces,
    unused_lifetimes,
    unused_results,
    variant_size_differences
)]

mod toml;

pub use self::toml::Config as Toml;

use std::fmt::Display;
use std::fs::read;
use std::path::Path;
use std::{env, io};

fn parse_string_arg(k: impl Display, v: impl Display) -> String {
    format!("--{k}={v}")
}

fn parse_bool_arg(k: impl Display, v: bool) -> Option<String> {
    v.then(|| format!("--{k}"))
}

/// Configuration file format
pub trait Format {
    type IntoIter: IntoIterator<Item = String>;

    /// reads configuration at `path` and returns an [IntoIter](Self::IntoIter) of arguments
    fn read(path: impl AsRef<Path>) -> io::Result<Self::IntoIter> {
        read(path).and_then(|buf| Self::from_slice(buf.as_slice()))
    }

    /// parses configuration in `buf` and returns an [IntoIter](Self::IntoIter) of arguments
    fn from_slice(buf: impl AsRef<[u8]>) -> io::Result<Self::IntoIter>;
}

pub type Filter = fn(&str) -> Option<&Path>;

pub fn prefix_char_filter<const C: char>(arg: &str) -> Option<&Path> {
    arg.strip_prefix(C).map(Path::new)
}

pub fn args<T: Format>(f: Filter) -> io::Result<impl IntoIterator<Item = String>> {
    let mut args = env::args();
    args.try_fold(Vec::with_capacity(args.len()), |mut args, arg| {
        if let Some(path) = f(&arg) {
            T::read(path)
                .map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Failed to parse config at `{}`: {e}", path.display()),
                    )
                })?
                .into_iter()
                .for_each(|arg| args.push(arg));
        } else {
            args.push(arg);
        }
        Ok(args)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Write;
    use std::iter::once;
    use std::process::Command;

    use clap::Parser;
    use tempfile::NamedTempFile;

    const TOML_CONFIG: &str = r#"#Test config
string = "foo"
integer = 42
float = 42.2
true = true
false = false
datetime = 01:02:03.000000004
array = [1, 2, 3]"#;

    #[test]
    fn filters() {
        let _: Filter = prefix_char_filter::<'@'>;
        let _: Filter = |arg| arg.strip_prefix("--config").map(Path::new);
    }

    #[test]
    fn args() {
        let mut conf = NamedTempFile::new().expect("failed to create temporary file");
        assert_eq!(
            conf.write(TOML_CONFIG.as_bytes())
                .expect("failed to write config"),
            TOML_CONFIG.len()
        );

        const PRINT_ARGS: &str = env!("CARGO_BIN_FILE_PRINT_ARGS");
        let out = Command::new(PRINT_ARGS)
            .args([
                "--test",
                &format!("@{}", conf.path().display()),
                "foo",
                "--bar=baz",
            ])
            .output()
            .unwrap();
        assert_eq!(String::from_utf8(out.stderr).unwrap(), "");
        assert_eq!(
            String::from_utf8(out.stdout).unwrap(),
            format!(
                r#"{PRINT_ARGS}
--test
--array=1
--array=2
--array=3
--datetime=01:02:03.000000004
--float=42.2
--integer=42
--string=foo
--true
foo
--bar=baz
"#,
            )
        );
    }

    #[test]
    fn clap() {
        #[derive(Clone, Debug, Parser, PartialEq)]
        struct Args {
            #[clap(long)]
            string: String,
            #[clap(long)]
            integer: isize,
            #[clap(long)]
            float: f32,
            #[clap(long)]
            r#true: bool,
            #[clap(long)]
            r#false: bool,
            #[clap(long)]
            datetime: String, // TODO: Use a "time" type
            #[clap(long)]
            array: Vec<usize>,
        }

        #[inline]
        fn assert_format<T: Format>(buf: impl AsRef<[u8]>) {
            let mut conf = NamedTempFile::new().expect("failed to create temporary file");
            let buf = buf.as_ref();
            assert_eq!(conf.write(buf).expect("failed to write config"), buf.len());
            assert_eq!(
                T::read(conf.path())
                    .map(|args| once("test".into()).chain(args))
                    .map(Args::try_parse_from)
                    .unwrap()
                    .unwrap(),
                Args {
                    string: "foo".into(),
                    integer: 42,
                    float: 42.2,
                    r#true: true,
                    r#false: false,
                    datetime: "01:02:03.000000004".into(), // TODO: Use a "time" type
                    array: vec![1, 2, 3],
                }
            )
        }

        assert_format::<Toml>(TOML_CONFIG)
    }
}
