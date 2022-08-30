use anyhow::Context;
use clap::Parser;
use confargs::{prefix_char_filter, Toml};

#[derive(Clone, Debug, Parser, PartialEq)]
struct Args {
    #[clap(long, default_value = "string")]
    string: String,
    #[clap(long, default_value_t = 42)]
    integer: isize,
    #[clap(long, default_value_t = 42.2)]
    float: f32,
    #[clap(long)]
    array: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let args = confargs::args::<Toml>(prefix_char_filter::<'@'>)
        .context("failed to parse config")
        .map(Args::try_parse_from)?
        .context("failed to parse arguments")?;
    println!("{:?}", args);
    Ok(())
}
