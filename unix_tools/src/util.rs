//! 共享工具:打开输入源 + 退出码辅助
//!
//! 给 cat / wc / grep 共用,因为三个工具都要:
//! 1. 接受 "-" 表示 stdin
//! 2. 接受文件路径
//! 3. 用 BufReader 包装
//! 4. 返回统一的 Box<dyn BufRead>

use crate::error::UtError;
use std::fs::File;
use std::io::{BufRead, BufReader, stdin};

/// 打开输入源:若 path 为 "-" 则返回 stdin,否则打开文件
///
/// **返回类型 `Box<dyn BufRead>` 的原因**:
/// - `File` 和 `StdinLock` 都实现了 `BufRead`,但它们类型不同,函数只能返回一个
/// - `dyn BufRead` 是 trait object,运行时多态
/// - `Box<...>` 堆分配是因为 trait object 大小不固定
///
/// **为什么总是包 BufReader**:
/// - `File` 和 `StdinLock` 本身**不**是 buffered,直接 read() 是无 buffer
/// - 包 BufReader 后用 8KB buffer,大文件读效率高
///
/// **错误信息带文件名**:
/// - 用 `map_err` 把 IO 错误转成带 context 的 UtError
/// - 用户看到 "cannot open 'a.txt': No such file or directory" 而不只是 "No such file"
pub fn open_input(path: &str) -> Result<Box<dyn BufRead>, UtError> {
    if path == "-" {
        // stdin 是全局的,lock() 拿到互斥访问的句柄
        // BufReader::new(stdin().lock()) 跟普通 BufReader 一样用
        Ok(Box::new(BufReader::new(stdin().lock())))
    } else {
        let f = File::open(path).map_err(|e| UtError::Io {
            context: format!("cannot open '{}'", path),
            source: e,
        })?;
        Ok(Box::new(BufReader::new(f)))
    }
}
