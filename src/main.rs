use rox::vm::InterpretError;
use rox::vm::VM;
use std::env;
use std::fs;
use std::process::exit;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        println!("Usage: rox [script]");
        exit(64);
    } else if args.len() == 2 {
        run_file(&args[1]);
    } else {
        repl();
    }
}

fn repl() {
    todo!()
}

fn run_file(filename: &str) {
    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");
    let mut vm = VM::new();
    match vm.interpret(contents) {
        Err(InterpretError::InterpretCompileError(err)) => {
            println!("{}", err);
            exit(65);
        }
        Err(InterpretError::InterpretRuntimeError) => {
            // TODO: proper error message
            println!("Runtime error");
            exit(70);
        }
        Ok(()) => {}
    }
}
