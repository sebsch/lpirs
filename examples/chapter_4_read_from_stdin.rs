use nix::libc::STDIN_FILENO;
use nix::libc::STDOUT_FILENO;
use nix::unistd::{read, write};

fn main() {
    let buf_size = 10;
    let mut buf = vec![0u8; buf_size];

    let num_read = read(STDIN_FILENO, &mut buf[..]).unwrap();

    // Hallo
    // input: [72, 97, 108, 108, 111, 10, 0, 0, 0, 0]
    // on rust the buf is initialized with 0 and read always adds the ASCII 10 (LF)
    // buf[num_read] = b'\0';

    println!("input: {buf:?}");

    write(STDOUT_FILENO, &buf[..num_read]).unwrap();
}
