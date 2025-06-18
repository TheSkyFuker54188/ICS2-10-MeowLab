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

fn align_alloc(size: usize) -> *mut u8 {
    let page_size = io_blocksize();
    
    // 分配比需要的大小多一些的内存，以便对齐
    let total_size = size + page_size;
    
    unsafe {
        let raw_ptr = libc::malloc(total_size) as *mut u8;
        if raw_ptr.is_null() {
            panic!("Failed to allocate memory");
        }
        
        // 计算对齐后的地址
        let aligned_addr = (raw_ptr as usize + page_size - 1) & !(page_size - 1);
        let aligned_ptr = aligned_addr as *mut u8;
        
        // 在对齐后的指针前面存储原始指针，用于释放时使用
        let header_ptr = (aligned_ptr as *mut usize).offset(-1);
        *header_ptr = raw_ptr as usize;
        
        aligned_ptr
    }
}

fn align_free(ptr: *mut u8) {
    if ptr.is_null() {
        return;
    }
    
    unsafe {
        // 获取原始指针
        let header_ptr = (ptr as *mut usize).offset(-1);
        let raw_ptr = *header_ptr as *mut libc::c_void;
        libc::free(raw_ptr);
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
    
    // 分配对齐的缓冲区
    let buffer_ptr = align_alloc(buffer_size + std::mem::size_of::<usize>()) as *mut libc::c_void;
    
    loop {
        let bytes_read = unsafe {
            libc::read(fd, buffer_ptr, buffer_size)
        };
        
        if bytes_read < 0 {
            eprintln!("Error reading file");
            align_free(buffer_ptr as *mut u8);
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
                align_free(buffer_ptr as *mut u8);
                process::exit(1);
            }
            
            total_written += bytes_written;
        }
    }
    
    // 释放对齐的缓冲区
    align_free(buffer_ptr as *mut u8);
}
