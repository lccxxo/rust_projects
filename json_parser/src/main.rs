use std::io::Read;

fn main() {
    let mut input = String::new();
    if let Err(e) = std::io::stdin().read_to_string(&mut input) {
        eprintln!("error reading stdin: {}", e);
        std::process::exit(1);
    }

    match json_parser::parse(&input) {
        Ok(value) => println!("{:#?}", value),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}
