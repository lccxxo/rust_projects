//! 统一错误类型 + 退出码映射
//！
//！
//

use std::process::ExitCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UtError {
    /// IO 错误,带上下文(哪个文件 / 哪个操作)
    /// 例:cat 打开 a.txt 失败 → "cannot open 'a.txt': No such file or directory"
    #[error("{context}: {source}")]
    Io {
        context: String,
        #[source]
        source: std::io::Error,
    },

    /// 参数解析错误(unknown flag / 缺参数)
    /// 例:cat -z → "invalid argument: unknown flag: -z"
    #[error("invalid argument: {0}")]
    Parse(String),

    /// UTF-8 解码错误(wc -m / grep 偶尔需要)
    /// 例:wc -m 读到非法 UTF-8 字节 → "utf-8 decode error: ..."
    #[error("utf-8 decode error: {0}")]
    Utf8(#[from] std::str::Utf8Error),
}

impl UtError {
    pub fn exit_code(&self) -> u8 {
        match self {
            UtError::Parse(_) => 2,
            UtError::Io { .. } | UtError::Utf8(_) => 1,
        }
    }
}

impl From<std::io::Error> for UtError {
    /// 让 `File::open(...)?` 能自动转 UtError(简化 main 里的错误处理)
    fn from(e: std::io::Error) -> Self {
        UtError::Io {
            context: "I/O error".into(),
            source: e,
        }
    }
}

/// 统一错误出口:打印到 stderr + 返回非零 ExitCode
///
/// 为什么集中在一个函数:
/// - main() 只调一次 die(),不用每处错误都写 eprintln! + ExitCode::from
/// - 退出码逻辑只在这里写一次,后面加新 UtError variant 不用改 main
pub fn die(e: UtError) -> ExitCode {
    eprintln!("ut: {}", e);
    ExitCode::from(e.exit_code())
}
