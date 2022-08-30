use std::io;

use confargs::{args, prefix_char_filter, Toml};

fn main() -> io::Result<()> {
    args::<Toml>(prefix_char_filter::<'@'>)
        .map(|args| args.into_iter().for_each(|arg| println!("{arg}")))
}
