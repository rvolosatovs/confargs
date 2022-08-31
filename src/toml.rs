// SPDX-License-Identifier: Apache-2.0

use super::{parse_bool_arg, parse_string_arg, Format};

use std::collections::VecDeque;
use std::fmt::Display;
use std::{io, vec};

use anyhow::{bail, Result};
use toml::Value;

fn parse_primitive_arg(k: impl Display, v: Value) -> Result<Option<String>> {
    match v {
        Value::String(v) => Ok(parse_string_arg(k, v).into()),
        Value::Integer(v) => Ok(parse_string_arg(k, v).into()),
        Value::Float(v) => Ok(parse_string_arg(k, v).into()),
        Value::Boolean(v) => Ok(parse_bool_arg(k, v)),
        Value::Datetime(v) => Ok(parse_string_arg(k, v).into()),
        Value::Array(_) => bail!("array not supported for field `{k}`"),
        Value::Table(_) => bail!("table not supported for field `{k}`"),
    }
}

struct ArrayIterator<K> {
    key: K,
    values: VecDeque<Value>,
}

impl<K> ArrayIterator<K> {
    fn new(key: K, values: Vec<Value>) -> Self {
        Self {
            key,
            values: values.into(),
        }
    }
}

impl<K: Display + Copy> Iterator for ArrayIterator<K> {
    type Item = Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(arg) = self
                .values
                .pop_front()
                .map(|v| parse_primitive_arg(self.key, v))?
                .transpose()
            {
                return Some(arg);
            }
        }
    }
}

fn parse_arg<'a>(
    k: impl Display + Copy + 'a,
    v: Value,
) -> Result<Box<dyn Iterator<Item = Result<String>> + 'a>> {
    match v {
        Value::String(_)
        | Value::Integer(_)
        | Value::Float(_)
        | Value::Boolean(_)
        | Value::Datetime(_) => {
            let arg = parse_primitive_arg(k, v)?;
            Ok(Box::new(arg.map(Ok).into_iter()))
        }
        Value::Array(vs) => Ok(Box::new(ArrayIterator::new(k, vs))),
        Value::Table(_) => bail!("table not supported for field `{k}`"),
    }
}

/// [TOML](https://toml.io/) configuration file format.
///
/// This format expects the configuration to be represented as a table. Nested tables are not
/// supported.
///
/// # Examples
///
/// ```
/// # use std::io::Write;
/// # use tempfile::NamedTempFile;
/// use confargs::{Format, Toml};
///
/// assert_eq!(
///     Toml::from_slice(
///         r#"#Test config
/// string = "foo"
/// integer = 42
/// float = 42.2
/// true = true
/// false = false
/// datetime = 01:02:03.000000004
/// array = [1, 2, 3]"#
///                    .as_bytes()
///     )
///     .unwrap(),
///     vec![
///         "--array=1",
///         "--array=2",
///         "--array=3",
///         "--datetime=01:02:03.000000004",
///         "--float=42.2",
///         "--integer=42",
///         "--string=foo",
///         "--true",
///     ]
/// );
/// ```
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Config;

impl Config {
    fn from_iter(iter: impl IntoIterator<Item = (String, Value)>) -> Result<Vec<String>> {
        iter.into_iter().try_fold(vec![], |mut args, (k, v)| {
            for arg in parse_arg(&k, v)? {
                args.push(arg?);
            }
            Ok(args)
        })
    }
}

impl Format for Config {
    type IntoIter = Vec<String>;

    fn from_slice(buf: impl AsRef<[u8]>) -> io::Result<Self::IntoIter> {
        match toml::from_slice(buf.as_ref()).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("failed to parse TOML: {e}"),
            )
        })? {
            Value::Table(kv) => Self::from_iter(kv).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("failed to parse TOML table: {e}"),
                )
            }),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid config file format",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use toml::value::{Datetime, Time};

    #[test]
    fn parse_arg() {
        assert_eq!(
            super::parse_arg("key", Value::String("foo".into()))
                .unwrap()
                .collect::<Result<Vec<_>>>()
                .unwrap(),
            vec!["--key=foo"]
        );
        assert_eq!(
            super::parse_arg("key", Value::Integer(42))
                .unwrap()
                .collect::<Result<Vec<_>>>()
                .unwrap(),
            vec!["--key=42"]
        );
        assert_eq!(
            super::parse_arg("key", Value::Float(42.))
                .unwrap()
                .collect::<Result<Vec<_>>>()
                .unwrap(),
            vec!["--key=42"]
        );
        assert_eq!(
            super::parse_arg("key", Value::Float(42.2))
                .unwrap()
                .collect::<Result<Vec<_>>>()
                .unwrap(),
            vec!["--key=42.2"]
        );
        assert_eq!(
            super::parse_arg("key", Value::Boolean(true))
                .unwrap()
                .collect::<Result<Vec<_>>>()
                .unwrap(),
            vec!["--key"]
        );
        assert!(super::parse_arg("key", Value::Boolean(false))
            .unwrap()
            .collect::<Result<Vec<_>>>()
            .unwrap()
            .is_empty());
        assert_eq!(
            super::parse_arg(
                "key",
                Value::Datetime(Datetime {
                    date: None,
                    time: Some(Time {
                        hour: 1,
                        minute: 2,
                        second: 3,
                        nanosecond: 4,
                    }),
                    offset: None,
                })
            )
            .unwrap()
            .collect::<Result<Vec<_>>>()
            .unwrap(),
            vec!["--key=01:02:03.000000004"]
        );
        assert_eq!(
            super::parse_arg(
                "key",
                Value::Array(vec![
                    Value::Boolean(true),
                    Value::Boolean(false),
                    Value::Integer(42),
                    Value::String("test".into())
                ])
            )
            .unwrap()
            .collect::<Result<Vec<_>>>()
            .unwrap(),
            vec!["--key", "--key=42", "--key=test"]
        );
    }
}
