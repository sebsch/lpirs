use clap::Parser;
use nix::fcntl::{open, OFlag};
use nix::libc::{STDIN_FILENO, STDOUT_FILENO};
use nix::sys::stat::Mode;
use nix::unistd::{close, read, write};
use std::error::Error;
use std::os::fd::RawFd;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg()]
    path: String,

    #[arg(short, action, help("append to existing file"))]
    append: bool,
}

struct Tea {
    fd_file_out: RawFd,
}

impl Tea {
    pub fn new(args: Args) -> Result<Tea, Box<dyn Error>> {
        let mut o_flags = OFlag::O_CREAT | OFlag::O_WRONLY;

        if args.append {
            o_flags |= OFlag::O_APPEND;
        }

        let fd_file_out = open(&args.path[..], o_flags, Mode::all())?;
        Ok(Tea { fd_file_out })
    }

    pub fn pour(&mut self) -> Result<(), Box<dyn Error>> {
        let mut buf = vec![0u8; 1024];

        loop {
            let num_read = read(STDIN_FILENO, &mut buf[..])?;
            if num_read == 0 {
                break;
            }
            write(STDOUT_FILENO, &buf[..num_read])?;
            write(self.fd_file_out, &buf[..num_read])?;
        }
        close(self.fd_file_out)?;

        Ok(())
    }
}

fn main() {
    let args = Args::parse();

    match Tea::new(args) {
        Err(error) => {
            eprintln!("Initializing Error: {error:#?}");
        }

        Ok(mut tea) => {
            if let Err(error) = tea.pour() {
                eprintln!("Operation Error: {error:#?}")
            }
        }
    }
}
