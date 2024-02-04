use nix::unistd::{fork, ForkResult};
use rustix::{
    fd::OwnedFd,
    pipe::pipe,
    process::{chdir, wait, WaitOptions},
    stdio::{dup2_stdin, dup2_stdout},
};
use std::{
    io::{stdin, stdout, Write},
    process::exit,
};

mod command;
use command::ShellCmd;

/// Parent shell process does the following in a loop:
/// - get input from user
/// - spawn child to execute user command
pub fn sh_loop() {
    let stdin = stdin();

    print!("> ");
    loop {
        if let Err(_) = stdout().flush() {
            // like a clogged toilet, sometimes flushing a second time will solve everything
            continue;
        }

        let mut input = String::new();
        if let Err(error) = stdin.read_line(&mut input) {
            println!("error when reading input (Error: {error})");
        }
        input.pop(); // get rid of newline
        let args_vec: Vec<String> = input.split(" ").map(|x| String::from(x)).collect();
        match ShellCmd::new(&args_vec[0..args_vec.len()]) {
            Ok(command) => {
                if let ShellCmd::Nothing = command {
                    continue;
                }
                parent_prep(command);
            }

            Err(errno) => println!("error when parsing command (Errno: {errno}"),
        };

        print!("> "); // prevents stacks on itself if we retry flushing
    }
}

fn parent_prep(command: ShellCmd) {
    if let ShellCmd::Chdir { path } = &command {
        if let Err(errno) = chdir(path) {
            println!("error when changing directory (Errno: {errno})");
        }
        return ();
    }
    match unsafe { fork() } {
        Ok(ForkResult::Parent { .. }) => {
            if let Err(errno) = wait(WaitOptions::CONTINUED) {
                println!("error when waiting for child process (Errno: {errno}");
            }
        }
        Ok(ForkResult::Child) => sh_launch(command),
        Err(errno) => println!("error when forking shell process (errno: {errno})"),
    }
}

/// Launching a command from a child process.
/// Process exits or gives control to a different process after executing this function.
fn sh_launch(cmd: ShellCmd) {
    match cmd {
        ShellCmd::Exec { args } => {
            let err = exec::execvp(args[0].clone(), args); // only returns if there's an error
            println!("error executing program {}:\n{}", args[0], err);
        }
        ShellCmd::Redir {
            command,
            descriptor,
            readmode,
        } => launch_redir(*command, descriptor, readmode),
        ShellCmd::Pipe { left, right } => launch_pipe(*left, *right),
        ShellCmd::Nothing | _ => (), // empty input or somehow Chdir got through the cracks
    }

    exit(0)
}

fn launch_redir(command: ShellCmd, descriptor: OwnedFd, readmode: bool) {
    if readmode {
        // replace stdin file descriptor with `descriptor`
        if let Err(errno) = dup2_stdin(descriptor) {
            println!("error when changing file descriptors (Errno: {errno}");
            exit(-1);
        }
    } else {
        // replace stdout file descriptor with `descriptor`
        if let Err(errno) = dup2_stdout(descriptor) {
            println!("error when changing file descriptors (Errno: {errno})");
            exit(-1);
        }
    }
    sh_launch(command);
}

fn launch_pipe(left: ShellCmd, right: ShellCmd) {
    match pipe() {
        Ok(p) => {
            // create write-end (left) process
            match unsafe { fork() } {
                Ok(ForkResult::Child) => {
                    if let Err(errno) = dup2_stdout(p.1) {
                        println!("error when changing file descriptors (Errno: {errno}");
                        exit(-1);
                    }
                    drop(p.0);
                    sh_launch(left);
                    exit(-1); // exit if sh_launch somehow fails
                }
                Err(errno) => {
                    println!("error when forking shell process (errno: {errno})");
                    exit(-1);
                }
                _ => (),
            }

            match unsafe { fork() } {
                Ok(ForkResult::Child) => {
                    if let Err(errno) = dup2_stdin(p.0) {
                        println!("error when changing file descriptors (Errno: {errno}");
                        exit(-1);
                    }
                    drop(p.1);
                    sh_launch(right);
                    exit(-1); // exit if sh_launch somehow fails
                }
                Err(errno) => {
                    println!("error when forking shell process (errno: {errno})");
                    exit(-1);
                }
                _ => (),
            }

            exit(0);
        }
        Err(errno) => {
            println!("error creating pipe (Errno: {errno}");
            exit(-1);
        }
    }
}
