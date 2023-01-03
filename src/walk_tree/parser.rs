use super::{expr::Expr, token::Token, token_kind::TokenKind};

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn expression(&mut self) -> Box<Expr> {
        self.equality()
    }

    fn equality(&mut self) -> Box<Expr> {
        // let mut expr = self.comparison();
        // while self.match_token(&[TokenKind::BangEqual, TokenKind::EqualEqual]) {
        //     let operator = self.previous();
        //     let right = self.comparison();
        //     expr = Box::new(Expr::Binary(expr, operator.to_owned(), right));
        // }
        // expr
        self.left_assoc(
            &[TokenKind::BangEqual, TokenKind::EqualEqual],
            Self::comparison,
        )
    }

    fn comparison(&mut self) -> Box<Expr> {
        todo!()
    }

    fn left_assoc(
        &mut self,
        operators: &[TokenKind],
        operand: fn(&mut Parser) -> Box<Expr>,
    ) -> Box<Expr> {
        let mut expr = operand(self);
        while self.match_token(operators) {
            let operator = self.previous().to_owned();
            let right = operand(self);
            expr = Box::new(Expr::Binary(expr, operator, right));
        }
        expr
    }

    fn match_token(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds {
            if self.check(kind) {
                self.advance();
                return true;
            }
        }
        return false;
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().kind() == kind
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind() == &TokenKind::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
}
