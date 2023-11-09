use crate::scanner::Scanner;
use crate::chunk::Chunk;

// FIXME: proper error handling
pub fn compile(source: String) -> Result<Chunk, String> {
    let scanner = Scanner::new(source);
    match scanner.scan_tokens() {
        Ok(_) => todo!(),
        Err(errors) => {
            let str_errors = errors.iter().map(|err| format!("{:?}", err));
            return Err(str_errors.collect::<Vec<String>>().join("\n"));
        }
    }
}
