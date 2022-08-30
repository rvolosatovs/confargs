# Description

`confargs` is a Rust library, which parses a configuration file in arbitrary format into a iterator of command-line arguments. The main use case for this is to add configuration file support for CLI tools and argument parsers, which do not have support for configuration files.

# Compatibility

This project primarily aims at compatibility with [clap](https://github.com/clap-rs/clap), which is tested automatically in CI. Other libraries *should* work as well, but that is not tested.

# Examples

Examples are provided in `examples` directory along an example configuration files in supported formats.

## clap

From the root of the repository:

```shell
$ cargo run -q --example clap    
Args { string: "string", integer: 42, float: 42.2, array: [] }
$ cargo run -q --example clap -- @examples/conf.toml
Args { string: "conf", integer: -42, float: -42.2, array: ["test", "config"] }
```
