use clap::Parser;
use eigen_cli::{args::Args, execute_command};

fn main() {
    let args = Args::parse();
    execute_command(args.command).unwrap();
}
