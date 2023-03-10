use std::rc::Rc;

use super::{
    error::ErrorReporter,
    expr::{Expr, Function},
    token::{Token, TokenKind},
    value::Cell,
};
use crate::walk_tree::stmt::Stmt;

pub struct Parser<'a> {
    tokens: Vec<Token>,
    current: usize,
    error_reporter: &'a ErrorReporter,
}

impl<'a> Parser<'a> {
    const EQUALITY_OPERATORS: [TokenKind; 2] = [TokenKind::BangEqual, TokenKind::EqualEqual];
    const COMPARISON_OPERATORS: [TokenKind; 4] = [
        TokenKind::Greater,
        TokenKind::GreaterEqual,
        TokenKind::Less,
        TokenKind::LessEqual,
    ];
    const TERM_OPERATORS: [TokenKind; 2] = [TokenKind::Minus, TokenKind::Plus];
    const FACTOR_OPERATORS: [TokenKind; 2] = [TokenKind::Slash, TokenKind::Star];
    const UNARY_OPERATORS: [TokenKind; 2] = [TokenKind::Bang, TokenKind::Minus];
    const SYNCHRONIZE: [TokenKind; 7] = [
        TokenKind::Class,
        TokenKind::Fun,
        TokenKind::Var,
        TokenKind::For,
        TokenKind::If,
        TokenKind::While,
        TokenKind::Return,
    ];

    pub fn new(tokens: Vec<Token>, error_reporter: &'a ErrorReporter) -> Self {
        Self {
            tokens,
            current: 0,
            error_reporter,
        }
    }

