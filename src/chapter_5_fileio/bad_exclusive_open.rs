use nix::errno::Errno;
use std::error::Error;

use nix::fcntl::open;
use nix::fcntl::OFlag;
use nix::sys::stat::Mode;
use nix::unistd::{close, getpid};

use std::{io, thread, time};

fn exclusive_open(file_path: &str, _sleep: bool) -> Result<(), Box<dyn Error>> {
    let pid = getpid();

    /*
    O_EXCL
    If O_CREAT and O_EXCL are set, open() shall fail if the file exists. The check for the existence
    of the file and the creation of the file if it does not exist shall be atomic with respect to
    other threads executing open() naming the same filename in the same directory with O_EXCL and
    O_CREAT set. If O_EXCL and O_CREAT are set, and path names a symbolic link, open() shall fail
    and set errno to [EEXIST], regardless of the contents of the symbolic link. If O_EXCL is set
    and O_CREAT is not set, the result is undefined.
    */
    match open(
        file_path,
        OFlag::O_WRONLY | OFlag::O_CREAT | OFlag::O_EXCL,
        Mode::S_IRUSR | Mode::S_IWUSR,
    ) {
        Ok(_fd) => {
            println!("[PID: {pid}] Created file {file_path} exclusively");
        }
        Err(err) => {
            if err == Errno::EEXIST {
                return Err(Box::new(io::Error::new(io::ErrorKind::AlreadyExists, err)));
            }

            return Err(Box::new(io::Error::new(io::ErrorKind::Other, err)));
        }
    }

    Ok(())
}

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

    open(
        file_path,
        OFlag::O_WRONLY | OFlag::O_CREAT,
        Mode::S_IRUSR | Mode::S_IWUSR,
    )?;
    println!("[PID: {pid}] Created file {file_path} exclusively");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::remove_file;
    use std::io::ErrorKind;
    use std::thread;

    #[test]
    fn test_run_bad_exclusive_open() {
        remove_file("/tmp/d").ok();
        bad_exclusive_open("/tmp/d", false).unwrap();
    }
    #[test]
    #[should_panic]
    fn test_run_bad_exclusive_open_panics_if_exists() {
        bad_exclusive_open("/tmp/d2", false).unwrap();
        bad_exclusive_open("/tmp/d2", false).unwrap();
    }

    #[test]
    #[should_panic] // file is opened twice!
    fn test_bad_exclusive_open_is_not_atomic() {
        let file_path = "/dev/shm/conflicting_file_not_atomic";
        remove_file(file_path).ok();

        // opens the file and sleeps
        let _ = thread::spawn(move || bad_exclusive_open(file_path, true).unwrap());

        let result = thread::spawn(move || {
            // this should result in an error, since the file is opened by the first thread
            assert!(bad_exclusive_open(file_path, false).is_err());
        });

        // is the assumption right?
        assert!(result.join().is_ok());
    }

    #[test]
    fn test_exclusive_open_is_atomic() {
        let file_path = "/dev/shm/conflicting_file";
        remove_file(file_path).ok();

        // opens the file and sleeps
        let _ = thread::spawn(move || exclusive_open(file_path, true).unwrap());

        let handle = thread::spawn(move || {
            // this should result in an error, since the file is opened by the first thread
            let error = exclusive_open(file_path, false).unwrap_err();
            assert_eq!(error.to_string(), "EEXIST: File exists");
        });

        // is the assumption right?
        assert!(handle.join().is_ok());
    }
}
