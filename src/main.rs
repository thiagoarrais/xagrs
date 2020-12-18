use std::process::Command;
use std::io::{self, BufRead, Error};

use itertools::{ Itertools, IntoChunks };
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "xagrs")]
struct Opt {
    #[structopt(short = "L", default_value = "1")]
    limit: usize,

    // #[structopt(short = "L", default_value = "1")]
    command_args: Vec<String>,
}

fn chunk_lines<L>(limit: usize, lines: L) -> IntoChunks<L>
  where L: Iterator
{
    lines.chunks(limit)
}

fn main() -> Result<(), Error> {
    let stdin = io::stdin();
    let mut opt = Opt::from_args();
    if opt.command_args.len() > 0 {
        let command = opt.command_args.remove(0);
        for chunk in &chunk_lines(opt.limit, stdin.lock().lines()) {
            // TODO: is it OK to unwrap?
            let input = chunk.into_iter().map(|s| s.unwrap().to_owned());
            Command::new(&command)
                .args(opt.command_args.clone().into_iter().chain(input))
                .spawn()?;
        }
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