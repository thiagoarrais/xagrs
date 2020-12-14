use std::process::Command;
use std::env;

fn main() {
    let mut args = env::args().skip(1);
    let command_opt = args.next();
    if let Some(command) = command_opt {
        Command::new(command)
            .args(args)
            .spawn()
            .expect("something went wrong");
    }
}
