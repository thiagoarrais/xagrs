use std::io::{self, BufRead, Error};
use std::process::Command;

use itertools::{Chunk, IntoChunks, Itertools};
use structopt::clap::AppSettings;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "xagrs")]
#[structopt(settings = &[AppSettings::TrailingVarArg])]
struct Opt {
    #[structopt(short = "L", default_value = "1")]
    limit: usize,

    #[structopt(short = "t", long = "verbose")]
    verbose: bool,

    #[structopt(name = "command", help = "Command (with arguments) to execute")]
    command_with_args: Vec<String>,
}

impl Opt {
    fn chunker<F>(
        self: &Self,
        lines: std::io::Lines<std::io::StdinLock>,
        mut op: F,
    ) -> Result<(), Error>
    where
        F: FnMut(Chunk<std::io::Lines<std::io::StdinLock>>) -> Result<(), Error>,
    {
        for chunk in &chunk_lines(self.limit, lines) {
            op(chunk)?
        }
        Ok(())
    }

    fn executor<'a>(
        self: &'a Self,
        command: &'a str,
        fixed_args: &'a [String],
    ) -> impl FnMut(&[String]) -> Result<(), Error> + 'a {
        move |input| {
            if self.verbose {
                let mut command_line = vec![command.to_owned()];
                command_line.extend(fixed_args.to_owned());
                command_line.extend(input.to_owned());
                println!("{}", &command_line.join(" "));
            }
            Command::new(command.clone())
                .args(fixed_args.clone().into_iter().chain(input))
                .spawn()?
                .wait()?;
            Ok(())
        }
    }
}

fn chunk_lines<L>(limit: usize, lines: L) -> IntoChunks<L>
where
    L: Iterator,
{
    lines.chunks(limit)
}

// TODO: make error more friendly
fn main() -> Result<(), Error> {
    let stdin = io::stdin();
    let mut opt = Opt::from_args();
    let command = if opt.command_with_args.len() > 0 {
        opt.command_with_args.remove(0)
    } else {
        String::from("echo")
    };
    let fixed_args = opt.command_with_args.clone();
    let mut executor = opt.executor(&command, &fixed_args);
    opt.chunker(
        stdin.lock().lines(),
        |chunk: Chunk<std::io::Lines<std::io::StdinLock>>| {
            // TODO: is it OK to unwrap?
            let input: Vec<String> = chunk.into_iter().map(|s| s.unwrap()).collect();
            executor(&input)?;
            Ok(())
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_lines_for_command() {
        let input = vec!["one", "two", "three"];
        let chunked = &chunk_lines(2, input.into_iter());
        let mut output = chunked.into_iter();

        assert_eq!(
            output.next().unwrap().collect::<Vec<&str>>(),
            vec!("one", "two")
        );
        assert_eq!(output.next().unwrap().collect::<Vec<&str>>(), vec!("three"));
        assert!(output.next().is_none());
    }
}
