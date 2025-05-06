use bf::interpreter::*;
use std::hint::black_box;
use std::io::{Write, stdout};

fn main() {
    let program = b"++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>?.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";

    let mut inter = Interpreter::new().with_str(program).unwrap();

    let input = b"\0";

    loop {
        match inter.poll() {
            Output::Output(c) => {
                print!("{}", c as char);
                stdout().flush().unwrap();
            }
            Output::Input => {
                inter.input(input);
            }
            Output::End => {
                break;
            }
        }
    }
}
