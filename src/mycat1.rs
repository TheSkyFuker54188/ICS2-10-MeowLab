use std::env;
use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        process::exit(1);
    }
    
    let filename = &args[1];
    
    // 打开文件
    let file = match File::open(filename) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Error opening file '{}': {}", filename, err);
            process::exit(1);
        }
    };
    
    let fd = file.as_raw_fd();
    let stdout_fd = 1; // stdout文件描述符
    
    // 每次读取一个字符
    loop {
        let mut buffer = [0u8; 1];
        
        let bytes_read = unsafe {
            libc::read(fd, buffer.as_mut_ptr() as *mut libc::c_void, 1)
        };
        
        if bytes_read < 0 {
            eprintln!("Error reading file");
            process::exit(1);
        }
        
        if bytes_read == 0 {
            // 文件结束
            break;
        }
        
        let bytes_written = unsafe {
            libc::write(stdout_fd, buffer.as_ptr() as *const libc::c_void, 1)
        };
        
        if bytes_written < 0 {
            eprintln!("Error writing to stdout");
            process::exit(1);
        }
    }
}
