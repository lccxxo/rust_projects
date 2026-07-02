//! JSON 解析器的交互式 REPL。
//!
//! ## 用法
//!
//! 启动后,在提示符 `> ` 后输入 JSON 文本,回车即可看到解析结果。
//!
//! - 空行 —— 忽略,继续等待输入
//! - `exit` / `:quit` / `Ctrl+Z` —— 退出 REPL
//! - 解析错误 —— 打到 stderr,继续等待输入(不退出)
//!
//! ## 例子
//!
//! ```text
//! JSON REPL  (Ctrl+Z 退出,输入 exit 或 :quit 也能退)
//! > {"name": "lccxxo", "age": 30}
//! Object {
//!     "name": String("lccxxo"),
//!     "age": Number(30.0),
//! }
//! > [1, 2,]
//! line 1, col 1: trailing comma in array
//! >
//! ```
//!
//! ## 为什么不用 stdin 一次性读
//!
//! 之前是从 stdin 一次性读到 EOF,适合管道 / 文件场景。REPL 更适合人机交互,
//! 而且能多输几次试错、不用每次重启 `cargo run`。

use std::io::{self, BufRead, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    println!("JSON REPL  (Ctrl+Z 退出,输入 exit 或 :quit 也能退)");

    // 拿到 stdin 的带缓冲句柄,行式读取
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut input = String::new();

    loop {
        // 1. 打提示符。print! 不换行,所以要 flush 才看得到
        print!("> ");
        if stdout.flush().is_err() {
            // stdout 写不进去一般意味着管道断了,直接收摊
            return ExitCode::SUCCESS;
        }

        // 2. 读一行。read_line 返回 Ok(0) 表示 EOF(Ctrl+Z / 管道关闭)
        input.clear();
        let n = match stdin.lock().read_line(&mut input) {
            Ok(n) => n,
            Err(e) => {
                eprintln!("error reading stdin: {}", e);
                return ExitCode::from(1);
            }
        };

        // 3. EOF 退出
        if n == 0 {
            println!();
            return ExitCode::SUCCESS;
        }

        // 4. trim 后再看:空行跳过,exit / :quit 退出
        let line = input.trim();
        if line.is_empty() {
            continue;
        }
        if line == "exit" || line == ":quit" || line == ":q" {
            return ExitCode::SUCCESS;
        }

        // 5. 调库入口 parse
        match json_parser::parse(line) {
            Ok(value) => println!("{:#?}", value),
            Err(e) => eprintln!("{}", e),
            // 注意:解析失败不 return,让 REPL 继续,方便试错
        }
    }
}