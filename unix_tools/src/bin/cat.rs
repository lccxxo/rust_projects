//! ut_cat:实现 GNU cat 的核心功能
//!
//! 设计要点:
//! - 无 `-n` 时:用 `std::io::copy` 字节流,零拷贝,大文件不爆内存
//! - 有 `-n` 时:必须按行读(为了计数 + 加行号),内存常驻 8KB buffer
//! - 多文件:循环,不在内存里拼接
//! - 退出:主循环成功 → ExitCode::SUCCESS,任一错误 → die(e)

use std::io::{copy, Write};
use std::process::ExitCode;
use unix_tools::error::{die, UtError};
use unix_tools::util::open_input;

/// cat 命令行参数
///
/// 用 struct 装所有 flag 是 Rust 惯例:
/// - 比一堆独立变量好读
/// - 扩展 flag 不用改函数签名
/// - 配合 parse() 函数,parse 完直接拿
struct CatArgs {
    number_lines: bool,
    files: Vec<String>,
}

impl CatArgs {
    /// 解析 argv
    ///
    /// **为什么手动解析而不是用 clap**:
    /// - MVP 只要 1 个 flag(`-n`),上 clap 杀鸡用牛刀
    /// - clap 会引入新依赖、build 变慢、二进制变大
    /// - 学 Rust 的好习惯是先把基础库用熟
    ///
    /// **参数约定**(GNU 兼容):
    /// - `-n` 任意位置都识别(不要求在文件前)
    /// - 任何 `-X`(X 不是 n) → 参数错
    /// - 单独 `-` 视为文件名(读 stdin)
    fn parse(argv: &[String]) -> Result<Self, UtError> {
        let mut number = false;
        let mut files = Vec::new();
        for arg in argv {
            if arg == "-n" {
                number = true;
            } else if arg.starts_with('-') && arg != "-" {
                // 排除 "-" 自身(那是文件名,不是 flag)
                return Err(UtError::Parse(format!("unknown flag: {}", arg)));
            } else {
                files.push(arg.clone());
            }
        }
        Ok(Self { number_lines: number, files })
    }
}

/// 主逻辑:解析 args 后调用
///
/// 返回 `Result<(), UtError>` 让 main 处理错误(避免 main 里堆 try-catch)
fn run(args: CatArgs) -> Result<(), UtError> {
    // 无文件参数 → 默认读 stdin(GNU cat 行为)
    let files: Vec<String> = if args.files.is_empty() {
        vec!["-".to_string()]
    } else {
        args.files
    };

    // stdout 锁:标准输出是全局资源,多线程要锁;单线程可省但留着更稳
    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    for path in &files {
        // open_input 出错 → 立即返回(后面文件不处理)
        let mut input = open_input(path)?;
        if args.number_lines {
            // -n 模式:必须按行
            // buffer 复用:每次 clear 不重新分配,避免 1GB 文件百万次 malloc
            let mut line_no = 1u64;
            let mut buf = String::new();
            loop {
                buf.clear();
                // read_line:读到 \n 停止(不含 EOF)
                // 返回值 n = 读到的字节数;n == 0 表示 EOF
                let n = std::io::BufRead::read_line(&mut input, &mut buf)?;
                if n == 0 { break; }
                // `{:>6}` 右对齐 6 字符,GNU cat 风格
                write!(out, "{:>6}  {}", line_no, buf)?;
                line_no += 1;
            }
        } else {
            // 无 -n:字节流 copy,效率最高
            // copy 内部循环 read+write,直到 EOF
            // 大文件:内存常驻 ~8KB(read 端 buffer)
            copy(&mut input, &mut out)?;
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    // 两段 match:参数错 vs 运行时错
    // - 参数错直接 die(不进入 run)
    // - 运行时错(IO)在 run 里返 Err,这里再 die
    match CatArgs::parse(&argv) {
        Ok(args) => match run(args) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => die(e),
        },
        Err(e) => die(e),
    }
}