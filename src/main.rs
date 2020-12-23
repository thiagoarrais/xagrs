use std::io::{self, BufRead, Error};
use std::process::Command;

use itertools::{ Itertools, IntoChunks };
use structopt::clap::ArgSettings;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "xagrs")]
struct Opt {
    #[structopt(short = "L", default_value = "1")]
    limit: usize,

    #[structopt(short = "t", long = "verbose")]
    verbose: bool,

    #[structopt(default_value = "echo")]
    command: String,

    #[structopt(set = ArgSettings::Last, name="args for command", help="Arguments that should be passed to the command on every invocation")]
    fixed_args: Vec<String>,
}

fn chunk_lines<L>(limit: usize, lines: L) -> IntoChunks<L>
  where L: Iterator
{
    lines.chunks(limit)
}

// TODO: make error more friendly
fn main() -> Result<(), Error> {
    let stdin = io::stdin();
    let opt = Opt::from_args();
    for chunk in &chunk_lines(opt.limit, stdin.lock().lines()) {
        // TODO: is it OK to unwrap?
        let input: Vec<String> = chunk.into_iter().map(|s| s.unwrap().to_owned()).collect();
        if opt.verbose {
            println!("{}", opt.command.to_owned() + " " + &input.join(" "));
        }
        Command::new(&opt.command)
            .args(opt.fixed_args.clone().into_iter().chain(input))
            .spawn()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_lines_for_command() {
        let input = vec!["one", "two", "three"];
        let chunked = &chunk_lines(2, input.into_iter());
        let mut output = chunked.into_iter();

        assert_eq!(output.next().unwrap().collect::<Vec<&str>>(), vec!("one", "two"));
        assert_eq!(output.next().unwrap().collect::<Vec<&str>>(), vec!("three"));
        assert!(output.next().is_none());
    }

}