use anyhow::{anyhow, Result};

use super::{error, expr::Expr, token::Token, token_kind::TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
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
    const SYNCHRONIZE: [TokenKind; 8] = [
        TokenKind::Class,
        TokenKind::Fun,
        TokenKind::Var,
        TokenKind::For,
        TokenKind::If,
        TokenKind::While,
        TokenKind::Print,
        TokenKind::Return,
    ];

    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Box<Expr>> {
        self.expression()
    }

    fn expression(&mut self) -> Result<Box<Expr>> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Box<Expr>> {
        self.binary(&Self::EQUALITY_OPERATORS, Self::comparison)
    }

    fn comparison(&mut self) -> Result<Box<Expr>> {
        self.binary(&Self::COMPARISON_OPERATORS, Self::term)
    }

    fn term(&mut self) -> Result<Box<Expr>> {
        self.binary(&Self::TERM_OPERATORS, Self::factor)
    }

    fn factor(&mut self) -> Result<Box<Expr>> {
        self.binary(&Self::FACTOR_OPERATORS, Self::unary)
    }

    fn binary<F>(&mut self, operators: &[TokenKind], mut operand: F) -> Result<Box<Expr>>
    where
        F: FnMut(&mut Self) -> Result<Box<Expr>>,
    {
        let mut expr = operand(self)?;
        while self.match_any(operators) {
            expr = Box::new(Expr::Binary(
                expr,
                self.previous().to_owned(),
                operand(self)?,
            ));
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Box<Expr>> {
        if self.match_any(&Self::UNARY_OPERATORS) {
            let operator = self.previous().to_owned();
            let right = self.unary()?;
            Ok(Box::new(Expr::Unary(operator, right)))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Box<Expr>> {
        let expr = if self.match_single(&TokenKind::False) {
            Expr::Boolean(false)
        } else if self.match_single(&TokenKind::True) {
            Expr::Boolean(true)
        } else if self.match_single(&TokenKind::Nil) {
            Expr::Nil
        } else if let Some(literal) = self.literal() {
            literal
        } else if self.match_single(&TokenKind::LeftParen) {
            let expr = self.expression()?;
            self.consume(&TokenKind::RightParen, || {
                "Expect ')' after expression.".into()
            })?;
            Expr::Grouping(expr)
        } else {
            self.peek().error("Expect expression")?
        };
        Ok(Box::new(expr))
    }

    fn literal(&mut self) -> Option<Expr> {
        let expr = if self.is_at_end() {
            None
        } else if let TokenKind::Number(number) = self.peek().kind {
            Some(Expr::Number(number))
        } else if let TokenKind::String(string) = &self.peek().kind {
            Some(Expr::String(string.into()))
        } else {
            None
        };
        if expr.is_some() {
            self.advance();
        }
        expr
    }

    fn match_any(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds {
            if self.check(kind) {
                self.advance();
                return true;
            }
        }
        return false;
    }

    fn match_single(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            return false;
        }
    }

    fn consume<M>(&mut self, kind: &TokenKind, message: M) -> Result<&Token>
    where
        M: Fn() -> String,
    {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            self.peek().error(&message())
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
