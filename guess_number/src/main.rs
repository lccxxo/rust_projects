// 猜数字游戏
use std::io;
use rand::Rng;
use std::cmp::Ordering;

fn main() {
    let secert_number = rand::thread_rng().gen_range(1..=100);

    loop {
            let mut guess_number  = String::new();
            println!("请输入你猜的数字：");
            io::stdin().read_line(&mut guess_number ).expect("读取失败");

            let guess_number:u32 =  guess_number.trim().parse().expect("输入的内容错误");

            match guess_number.cmp(&secert_number) {
                Ordering::Less => println!("你猜的数字小了"),
                Ordering::Greater => println!("你猜的数字大了"),
                Ordering::Equal => {
                    println!("恭喜你，猜对了！");
                    break;
                }
            }
    }
}