// 实现一个简单的计算器 两位数的加减乘除

use std::io;

fn main() {
    let mut user_type = UserTyper::new(SimpleComputer);

    loop {
        user_type.command_line();
        println!("result is {}", user_type.execute())
    }
}

trait Computer {
    fn execute(&self,expr: String) -> i32;
}

struct SimpleComputer;

impl Computer for SimpleComputer {
    fn execute(&self,expr :String) -> i32 {
        let mut num1 = String::new();
        let mut num2 = String::new();
        let mut op:Option<char>= None;

        for x in expr.trim().chars() {
            if x.is_digit(10){
                if op.is_none(){
                    num1.push(x);
                }else {
                    num2.push(x);
                }

                continue;
            }

            match x {
                '+' | '-' | '*' | '/' if op.is_none() => {op = Some(x)}
                _ if x.is_whitespace() => {continue;}
                _ => panic!("Invalid character: {}", x)
            }
        }

        if num1.is_empty() || num2.is_empty() || op.is_none() {
            panic!("Invalid expression: {}", expr);
        }

        let num1 = num1.parse::<i32>().unwrap();
        let num2 = num2.parse::<i32>().unwrap();
        let op = op.unwrap();

        match op {
            '+' => num1 + num2,
            '-' => num1 - num2,
            '*' => num1 * num2,
            '/' => num1 / num2,
            _ => unreachable!()
        }
    }
}

struct UserTyper<T: Computer> {
    computer: T,
    expr: String,
}

impl<T:Computer>  UserTyper<T> {
    fn new(computer: T) -> Self{
        Self{
            computer,
            expr: String::new()
        }
    }

    fn command_line(&mut self){
        self.expr.clear();
        println!("please enter an expression");
        io::stdin().read_line(&mut self.expr).unwrap();
    }

    // 进行计算
    fn execute(&self) -> i32 {
        self.computer.execute(self.expr.clone())
    }
}