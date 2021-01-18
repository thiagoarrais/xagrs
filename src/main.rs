use std::io::{self, BufRead, Error};
use std::process::Command;

use itertools::{Chunk, IntoChunks, Itertools};
use structopt::clap::AppSettings;
use structopt::StructOpt;

#[derive(Debug, Default, StructOpt)]
#[structopt(name = "xagrs")]
#[structopt(settings = &[AppSettings::TrailingVarArg])]
struct Opt {
    #[structopt(short = "L", default_value = "1")]
    limit: usize,

    #[structopt(short = "i", long = "replace")]
    replace: Option<String>,

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

    fn program(self: &Self) -> &str {
        if self.command_with_args.len() > 0 {
            &self.command_with_args[0]
        } else {
            "echo"
        }
    }

    fn fixed_args(self: &Self) -> &[String] {
        &self.command_with_args[1..]
    }

    fn command(self: &Self, input: &[String]) -> (String, Vec<String>) {
        let args = match &self.replace {
            None => self
                .fixed_args()
                .clone()
                .into_iter()
                .map(|s| s.to_owned())
                .chain(
                    input
                        .iter()
                        .flat_map(|s| s.split_whitespace().map(|s| s.to_owned())),
                )
                .collect(),
            Some(pattern) => {
                let joined_input = input.join(" ");
                self.fixed_args()
                    .clone()
                    .into_iter()
                    .map(|s| s.replace(pattern, &joined_input))
                    .collect()
            }
        };

        (self.program().to_owned(), args)
    }

    fn executor<'a>(self: &'a Self) -> impl FnMut(&[String]) -> Result<(), Error> + 'a {
        move |input| {
            let (program, args) = self.command(input);
            if self.verbose {
                let mut command_line = vec![program.clone()];
                command_line.extend(args.to_owned());
                println!("{}", &command_line.join(" "));
            }
            Command::new(program).args(args).spawn()?.wait()?;
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
    let opt = Opt::from_args();
    let mut executor = opt.executor();
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
    fn echo_is_default_command() {
        let mut opt = Opt::default();
        opt.command_with_args = vec![];
        let opt = opt;

        assert_eq!(opt.program(), "echo");
    }

    #[test]
    fn splits_program_and_args() {
        let mut opt = Opt::default();
        opt.command_with_args = vec!["program", "arg1", "arg2", "arg3"]
            .into_iter()
            .map(|s| s.to_owned())
            .collect();
        let opt = opt;

        assert_eq!(opt.program(), "program");
        assert_eq!(opt.fixed_args(), ["arg1", "arg2", "arg3"]);
    }
    #[test]
    fn concats_input_to_the_end_of_fixed_args() {
        let mut opt = Opt::default();
        opt.command_with_args = vec!["program", "arg1", "arg2"]
            .into_iter()
            .map(|s| s.to_owned())
            .collect();
        let opt = opt;

        let (_, args) = opt.command(
            &vec!["input1", "input2", "input3"]
                .into_iter()
                .map(|s| s.to_owned())
                .collect::<Vec<String>>(),
        );
        assert_eq!(args, ["arg1", "arg2", "input1", "input2", "input3"])
    }

    #[test]
    fn multiple_words_in_input_interpreted_as_multiple_arguments() {
        let mut opt = Opt::default();
        opt.command_with_args = vec!["program", "fixed"]
            .into_iter()
            .map(|s| s.to_owned())
            .collect();
        let opt = opt;

        let (_, args) = opt.command(
            &vec!["first  second"]
                .into_iter()
                .map(|s| s.to_owned())
                .collect::<Vec<String>>(),
        );
        assert_eq!(args, ["fixed", "first", "second"])
    }

    #[test]
    fn replaces_pattern_with_input() {
        let mut opt = Opt::default();
        opt.replace = Some(String::from("PP"));
        opt.command_with_args = vec!["program", "PP", "abcPPghi", "jklm"]
            .into_iter()
            .map(|s| s.to_owned())
            .collect();
        let opt = opt;

        let (_, args) = opt.command(
            &vec!["def"]
                .into_iter()
                .map(|s| s.to_owned())
                .collect::<Vec<String>>(),
        );
        assert_eq!(args, ["def", "abcdefghi", "jklm"])
    }

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
