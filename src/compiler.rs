use crate::ast::{
    Assignment, Binary, Declaration, DeclarationWithLineNo, Expr, FunDecl, IfStmt, LetDecl,
    Literal, Logical, Program, Statement, Unary, Variable, WhileStmt,
};
use crate::chunk::{Chunk, OpCode};
use crate::token::{Token, TokenType};
use crate::value::{Function, Value};

pub struct Compiler {
    current_line: u16,
    pub function: Function,
    function_type: FunctionType,
    locals: Vec<Local>,
    scope_depth: u8,
}

struct Local {
    name: Token,
    depth: u8,
}

// useful to distinguish real functions from implicit top level function
pub enum FunctionType {
    Function(String),
    Script,
}

impl Compiler {
    pub fn new(function_type: FunctionType) -> Self {
        let mut function = Function::new();
        function.name = match &function_type {
            FunctionType::Function(name) => Some(name.clone()),
            FunctionType::Script => Some(String::from("<script>")),
        };
        Compiler {
            current_line: 0,
            function,
            function_type,
            // TODO: initialize locals like in page 438
            locals: Vec::new(),
            scope_depth: 0,
        }
    }

    pub fn run(&mut self, program_ast: Program) -> Result<(), String> {
        for decl in program_ast.declarations {
            self.declaration(decl)?;
        }
        self.emit_byte(OpCode::OpEof as u8);
        #[cfg(feature = "debugPrintCode")]
        {
            let chunk_name = match &self.function.name {
                Some(func_name) => func_name.clone(),
                None => String::from("<script>"),
            };
            self.current_chunk().disassemble(&chunk_name);
        }
        Ok(())
    }

    fn declaration(&mut self, decl: DeclarationWithLineNo) -> Result<(), String> {
        let inner_decl = decl.decl;
        self.current_line = decl.lineno;
        match inner_decl {
            Declaration::FunDecl(decl) => self.fun_decl(decl),
            Declaration::LetDecl(decl) => self.let_decl(decl),
            Declaration::Statement(statement) => self.statement(statement),
        }
    }

    fn statement(&mut self, statement: Statement) -> Result<(), String> {
        match statement {
            Statement::ExprStmt(expr) => {
                // popping unused expression from the stack
                self.expression(expr)?;
                self.emit_byte(OpCode::OpPop as u8);
                Ok(())
            }
            Statement::IfStmt(if_stmt) => self.if_statement(if_stmt),
            Statement::PrintStmt(expr) => self.print_statement(expr),
            Statement::ReturnStmt(_) => self.return_statement(),
            Statement::WhileStmt(while_stmt) => self.while_statement(while_stmt),
            Statement::Block(declarations) => self.block(declarations),
        }
    }

    /// compiles code of the form:
    /// addr: JUMP_IF_FALSE to addr2 + 3
    /// jump_operand 1
    /// jump_operand 2
    /// ... then code
    /// addr2: JUMP to addr3
    /// jump_operand 1
    /// jump_operand 2
    /// ... else code
    /// addr3
    fn if_statement(&mut self, if_stmt: IfStmt) -> Result<(), String> {
        self.expression(if_stmt.condition)?;
        let then_jump = self.emit_jump(OpCode::OpJumpIfFalse as u8);
        // clean up condition value
        self.emit_byte(OpCode::OpPop as u8);
        self.statement(*if_stmt.then_branch)?;
        let else_jump = self.emit_jump(OpCode::OpJump as u8);

        self.patch_jump(then_jump);
        // clean up condition value
        self.emit_byte(OpCode::OpPop as u8);
        if let Some(else_branch) = if_stmt.else_branch {
            self.statement(*else_branch)?;
        }
        self.patch_jump(else_jump);
        Ok(())
    }

