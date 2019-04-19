//! Little parser/formatter for our needs

const L_PAREN: &'static str = "{";

pub struct Lexer<'src> {
    src: &'src str,
    state: LexerState,
}

impl<'src> Lexer<'src> {
    pub fn new(src: &'src str) -> Self {
        Lexer {
            src,
            state: LexerState::Normal,
        }
    }

    pub fn next(&mut self) -> Option<Token<'src>> {
        match self.state {
            LexerState::Normal => self.next_normal(),
            LexerState::Parenthesized => self.next_parenthesized(),
        }
    }

    fn next_normal(&mut self) -> Option<Token<'src>> {
        let input = self.src.as_bytes();
        if input.get(0).is_none() {
            return None;
        }
        if input[0] == b'{' {
            self.src = &self.src[1..];
            self.state = LexerState::Parenthesized;
            return self.next();
        }
        let mut pos = 1;
        loop {
            match input.get(pos) {
                None => break,
                Some(ch) if *ch == b'{' => break,
                _ => pos += 1,
            }
        }
        let lit = &self.src[..pos];
        self.src = &self.src[pos..];
        Some(Token::Literal(lit))
    }

    fn next_parenthesized(&mut self) -> Option<Token<'src>> {
        let input = self.src.as_bytes();
        Some(match input.get(0) {
            // ended with '{'
            None => {
                self.state = LexerState::Normal;
                Token::Literal(L_PAREN)
            }
            // '{{'
            Some(ch) if *ch == b'{' => {
                self.src = &self.src[1..];
                self.state = LexerState::Normal;
                Token::Literal(L_PAREN)
            }
            _ => {
                let mut pos = 1;
                loop {
                    match input.get(pos) {
                        None => panic!("unmatched parenthesis"),
                        Some(ch) if *ch == b'}' => {
                            let ph = &self.src[0..pos];
                            self.src = &self.src[pos + 1..];
                            self.state = LexerState::Normal;
                            break Token::Placeholder(ph);
                        }
                        Some(_) => pos += 1,
                    }
                }
            }
        })
    }
}

enum LexerState {
    Normal,
    Parenthesized,
}

#[derive(Debug)]
pub enum Token<'src> {
    Literal(&'src str),
    Placeholder(&'src str),
}
