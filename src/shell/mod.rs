use fork::{fork, Fork}; // https://docs.rs/fork/0.1.18/fork/
use std::{io::stdin, process::exit};

mod command;
use command::ShellCmd;

/// Parent shell process does the following in a loop:
/// - get input from user
/// - spawn child to execute user command
pub fn sh_loop() {
    let mut input = String::new();
    let stdin = stdin();

    loop {
        if let Err(error) = stdin.read_line(&mut input) {
            println!("error when reading input (Error: {error})");
        }
        input.pop(); // get rid of newline
        let args_vec: Vec<String> = input.split(" ").map(|x| String::from(x)).collect();
        match ShellCmd::new(&args_vec[0..args_vec.len()]) {
            Ok(command) => match fork() {
                Ok(Fork::Parent(_)) => (),
                Ok(Fork::Child) => sh_launch(command),
                Err(error) => println!("error when forking shell process (error: {error})"),
            },

            Err(errno) => println!("error when parsing command (Errno: {errno}"),
        }
    }
}

/// Launching a command from a child process.
/// Process exits or gives control to a different process after executing this function.
fn sh_launch(command: ShellCmd) {
    match command {
        ShellCmd::Exec { args } => {
            let err = exec::execvp(args[0].clone(), args); // only returns if there's an error
            println!("Error executing program {}:\n{}", args[0], err);
        }
        ShellCmd::Redir {
            command,
            descriptor,
            readmode,
        } => todo!(),
        ShellCmd::Pipe { left, right } => todo!(),
    }

    exit(0)
}
