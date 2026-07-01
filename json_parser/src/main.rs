//! JSON 解析器的命令行入口。
//!
//! ## 用法
//!
//! 从 stdin 读取一段 JSON 文本,解析后把 [`JsonValue`](crate::JsonValue) 用 `{:#?}` 格式化打印到 stdout。
//!
//! 退出码:
//! - `0` —— 解析成功
//! - `1` —— 解析失败 / 读取 stdin 失败(错误信息打到 stderr)
//!
//! ## 例子
//!
//! ```text
//! $ echo '{"name": "lccxxo", "age": 30}' | cargo run
//! Object {
//!     "name": String("lccxxo"),
//!     "age": Number(30.0),
//! }
//! ```
//!
//! ## 在 PowerShell / Git Bash 里
//!
//! ```powershell
//! PS> '{"x": 1}' | cargo run
//! ```
//!
//! ```bash
//! $ echo '{"x": 1}' | cargo run
//! ```
//!
//! ## 错误示例
//!
//! ```text
//! $ echo "[1, 2,]" | cargo run
//! line 1, col 1: trailing comma in array
//! ```
//!
//! ## 为什么是 CLI 而不是 REPL
//!
//! 这只是为了演示 [`parse`](crate::parse) 的用法。要做交互式 REPL,在 `loop { ... }` 里包一层
//! `read_line` 即可,逻辑跟本文件几乎一样。

use std::io::Read;
use std::process::ExitCode;

/// 命令行入口:从 stdin 读 JSON,解析,打印结果。
///
/// 退出码语义:
/// - 成功 → 0
/// - 读取 stdin 失败 → 1
/// - 解析失败 → 1
fn main() -> ExitCode {
    // 1. 把 stdin 全部读到一个 String
    let mut input = String::new();
    if let Err(e) = std::io::stdin().read_to_string(&mut input) {
        eprintln!("error reading stdin: {}", e);
        return ExitCode::from(1);
    }

    // 2. 调库入口 parse
    match json_parser::parse(&input) {
        // 成功:{:#?} 美化打印 JsonValue
        Ok(value) => {
            println!("{:#?}", value);
            ExitCode::SUCCESS
        }
        // 失败:错误信息打到 stderr
        Err(e) => {
            eprintln!("{}", e);
            ExitCode::from(1)
        }
    }
}