    pub fn parse(&mut self) -> Option<Box<[Box<Stmt>]>> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration()?)
        }
        Some(Box::from(statements))
    }

    pub fn expression(&mut self) -> Option<Box<Expr>> {
        self.assigment()
    }

    fn declaration(&mut self) -> Option<Box<Stmt>> {
        let result = self.try_declaration();
        if matches!(result, None) {
            self.synchronize();
        }
        result
    }

    fn try_declaration(&mut self) -> Option<Box<Stmt>> {
        if self.match_one(&TokenKind::Class) {
            self.class_declaration()
        } else if self.match_one(&TokenKind::Fun) {
            self.function_declaration()
        } else if self.match_one(&TokenKind::Var) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    fn statement(&mut self) -> Option<Box<Stmt>> {
        if self.match_one(&TokenKind::For) {
            self.for_statement()
        } else if self.match_one(&TokenKind::If) {
            self.if_statement()
        } else if self.match_one(&TokenKind::Return) {
            self.return_stmt()
        } else if self.match_one(&TokenKind::While) {
            self.while_statement()
        } else if self.match_one(&TokenKind::LeftBrace) {
            self.block()
        } else {
            self.expression_statement()
        }
    }

    fn for_statement(&mut self) -> Option<Box<Stmt>> {
        self.consume(&TokenKind::LeftParen, || "Expect '(' after 'for'.".into())?;
        let initializer = if self.match_one(&TokenKind::Semicolon) {
            None
        } else if self.match_one(&TokenKind::Var) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };
        let condition = if !self.check(&TokenKind::Semicolon) {
            self.expression()?
        } else {
            Box::new(Expr::from(true))
        };
        self.consume(&TokenKind::Semicolon, || {
            "Expect ';' after loop condition.".into()
        })?;
        let increment = if !self.check(&TokenKind::RightParen) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(&TokenKind::RightParen, || {
            "Expect ')' after for clauses.".into()
        })?;

        let mut body = self.statement()?;
        if let Some(increment) = increment {
            body = Box::new(Stmt::Block(Rc::new([
                body,
                Box::new(Stmt::Expr(increment)),
            ])));
        }
        body = Box::new(Stmt::While { condition, body });
        if let Some(initializer) = initializer {
            body = Box::new(Stmt::Block(Rc::new([initializer, body])));
        }

        Some(body)
    }

    fn if_statement(&mut self) -> Option<Box<Stmt>> {
        self.consume(&TokenKind::LeftParen, || "Expect '(' after 'if'.".into())?;
        let condition = self.expression()?;
        self.consume(&TokenKind::RightParen, || {
            "Expect ')' after if condition.".into()
        })?;

        let then_branch = self.statement()?;
        let else_branch = if self.match_one(&TokenKind::Else) {
            Some(self.statement()?)
        } else {
            None
        };
        Some(Box::new(Stmt::If {
            condition,
            then_branch,
            else_branch,
        }))
    }

    fn return_stmt(&mut self) -> Option<Box<Stmt>> {
        let keyword = self.previous().to_owned();
        let value = if !self.check(&TokenKind::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(&TokenKind::Semicolon, || {
            "Expect ';' after return value.".to_string()
        })?;
        Some(Box::new(Stmt::Return {
            keyword,
            expr: value,
        }))
    }

    fn var_declaration(&mut self) -> Option<Box<Stmt>> {
        let name = self
            .consume(&TokenKind::Identifier, || {
                "Expect variable name.".to_string()
            })?
            .to_owned();
        let initializer = if self.match_one(&TokenKind::Equal) {
            self.expression()
        } else {
            None
        };
        self.consume(&TokenKind::Semicolon, || {
            "Expect ';' after variable declaration.".to_string()
        })?;
        Some(Box::new(Stmt::VarDeclaration { name, initializer }))
    }

    fn while_statement(&mut self) -> Option<Box<Stmt>> {
        self.consume(&TokenKind::LeftParen, || "Expect '(' after 'while'.".into())?;
        let condition = self.expression()?;
        self.consume(&TokenKind::RightParen, || {
            "Expect ')' after condition.".into()
        })?;
        let body = self.statement()?;
        Some(Box::new(Stmt::While { condition, body }))
    }

    fn expression_statement(&mut self) -> Option<Box<Stmt>> {
        let expr = self.expression()?;
        self.consume(&TokenKind::Semicolon, || {
            "Expect ';' after expression.".into()
        })?;
        Some(Box::new(Stmt::Expr(expr)))
    }

    fn class_declaration(&mut self) -> Option<Box<Stmt>> {
        let name = self
            .consume(&TokenKind::Identifier, || "Expect class name.".into())?
            .to_owned();
        let superclass = if self.match_one(&TokenKind::Less) {
            self.consume(&TokenKind::Identifier, || "Expect superclass name.".into())?;
            Some(Box::new(Expr::Variable(self.previous().to_owned())))
        } else {
            None
        };
        self.consume(&TokenKind::LeftBrace, || {
            "Expect '{' before class body.".into()
        })?;
        let mut methods = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            methods.push(self.function("method")?);
        }
        self.consume(&TokenKind::RightBrace, || {
            "Expect '}' after class body.".into()
        })?;
        Some(Box::new(Stmt::Class {
            name,
            superclass,
            methods: Box::from(methods),
        }))
    }

    fn function_declaration(&mut self) -> Option<Box<Stmt>> {
        let function = self.function("function")?;
        let declaration = Stmt::VarDeclaration {
            name: function.name()?.clone(),
            initializer: Some(Box::new(Expr::from(function))),
        };
        Some(Box::new(declaration))
    }

    fn function(&mut self, kind: &str) -> Option<Function> {
        let name = self
            .consume(&TokenKind::Identifier, || format!("Expect {kind} name."))?
            .clone();
        self.function_literal(kind, Some(name))
    }

    fn block(&mut self) -> Option<Box<Stmt>> {
        let stmts = self.stmt_vec()?;
        Some(Box::new(Stmt::Block(stmts)))
    }

    fn stmt_vec(&mut self) -> Option<Rc<[Box<Stmt>]>> {
        let mut statements = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        self.consume(&TokenKind::RightBrace, || "Expect '}' after block.".into());
        Some(Rc::from(statements))
    }

    fn assigment(&mut self) -> Option<Box<Expr>> {
        let expr = self.ternary()?;
        if self.match_one(&TokenKind::Equal) {
            let equals = self.previous().to_owned();
            let value = self.assigment()?;
            match *expr {
                Expr::Variable(name) => Some(Box::new(Expr::Assignment { name, value })),
                Expr::Get { object, name } => Some(Box::new(Expr::Set {
                    object,
                    name,
                    value,
                })),
                _ => self.error(&equals, "Invalid assignment target."),
            }
        } else {
            Some(expr)
        }
    }

    fn ternary(&mut self) -> Option<Box<Expr>> {
        let expr = self.or()?;
        if self.match_one(&TokenKind::QuestionMark) {
            let then_expr = self.expression()?;
            self.consume(&TokenKind::Colon, || "Expect ':'.".into());
            let else_expr = self.expression()?;
            Some(Box::new(Expr::Ternary {
                condition: expr,
                then_expr,
                else_expr,
            }))
        } else {
            Some(expr)
        }
    }

    fn or(&mut self) -> Option<Box<Expr>> {
        self.logical(&TokenKind::Or, Self::and)
    }

    fn and(&mut self) -> Option<Box<Expr>> {
        self.logical(&TokenKind::And, Self::equality)
    }

    fn logical<F>(&mut self, token_kind: &TokenKind, mut operand: F) -> Option<Box<Expr>>
    where
        F: FnMut(&mut Self) -> Option<Box<Expr>>,
    {
        let mut expr = operand(self)?;

        while self.match_one(token_kind) {
            let operator = self.previous().to_owned();
            let right = operand(self)?;
            expr = Box::new(Expr::Logical {
                left: expr,
                operator,
                right,
            });
        }
        Some(expr)
    }

    fn equality(&mut self) -> Option<Box<Expr>> {
        self.binary(&Self::EQUALITY_OPERATORS, Self::comparison)
    }

    fn comparison(&mut self) -> Option<Box<Expr>> {
        self.binary(&Self::COMPARISON_OPERATORS, Self::term)
    }

    fn term(&mut self) -> Option<Box<Expr>> {
        self.binary(&Self::TERM_OPERATORS, Self::factor)
    }

    fn factor(&mut self) -> Option<Box<Expr>> {
        self.binary(&Self::FACTOR_OPERATORS, Self::unary)
    }

    fn binary<F>(&mut self, operators: &[TokenKind], mut operand: F) -> Option<Box<Expr>>
    where
        F: FnMut(&mut Self) -> Option<Box<Expr>>,
    {
        let mut expr = operand(self)?;
        while self.match_many(operators) {
            expr = Box::new(Expr::binary(
                expr,
                self.previous().to_owned(),
                operand(self)?,
            ));
        }
        Some(expr)
    }

    fn unary(&mut self) -> Option<Box<Expr>> {
        if self.match_many(&Self::UNARY_OPERATORS) {
            let operator = self.previous().to_owned();
            let right = self.unary()?;
            Some(Box::new(Expr::Unary {
                operator,
                operand: right,
            }))
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> Option<Box<Expr>> {
        let mut expr = self.primary()?;
        loop {
            if self.match_one(&TokenKind::LeftParen) {
                expr = self.finish_call(expr)?;
            } else if self.match_one(&TokenKind::Dot) {
                let name = self
                    .consume(&TokenKind::Identifier, || {
                        "Expect property name after '.'.".into()
                    })?
                    .to_owned();
                expr = Box::new(Expr::Get { object: expr, name });
            } else {
                break;
            }
        }
        Some(expr)
    }

    fn finish_call(&mut self, callee: Box<Expr>) -> Option<Box<Expr>> {
        let mut arguments = Vec::new();
        if !self.check(&TokenKind::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    self.error::<()>(self.peek(), "Can't have more than 255 arguments.");
                }
                arguments.push(self.expression()?);
                if !self.match_one(&TokenKind::Comma) {
                    break;
                }
            }
        }
        let paren = self
            .consume(&TokenKind::RightParen, || {
                "Expect ')' after arguments.".into()
            })?
            .to_owned();

        Some(Box::new(Expr::Call {
            callee,
            paren,
            arguments: Box::from(arguments),
        }))
    }

    fn primary(&mut self) -> Option<Box<Expr>> {
        let expr = if self.match_one(&TokenKind::False) {
            Expr::from(false)
        } else if self.match_one(&TokenKind::True) {
            Expr::from(true)
        } else if self.match_one(&TokenKind::Nil) {
            Expr::from(())
        } else if let Some(literal) = self.literal() {
            literal
        } else if self.match_one(&TokenKind::Super) {
            let keyword = self.previous().to_owned();
            self.consume(&TokenKind::Dot, || "Expect '.' after 'super'.".into())?;
            let method = self
                .consume(&TokenKind::Identifier, || {
                    "Expect superclass method name.".into()
                })?
                .to_owned();
            Expr::Super { keyword, method }
        } else if self.match_one(&TokenKind::This) {
            Expr::This {
                keyword: self.previous().to_owned(),
            }
        } else if self.match_one(&TokenKind::Identifier) {
            Expr::Variable(self.previous().to_owned())
        } else if self.match_one(&TokenKind::Fun) {
            self.anonymous_function()?
        } else if self.match_one(&TokenKind::LeftParen) {
            let expr = self.expression()?;
            self.consume(&TokenKind::RightParen, || {
                "Expect ')' after expression.".into()
            })?;
            Expr::Grouping(expr)
        } else {
            self.error(self.peek(), "Expect expression")?
        };
        Some(Box::new(expr))
    }

    fn literal(&mut self) -> Option<Expr> {
        let expr = if self.is_at_end() {
            None
        } else if let TokenKind::Number(number) = self.peek().kind {
            Some(Expr::from(number))
        } else if let TokenKind::String(string) = &self.peek().kind {
            Some(Expr::Literal(Cell::from(Rc::clone(string))))
        } else {
            None
        };
        if expr.is_some() {
            self.advance();
        }
        expr
    }

    fn anonymous_function(&mut self) -> Option<Expr> {
        self.function_literal("function", None).map(Expr::from)
    }

    fn function_literal(&mut self, kind: &str, name: Option<Token>) -> Option<Function> {
        self.consume(&TokenKind::LeftParen, || {
            format!("Expect '(' after {kind} name.")
        })?;
        let mut parameters = Vec::new();
        if !self.check(&TokenKind::RightParen) {
            loop {
                if parameters.len() >= 255 {
                    self.error::<()>(self.peek(), "Can't have more than 255 parameters.");
                }

                parameters.push(
                    self.consume(&TokenKind::Identifier, || {
                        "Expect parameter name.".to_string()
                    })?
                    .to_owned(),
                );
                if !self.match_one(&TokenKind::Comma) {
                    break;
                }
            }
        }
        self.consume(&TokenKind::RightParen, || {
            "Expect ')' after parameters.".to_string()
        })?;
        self.consume(&TokenKind::LeftBrace, || {
            format!("Expect '{{' before {kind} body.")
        })?;
        let body = self.stmt_vec()?;
        Some(Function::new(name, Rc::from(parameters), body))
    }

    fn match_many(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds {
            if self.check(kind) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn match_one(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume<M>(&mut self, kind: &TokenKind, message: M) -> Option<&Token>
    where
        M: Fn() -> String,
    {
        if self.check(kind) {
            Some(self.advance())
        } else {
            self.error(self.peek(), &message())
        }
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            false
        } else {
            &self.peek().kind == kind
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn error<T>(&self, token: &Token, message: &str) -> Option<T> {
        self.error_reporter.token_error(token, message);
        None
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().kind == TokenKind::Semicolon {
                return;
            }
            if Self::SYNCHRONIZE.contains(&self.peek().kind) {
                return;
            }
            self.advance();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::walk_tree::{error::ErrorReporter, scanner::Scanner};

    use super::*;

    #[test]
    fn parsing_literals_works() {
        assert_eq!(test_parse_expr("2").unwrap().as_ref(), &Expr::from(2.0));
        assert_eq!(test_parse_expr("true").unwrap().as_ref(), &Expr::from(true));
        assert_eq!(
            test_parse_expr("false").unwrap().as_ref(),
            &Expr::from(false)
        );
        assert_eq!(test_parse_expr("nil").unwrap().as_ref(), &Expr::from(()));
        assert_eq!(
            test_parse_expr("\"abc\"").unwrap().as_ref(),
            &Expr::from(Rc::from("abc"))
        );
    }

    #[test]
    fn parsing_expressions_works() {
        assert_eq!(
            test_parse_expr("2+2").unwrap().as_ref(),
            &Expr::binary(
                Box::new(Expr::from(2.0)),
                Token::new(TokenKind::Plus, "+".into(), 1),
                Box::new(Expr::from(2.0))
            )
        );
        assert_eq!(
            test_parse_expr("1+2*3").unwrap().as_ref(),
            &Expr::binary(
                Box::new(Expr::from(1.0)),
                Token::new(TokenKind::Plus, "+".into(), 1),
                Box::new(Expr::binary(
                    Box::new(Expr::from(2.0)),
                    Token::new(TokenKind::Star, "*".into(), 1),
                    Box::new(Expr::from(3.0))
                ))
            )
        );
        assert_eq!(
            test_parse_expr("(1+2)*3").unwrap().as_ref(),
            &Expr::binary(
                Box::new(Expr::Grouping(Box::new(Expr::binary(
                    Box::new(Expr::from(1.0)),
                    Token::new(TokenKind::Plus, "+".into(), 1),
                    Box::new(Expr::from(2.0))
                )))),
                Token::new(TokenKind::Star, "*".into(), 1),
                Box::new(Expr::from(3.0))
            )
        );
        assert_eq!(
            test_parse_expr("1 + 2 + 3").unwrap().as_ref(),
            &Expr::binary(
                Box::new(Expr::binary(
                    Box::new(Expr::from(1.0)),
                    Token::new(TokenKind::Plus, "+".into(), 1),
                    Box::new(Expr::from(2.0))
                )),
                Token::new(TokenKind::Plus, "+".into(), 1),
                Box::new(Expr::from(3.0)),
            )
        );
    }

    #[test]
    fn parsing_comperison_works() {
        assert_eq!(
            test_parse_expr("2 < 3").unwrap().as_ref(),
            &Expr::binary(
                Box::new(Expr::from(2.0)),
                Token::new(TokenKind::Less, "<".into(), 1),
                Box::new(Expr::from(3.0))
            )
        );
    }

    #[test]
    fn parsing_ternary_works() {
        assert_eq!(
            test_parse_expr("2 < 3 ? 4 : 5").unwrap().as_ref(),
            &Expr::Ternary {
                condition: Box::new(Expr::binary(
                    Box::new(Expr::from(2.0)),
                    Token::new(TokenKind::Less, "<".into(), 1),
                    Box::new(Expr::from(3.0))
                )),
                then_expr: Box::new(Expr::from(4.0)),
                else_expr: Box::new(Expr::from(5.0))
            }
        );
    }

    #[test]
    fn assignment_has_lower_predence_than_ternary() {
        assert_eq!(
            test_parse_expr("a = 3 ? 4 : 5").unwrap().as_ref(),
            &Expr::Assignment {
                name: Token::new(TokenKind::Identifier, "a".into(), 1),
                value: Box::new(Expr::Ternary {
                    condition: Box::new(Expr::from(3.0)),
                    then_expr: Box::new(Expr::from(4.0)),
                    else_expr: Box::new(Expr::from(5.0))
                })
            }
        );
    }

    fn test_parse_expr(source: &str) -> Option<Box<Expr>> {
        let error_reporer = ErrorReporter::new();
        let scanner = Scanner::new(&error_reporer);
        let tokens: Vec<_> = scanner.scan_tokens(source).collect();
        let mut parser = Parser::new(tokens, &error_reporer);
        parser.expression()
    }
}
