use nix::errno::Errno;
use std::error::Error;

use nix::fcntl::open;
use nix::fcntl::OFlag;
use nix::libc::O_CREAT;
use nix::sys::stat::Mode;
use nix::unistd::{close, getpid, read, write};

fn bad_exclusive_open(file_path: &str) -> Result<(), Box<dyn Error>> {
    let pid = getpid();

    match open(file_path, OFlag::O_WRONLY, Mode::all()) {
        Ok(fd) => {
            eprintln!("[PID: {pid}] File Already exists");
            close(fd)?;
            panic!("file exists")
        }
        Err(error) => {
            if error != Errno::ENOENT {
                panic!("open");
            }
        }
    }

    let fd = open(
        file_path,
        OFlag::O_WRONLY | OFlag::O_CREAT,
        Mode::S_IRUSR | Mode::S_IWUSR,
    )
    .expect("open");

    println!("[PID: {pid}] Created file {file_path} exclusively");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::remove_file;

    #[test]
    fn test_run_bad_exclusive_open() {
        remove_file("/tmp/d");
        bad_exclusive_open("/tmp/d").unwrap();
    }
    #[test]
    #[should_panic]
    fn test_run_bad_exclusive_open_panics_id_exists() {
        bad_exclusive_open("/tmp/d2").unwrap();
        bad_exclusive_open("/tmp/d2").unwrap();
    }
}
