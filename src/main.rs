use std::env;
use std::process;

fn main() {
    match make_mate::run(env::args().skip(1).collect()) {
        Ok(code) => process::exit(code),
        Err(error) => {
            eprintln!("mate: {error}");
            process::exit(2);
        }
    }
}