    fn expression(&mut self, expr: Expr) -> Result<(), String> {
        match expr {
            Expr::Literal(literal) => self.literal(literal),
            Expr::Unary(op) => self.unary(op),
            Expr::Binary(op) => self.binary(op),
            Expr::Call(_) => todo!(),
            Expr::Grouping(group) => self.expression(*group.expression),
            Expr::Variable(variable) => self.variable(variable),
            Expr::Assignment(assignment) => self.assignment(assignment),
            Expr::Logical(logical) => self.logical(logical),
            Expr::Get(_) => todo!(),
            Expr::Set(_) => Err(self.report_error("Set not supported".to_string())),
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
        match op.operator.typ {
            TokenType::And => {
                let jump_offset = self.emit_jump(OpCode::OpJumpIfFalse as u8);
                self.emit_byte(OpCode::OpPop as u8);
                self.expression(*op.right)?;
                self.patch_jump(jump_offset)
            }
            TokenType::Or => {
                let jump_offset = self.emit_jump(OpCode::OpJumpIfTrue as u8);
                self.emit_byte(OpCode::OpPop as u8);
                self.expression(*op.right)?;
                self.patch_jump(jump_offset);
            }
            _ => Err(format!(
                "Unexpected logical operator: {} at line {}",
                op.operator.lexeme, op.operator.line
            ))?,
        }
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

    fn while_statement(&mut self, while_stmt: WhileStmt) -> Result<(), String> {
        let loop_start = self.current_chunk().count();
        self.expression(while_stmt.condition)?;
        let jump_offset = self.emit_jump(OpCode::OpJumpIfFalse as u8);
        self.emit_byte(OpCode::OpPop as u8);
        self.statement(*(while_stmt.body))?;
        self.emit_loop(loop_start);
        self.patch_jump(jump_offset);
        self.emit_byte(OpCode::OpPop as u8);
        Ok(())
    }

    fn let_decl(&mut self, decl: LetDecl) -> Result<(), String> {
        // FIXME: allow absence of initializer
        let initializer = decl
            .initializer
            .expect("Expected initializer to let declaration");
        self.expression(initializer)?;
        if self.scope_depth > 0 {
            self.add_local(decl.identifier)?;
            return Ok(());
        }
        let constant = self.make_constant(Value::Str(decl.identifier.lexeme));
        self.emit_bytes(OpCode::OpDefineGlobal as u8, constant);
        Ok(())
    }

    fn fun_decl(&mut self, decl: FunDecl) -> Result<(), String> {
        let func_name = &decl.name.lexeme;
        let mut compiler = Compiler::new(FunctionType::Function(func_name.clone()));
        compiler.run(Program {
            declarations: decl.body,
        })?;
        self.emit_constant(Value::Function(compiler.function));
        if self.scope_depth > 0 {
            self.add_local(decl.name)?;
            return Ok(());
        }
        let constant = self.make_constant(Value::Str(func_name.clone()));
        self.emit_bytes(OpCode::OpDefineGlobal as u8, constant);
        Ok(())
    }

    fn variable(&mut self, variable: Variable) -> Result<(), String> {
        let local_index = self.resolve_local(&variable.name);
        match local_index {
            Some(index) => self.emit_bytes(OpCode::OpGetLocal as u8, index.try_into().unwrap()),
            None => {
                let constant = self.make_constant(Value::Str(variable.name.lexeme));
                self.emit_bytes(OpCode::OpGetGlobal as u8, constant);
            }
        };
        Ok(())
    }

    fn assignment(&mut self, assignment: Assignment) -> Result<(), String> {
        self.expression(*assignment.value)?;
        match self.resolve_local(&assignment.name) {
            Some(local_index) => {
                self.emit_bytes(OpCode::OpSetLocal as u8, local_index.try_into().unwrap());
            }
            None => {
                let constant = self.make_constant(Value::Str(assignment.name.lexeme));
                self.emit_bytes(OpCode::OpSetGlobal as u8, constant);
            }
        }
        Ok(())
    }

    fn block(&mut self, declarations: Vec<DeclarationWithLineNo>) -> Result<(), String> {
        self.scope_depth += 1;
        // FIXME: line number are not tracked inside blocks
        for decl in declarations {
            self.declaration(decl)?;
        }
        self.scope_depth -= 1;
        let mut nb_vars_to_pop: u8 = 0;
        while self.locals.len() > 0 && self.locals[self.locals.len() - 1].depth > self.scope_depth {
            self.locals.pop();
            nb_vars_to_pop += 1;
        }
        if nb_vars_to_pop == 1 {
            self.emit_byte(OpCode::OpPop as u8);
        } else if nb_vars_to_pop > 1 {
            self.emit_bytes(OpCode::OpPopN as u8, nb_vars_to_pop);
        }
        Ok(())
    }

    fn add_local(&mut self, name: Token) -> Result<(), String> {
        for index in (0..self.locals.len()).rev() {
            let local = &self.locals[index];
            if local.depth < self.scope_depth {
                break;
            }
            if self.identifiers_equal(&local.name, &name) {
                return Err(self.report_error(format!(
                    "Already a variable with the name {} in this scope",
                    name.lexeme
                )));
            }
        }
        self.locals.push(Local {
            name,
            depth: self.scope_depth,
        });
        Ok(())
    }

    /// return the local index on the stack
    fn resolve_local(&self, name: &Token) -> Option<usize> {
        for index in (0..self.locals.len()).rev() {
            let local = &self.locals[index];
            if self.identifiers_equal(&local.name, &name) {
                return Some(index);
            }
        }
        None
    }

    fn current_chunk(&mut self) -> &mut Chunk {
        &mut self.function.chunk
    }

    fn identifiers_equal(&self, first: &Token, second: &Token) -> bool {
        first.lexeme == second.lexeme
    }

    fn report_error(&self, message: String) -> String {
        format!(
            "Compilation error: {}\nat line {}",
            message, self.current_line
        )
    }

    fn emit_byte(&mut self, byte: u8) {
        let lineno = self.current_line;
        self.current_chunk().write(byte, lineno as usize);
    }

    fn emit_bytes(&mut self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn emit_constant(&mut self, value: Value) {
        let constant = self.make_constant(value);
        self.emit_bytes(OpCode::OpConstant as u8, constant);
    }

    fn emit_jump(&mut self, instruction: u8) -> usize {
        self.emit_byte(instruction);
        self.emit_byte(0);
        self.emit_byte(0);
        self.current_chunk().count() - 2
    }

    /// fill jump operand specified at offset, namely jump to current location
    fn patch_jump(&mut self, offset: usize) {
        let jump = self.current_chunk().count() - offset - 2;
        self.current_chunk()
            .replace_at((jump >> 8 & 0xff).try_into().unwrap(), offset);
        self.current_chunk()
            .replace_at((jump & 0xff).try_into().unwrap(), offset + 1);
    }

    fn emit_loop(&mut self, start_loop: usize) -> usize {
        self.emit_byte(OpCode::OpLoop as u8);
        let loop_size = self.current_chunk().count() - start_loop + 2;
        self.emit_byte((loop_size >> 8 & 0xff).try_into().unwrap());
        self.emit_byte((loop_size & 0xff).try_into().unwrap());
        self.current_chunk().count() - 2
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        self.current_chunk().add_constant(value)
    }
}
