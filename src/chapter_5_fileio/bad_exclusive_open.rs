use nix::errno::Errno;
use std::error::Error;

use nix::fcntl::open;
use nix::fcntl::OFlag;
use nix::sys::stat::Mode;
use nix::unistd::{close, getpid, read, write};

use std::{io, thread, time};

fn bad_exclusive_open(file_path: &str, sleep: bool) -> Result<(), Box<dyn Error>> {
    let pid = getpid();

    match open(file_path, OFlag::O_WRONLY, Mode::all()) {
        Ok(fd) => {
            eprintln!("[PID: {pid}] File Already exists");
            close(fd)?;
            return Err(Box::new(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "File already exists",
            )));
        }
        Err(error) => {
            if error != Errno::ENOENT {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Open {error}"),
                )));
            }
        }
    }

    if sleep {
        thread::sleep(time::Duration::from_millis(5000))
    }
    // not atomic!! One can open the file here

    match open(
        file_path,
        OFlag::O_WRONLY | OFlag::O_CREAT,
        Mode::S_IRUSR | Mode::S_IWUSR,
    ) {
        Ok(fd) => {
            println!("[PID: {pid}] Created file {file_path} exclusively");
        }
        Err(err) => {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                format!("Open: {err}"),
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::remove_file;
    use std::thread;

    #[test]
    fn test_run_bad_exclusive_open() {
        remove_file("/tmp/d");
        bad_exclusive_open("/tmp/d", false).unwrap();
    }
    #[test]
    #[should_panic]
    fn test_run_bad_exclusive_open_panics_id_exists() {
        bad_exclusive_open("/tmp/d2", false).unwrap();
        bad_exclusive_open("/tmp/d2", false).unwrap();
    }

    #[test]
    #[should_panic] // file is opened twice!
    fn test_bad_exclusive_open_is_not_atomic() {
        let file_path = "/dev/shm/conflicting_file";
        remove_file(file_path);

        // opens the file and sleeps
        let _ = thread::spawn(move || bad_exclusive_open(file_path, true).unwrap());

        let result = thread::spawn(move || {
            // this should result in an error, since the file is opened by the first thread
            assert!(bad_exclusive_open(file_path, false).is_err());
        });

        // is the assumption right?
        assert!(result.join().is_ok());
    }
}
