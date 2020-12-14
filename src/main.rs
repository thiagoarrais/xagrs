use std::process::Command;
use std::env;
use std::io::{self, BufRead};
use std::iter;

fn main() {
    let stdin = io::stdin();
    let mut args = env::args().skip(1);
    let command_opt = args.next();
    if let Some(command) = command_opt {
        let line = stdin.lock().lines().next().unwrap().unwrap();
        Command::new(&command)
            .args(args.chain(iter::once(line)))
            .spawn()
            .expect("something went wrong");
    }
}
