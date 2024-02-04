use rustix::{
    fd::OwnedFd,
    fs::{open, Mode, OFlags},
    io::Errno,
}; // https://docs.rs/rustix/latest/rustix/

#[derive(Debug)]
pub enum ShellCmd<'a> {
    Exec {
        args: &'a [String],
    },
    Redir {
        command: Box<ShellCmd<'a>>,
        descriptor: OwnedFd,
        readmode: bool,
    },
    Pipe {
        left: Box<ShellCmd<'a>>,
        right: Box<ShellCmd<'a>>,
    },
    Nothing,
}

impl<'a> ShellCmd<'a> {
    /// Recursively creates a shell command.
    /// Should initially be envoked with a simple slice of some vector of arguments.
    pub fn new(args: &'a [String]) -> Result<Self, Errno> {
        // check for silly empty args
        if args == [""] {
            return Ok(Self::Nothing);
        }

        // find special symbols,
        // right now includes `|`, `>`, and `<`
        let args_iterator = args.into_iter();
        if let Some(first_pipe) = args_iterator.clone().position(|x| x == "|") {
            // Pipes have lowest precedence in current version
            // create and return a Pipe variant
            Ok(Self::Pipe {
                left: Box::new(Self::new(&args[0..first_pipe])?),
                right: Box::new(Self::new(&args[first_pipe + 1..args.len()])?),
            })
        } else if let Some(first_larrow) = args_iterator.clone().position(|x| x == "<") {
            // create and return a Redir variant with readmode on,
            // files are always on the rhs
            Ok(Self::Redir {
                command: Box::new(Self::new(&args[0..first_larrow])?),
                descriptor: open(args[first_larrow + 1].clone(), OFlags::RDONLY, Mode::RUSR)?,
                readmode: true,
            })
        } else if let Some(first_rarrow) = args_iterator.clone().position(|x| x == ">") {
            // create and return a Redir variant with readmode off,
            // files are always on the rhs
            Ok(Self::Redir {
                command: Box::new(Self::new(&args[0..first_rarrow])?),
                descriptor: open(
                    args[first_rarrow + 1].clone(),
                    OFlags::WRONLY.union(OFlags::CREATE),
                    Mode::WUSR,
                )?,
                readmode: false,
            })
        } else {
            Ok(Self::Exec { args })
        }
    }
}
