use rox::compiler::compile;
use rox::scanner::Scanner;
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
    run(contents);
}

fn run(source: String) {
    let scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();
    if let Err(errors) = tokens {
        // FIXME: proper error handling
        let str_errors = errors.iter().map(|err| format!("{:?}", err));
        println!("{}", str_errors.collect::<Vec<String>>().join("\n"));
        exit(65);
    }
    let chunk = compile(tokens.expect("Expected successful scan"));
    if let Err(err) = chunk {
        println!("{}", err);
        exit(65);
    }
    let mut vm = VM::new();
    match vm.interpret(chunk.expect("Expected successful compilation")) {
        Err(InterpretError::InterpretRuntimeError) => {
            // TODO: proper error message
            println!("Runtime error");
            exit(70);
        }
        Ok(()) => {}
    }
}
