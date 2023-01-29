mod device;
mod cli;

use crate::cli::SlightCommand;

fn main() {
    let args: SlightCommand = argh::from_env();
    println!("{args:?}");
}
