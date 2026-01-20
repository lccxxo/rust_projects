use tokio::time::{sleep, Duration};
use tokio::task;

#[tokio::main]
async fn main() {
    // 启动异步任务：持续打印
    let async_task = task::spawn(async {
        let mut count = 0;
        loop {
            count += 1;
            println!("异步任务 - 计数: {}", count);
            sleep(Duration::from_millis(500)).await; // 每500ms打印一次
        }
    });

    // 主线程等待5秒
    println!("主线程等待5秒...");
    sleep(Duration::from_secs(5)).await;
    
    // 5秒后，主线程打印
    println!("=== 主线程5秒后打印 ===");
    
    // 继续让异步任务运行（或停止它）
    // 按 Ctrl+C 停止程序
    sleep(Duration::from_secs(2)).await; // 再运行2秒
    
    // 取消异步任务
    async_task.abort();
    let _ = async_task.await; // 等待任务结束
}