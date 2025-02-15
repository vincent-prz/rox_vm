use rox::ast::parser::Parser;
use rox::compiler::Compiler;
use rox::compiler::FunctionType;
use rox::scanner::Scanner;
use rox::vm::RuntimeError;
use rox::vm::VM;
use std::env;
use std::fs;
use std::io;
use std::io::Write;
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
    loop {
        print!("> ");
        io::stdout()
            .flush()
            .expect("Somethig went wrong when flushing IO");
        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .expect("Something went wrong when reading the line");
        if line == "\n" {
            break;
        }
        if !line.ends_with(";\n") {
            line.insert(line.len() - 1, ';')
        }
        run(format!("print {}", line));
    }
}

fn run_file(filename: &str) {
    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");
    run(contents);
}

/// source processing pipeline
/// 1. scan
/// 2. parse
/// 3. compile to bytecode chunk
/// 4. vm execs bytecode chunk
fn run(source: String) {
    // FIXME: proper error handling
    let scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();
    if let Err(errors) = tokens {
        let str_errors = errors.iter().map(|err| format!("{:?}", err));
        println!("{}", str_errors.collect::<Vec<String>>().join("\n"));
        exit(65);
    }

    let mut parser = Parser::new(tokens.expect("Expected successful scan"));
    let program_ast = parser.parse();
    if let Err(error) = program_ast {
        println!("{:?}", error);
        exit(65);
    }

    let mut compiler = Compiler::new(FunctionType::Script);
    let compilation_result = compiler.run(program_ast.expect("Expected successful parse"));
    if let Err(err) = compilation_result {
        println!("{}", err);
        exit(65);
    }

    let mut vm = VM::new(compiler.function);
    match vm.interpret() {
        Err(RuntimeError { msg }) => {
            println!("{}", msg);
            exit(70);
        }
        Ok(()) => {}
    }
}
