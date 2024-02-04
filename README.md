# apemode
A rusty shell for *nix systems.

### Features

Current features are limited to:
- straight execution of programs
- I/O redirection with files
- I/O redirection between processes with pipes
- changing working directory of shell
- running list programs

### Future goals

Future goals include
globbing, history,
running background tasks,
interrupt handling,
user customization (a la "dotfiles"),
and easy integration with systems.

Beyond the virtues of having memory and I/O safety
with Rust and the crates involved with this project,
I aim for `apemode` to be as lightweight as, if not moreso than,
peer shell programs.

##### Dependencies

- [rustix](https://crates.io/crates/rustix)
- [nix](https://docs.rs/nix/latest/nix/)
- [exec](https://docs.rs/exec/latest/exec/)
