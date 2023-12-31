use crate::ast::{
    Binary, Declaration, Expr, LetDecl, Literal, Logical, Program, Statement, Unary, Variable,
};
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
        for (lineno, decl) in program_ast.declarations {
            self.current_line = lineno;
            self.declaration(decl)?;
        }
        self.emit_byte(OpCode::OpEof as u8);
        #[cfg(feature = "debugPrintCode")]
        {
            self.current_chunk.disassemble("code");
        }
        Ok(())
    }

    fn declaration(&mut self, decl: Declaration) -> Result<(), String> {
        match decl {
            Declaration::FunDecl(_) => todo!(),
            Declaration::LetDecl(decl) => self.let_decl(decl),
            Declaration::Statement(statement) => self.statement(statement),
        }
    }

    fn statement(&mut self, statement: Statement) -> Result<(), String> {
        match statement {
            Statement::ExprStmt(expr) => self.expression(expr),
            Statement::IfStmt(_) => todo!(),
            Statement::PrintStmt(expr) => self.print_statement(expr),
            Statement::ReturnStmt(_) => self.return_statement(),
            Statement::WhileStmt(_) => todo!(),
            Statement::Block(_) => todo!(),
        }
    }

    fn expression(&mut self, expr: Expr) -> Result<(), String> {
        match expr {
            Expr::Literal(literal) => self.literal(literal),
            Expr::Unary(op) => self.unary(op),
            Expr::Binary(op) => self.binary(op),
            Expr::Call(_) => todo!(),
            Expr::Grouping(group) => self.expression(*group.expression),
            Expr::Variable(variable) => self.variable(variable),
            Expr::Assignment(ass) => Err(format!(
                "Assignement not supported, see line {}",
                ass.name.line
            )),
            Expr::Logical(logical) => self.logical(logical),
            Expr::Get(_) => todo!(),
            Expr::Set(set) => Err(format!("Set not supported, see line {}", set.name.line)),
        }
    }

    fn literal(&mut self, literal: Literal) -> Result<(), String> {
        match literal {
            Literal::Number(number) => self.emit_constant(Value::Number(number)),
            Literal::Str(s) => self.emit_constant(Value::Str(s)),
            Literal::True => self.emit_byte(OpCode::OpTrue as u8),
            Literal::False => self.emit_byte(OpCode::OpFalse as u8),
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
            TokenType::Not => {
                self.expression(*op.right)?;
                self.emit_byte(OpCode::OpNot as u8);
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
            TokenType::EqualEqual => OpCode::OpEqualEqual,
            TokenType::BangEqual => OpCode::OpBangEqual,
            TokenType::Less => OpCode::OpLess,
            TokenType::LessEqual => OpCode::OpLessEqual,
            TokenType::Greater => OpCode::OpGreater,
            TokenType::GreaterEqual => OpCode::OpGreaterEqual,
            _ => Err(format!(
                "Unexpected binary operator: {} at line {}",
                op.operator.lexeme, op.operator.line
            ))?,
        };
        self.emit_byte(op_code as u8);
        Ok(())
    }

    fn logical(&mut self, op: Logical) -> Result<(), String> {
        self.expression(*op.left)?;
        self.expression(*op.right)?;
        let op_code = match op.operator.typ {
            TokenType::And => OpCode::OpAnd,
            TokenType::Or => OpCode::OpOr,
            _ => Err(format!(
                "Unexpected logical operator: {} at line {}",
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

    fn print_statement(&mut self, expr: Expr) -> Result<(), String> {
        self.expression(expr)?;
        self.emit_byte(OpCode::OpPrint as u8);
        Ok(())
    }

    fn let_decl(&mut self, decl: LetDecl) -> Result<(), String> {
        // FIXME: allow absence of initializer
        let initializer = decl
            .initializer
            .expect("Expected initializer to let declaration");
        self.expression(initializer)?;
        let constant = self.make_constant(Value::Str(decl.identifier.lexeme));
        self.emit_bytes(OpCode::OpDefineGlobal as u8, constant);
        Ok(())
    }

    fn variable(&mut self, variable: Variable) -> Result<(), String> {
        let constant = self.make_constant(Value::Str(variable.name.lexeme));
        self.emit_bytes(OpCode::OpGetGlobal as u8, constant);
        Ok(())
    }

    fn emit_byte(&mut self, byte: u8) {
        self.current_chunk.write(byte, self.current_line as usize);
    }

    fn emit_bytes(&mut self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn emit_constant(&mut self, value: Value) {
        let constant = self.make_constant(value);
        self.emit_bytes(OpCode::OpConstant as u8, constant);
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        self.current_chunk.add_constant(value)
    }
}
