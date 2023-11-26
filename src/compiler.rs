use crate::ast::{Binary, Declaration, Expr, InnerStatement, Literal, Program, Statement, Unary};
use crate::chunk::{Chunk, OpCode};
use crate::token::TokenType;
use crate::value::Value;

pub struct Compiler<'a> {
    current_line: u16,
    current_chunk: &'a mut Chunk,
}

impl<'a> Compiler<'a> {
    pub fn new(chunk: &'a mut Chunk) -> Self {
        Compiler {
            current_line: 0,
            current_chunk: chunk,
        }
    }

    pub fn run(&mut self, program_ast: Program) -> Result<(), String> {
        for decl in program_ast.declarations {
            self.declaration(decl)?;
        }
        #[cfg(feature = "debugPrintCode")]
        {
            self.current_chunk.disassemble("code");
        }
        Ok(())
    }

    fn declaration(&mut self, decl: Declaration) -> Result<(), String> {
        match decl {
            Declaration::FunDecl(_) => todo!(),
            Declaration::LetDecl(_) => todo!(),
            Declaration::Statement(statement) => self.statement(statement),
        }
    }

    fn statement(&mut self, statement: Statement) -> Result<(), String> {
        self.current_line = statement.token.line;
        match statement.statement {
            InnerStatement::ExprStmt(expr) => self.expression(expr),
            InnerStatement::IfStmt(_) => todo!(),
            InnerStatement::PrintStmt(_) => todo!(),
            InnerStatement::ReturnStmt(_) => self.return_statement(),
            InnerStatement::WhileStmt(_) => todo!(),
            InnerStatement::Block(_) => todo!(),
        }
    }

    fn expression(&mut self, expr: Expr) -> Result<(), String> {
        match expr {
            Expr::Literal(literal) => self.literal(literal),
            Expr::Unary(op) => self.unary(op),
            Expr::Binary(op) => self.binary(op),
            Expr::Call(_) => todo!(),
            Expr::Grouping(group) => self.expression(*group.expression),
            Expr::Variable(_) => todo!(),
            Expr::Assignment(_) => todo!(),
            Expr::Logical(_) => todo!(),
            Expr::Get(_) => todo!(),
            Expr::Set(_) => todo!(),
        }
    }

    fn literal(&mut self, literal: Literal) -> Result<(), String> {
        match literal {
            Literal::Number(number) => {
                let constant = self.current_chunk.add_constant(Value::Number(number));
                self.emit_byte(OpCode::OpConstant as u8);
                self.emit_byte(constant);
            }
            Literal::Str(_) => todo!(),
            Literal::True => todo!(),
            Literal::False => todo!(),
            Literal::Null => todo!(),
        }
        Ok(())
    }

    fn unary(&mut self, op: Unary) -> Result<(), String> {
        match op.operator.typ {
            TokenType::Minus => {
                self.expression(*op.right)?;
                self.emit_byte(OpCode::OpNegate as u8);
                Ok(())
            }
            _ => Err(format!(
                "Unexpected unary operator: {} at line {}",
                op.operator.lexeme, op.operator.line
            )),
        }
    }

    fn binary(&mut self, op: Binary) -> Result<(), String> {
        self.expression(*op.left)?;
        self.expression(*op.right)?;
        let op_code = match op.operator.typ {
            TokenType::Minus => OpCode::OpSubtract,
            TokenType::Plus => OpCode::OpAdd,
            TokenType::Slash => OpCode::OpDivide,
            TokenType::Star => OpCode::OpMultiply,
            _ => Err(format!(
                "Unexpected binary operator: {} at line {}",
                op.operator.lexeme, op.operator.line
            ))?,
        };
        self.emit_byte(op_code as u8);
        Ok(())
    }

    fn return_statement(&mut self) -> Result<(), String> {
        self.emit_byte(OpCode::OpReturn as u8);
        Ok(())
    }

    fn emit_byte(&mut self, byte: u8) {
        self.current_chunk.write(byte, self.current_line as usize);
    }
}
