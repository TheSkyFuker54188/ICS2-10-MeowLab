use std::env;
use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::process;
use std::ptr;

fn io_blocksize() -> usize {
    // 获取系统页面大小
    unsafe {
        let page_size = libc::sysconf(libc::_SC_PAGESIZE);
        if page_size < 0 {
            // 如果获取失败，使用默认值4KB
            4096
        } else {
            page_size as usize
        }
    }
}

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
    
    // 获取缓冲区大小（内存页大小）
    let buffer_size = io_blocksize();
    
    // 动态分配缓冲区
    let buffer: Vec<u8> = vec![0; buffer_size];
    let buffer_ptr = buffer.as_ptr() as *mut libc::c_void;
    
    loop {
        let bytes_read = unsafe {
            libc::read(fd, buffer_ptr, buffer_size)
        };
        
        if bytes_read < 0 {
            eprintln!("Error reading file");
            process::exit(1);
        }
        
        if bytes_read == 0 {
            // 文件结束
            break;
        }
        
        let mut total_written = 0;
        while total_written < bytes_read {
            let bytes_written = unsafe {
                libc::write(
                    stdout_fd,
                    (buffer_ptr as *const u8).offset(total_written as isize) as *const libc::c_void,
                    (bytes_read - total_written) as usize
                )
            };
            
            if bytes_written < 0 {
                eprintln!("Error writing to stdout");
                process::exit(1);
            }
            
            total_written += bytes_written;
        }
    }
}
