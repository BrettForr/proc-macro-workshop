// This test is similar to the previous where want to ensure that the macro
// correctly generates an error when the input enum is out of order, but this
// time it is using an enum that also has data associated with each variant.

use sorted::sorted;

#[sorted]
pub enum Error {
    Fmt(std::fmt::Error),
    Io(std::io::Error),
    Utf8(std::str::Utf8Error),
    Var(std::env::VarError),
    Dyn(Box<dyn std::error::Error>),
}

fn main() {}
