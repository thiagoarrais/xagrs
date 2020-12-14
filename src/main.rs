use std::process::Command;
use std::env;
use std::io::{self, BufRead};
use std::iter;

fn main() {
    let stdin = io::stdin();
    let mut args = env::args().skip(1);
    let command_opt = args.next();
    if let Some(command) = command_opt {
        let command_args : Vec<String> = args.collect();
        for line in stdin.lock().lines() {
            Command::new(&command)
                .args(command_args.iter().chain(iter::once(&line.unwrap())))
                .spawn()
                .expect("something went wrong");
        }
    }
}
