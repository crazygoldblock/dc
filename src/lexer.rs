use crate::parser::ParseError;

pub struct Tokenizer<'a> {
    buffer: &'a [u8],
    state: State,
    index: usize,
    last: &'a [u8],
}
#[derive(Debug)]
pub struct Token<'a> {
    token: &'a [u8],
}
impl<'a> Token<'a> {
    pub fn as_bytes(&self) -> &'a [u8] {
        self.token
    } 
    pub fn as_str(&self) -> &'a str {
        str::from_utf8(self.token).unwrap()
    } 
}
impl<'a> PartialEq<&str> for Token<'a> {
    fn eq(&self, other: &&str) -> bool {
        self.token == other.as_bytes()
    }
}
pub struct Checkpoint {
    state: State,
    index: usize,
}
#[derive(Clone)]
enum State {
    Start,
    LineComment,
    BlockComment,
    Name{start: usize},
}
impl<'a> Tokenizer<'a> {
    pub fn new<'b>(input: &'b [u8]) -> Tokenizer<'b> {
        Tokenizer { buffer: input, state: State::Start, index: 0, last: &[] }
    }
    pub fn peek(&mut self) -> Result<Token<'a>, ParseError<'a>> {
        let state = self.state.clone();
        let index = self.index;

        let token = self.next();

        self.state = state;
        self.index = index;

        return token;
    }
    pub fn next(&mut self) -> Result<Token<'a>, ParseError<'a>> {
        loop {
            if self.index == self.buffer.len() {
                break;
            }
            let current = self.buffer[self.index];
            let next = self.buffer.get(self.index+1);

            match self.state {
                State::Start => {
                    if b" \r\n".contains(&current) {
                        self.index += 1;
                        continue;
                    }
                    if current == b'/' && next.is_some_and(|x| *x == b'/') {
                        self.index += 2;
                        self.state = State::LineComment;
                        continue;
                    }
                    if current == b'/' && next.is_some_and(|x| *x == b'*') {
                        self.index += 2;
                        self.state = State::BlockComment;
                        continue;
                    }
                    if b"*/:=;->(){},#[]+-.".contains(&current) {
                        if let Some(n) = next {
                            if (current == b'&' && *n == b'&') || (current == b'|' && *n == b'|') ||
                                (current == b'<' && (*n == b'<' || *n == b'=')) ||
                                (current == b'>' && (*n == b'>' || *n == b'=')) || 
                                (*n == b'=' && b"=!|^&+-*/%".contains(&current)) {
                                self.index += 2;
                                self.last = &self.buffer[self.index-2..self.index];
                                return Ok(Token{ token: &self.buffer[self.index-2..self.index]});
                            }
                            if let Some(n2) = self.buffer.get(self.index+2) {
                                if (current == b'<' && *n == b'<' && *n2 == b'=') ||
                                    (current == b'>' && *n == b'>' && *n2 == b'=') {
                                    self.index += 3;
                                    self.last = &self.buffer[self.index-3..self.index];
                                    return Ok(Token{ token: &self.buffer[self.index-3..self.index]});
                                }
                            }
                        }
                        self.index += 1;
                        self.last = &self.buffer[self.index-1..self.index];
                        return Ok(Token{ token: &self.buffer[self.index-1..self.index]});
                    }
                    
                    self.state = State::Name{start: self.index};
                    self.index += 1;
                },
                State::LineComment => {
                    if current == b'\n'  {
                        self.state = State::Start;
                    }
                    self.index += 1;
                },
                State::BlockComment => {
                    if current == b'*' && next.is_some_and(|x| *x == b'/') {
                        self.index += 1;
                        self.state = State::Start;
                    }
                    self.index += 1;
                },
                State::Name{ start } => {
                    if b" \r\n*/:=;->(){},#[]+-.".contains(&current) {
                        self.state = State::Start;
                        self.last = &self.buffer[start..self.index];
                        return Ok(Token{ token: &self.buffer[start..self.index]});
                    }
                    self.index += 1;
                },
            }
        }

        match self.state {
            State::Start | State::BlockComment | State::LineComment => return Err(ParseError::UnexpectedEndOfFile { line: self.get_current_line() }),
            State::Name { start } => {
                self.state = State::Start;
                self.last = &self.buffer[start..self.index];
                return Ok(Token{ token: &self.buffer[start..self.index]});
            },
        }
    }
    pub fn until_semicolon(&mut self) -> Option<Token<'a>> {
        let index = self.index;

        while self.next().ok()? != ";" {}

       return Some(Token{ token: &self.buffer[index..self.index]});
    }
    pub fn combine(&mut self, len: usize) -> Result<Token<'a>, ParseError<'a>> {
        let index = self.index;

        for _ in 0..len {
            self.next()?;
        }

        return Ok(Token{ token: &self.buffer[index..self.index]});
    }
    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint { state: self.state.clone(), index: self.index }
    }
    pub fn restore(&mut self, checkpoint: Checkpoint) {
        self.state = checkpoint.state;
        self.index = checkpoint.index;
    }
    pub fn rest(&self) -> &'a str {
        str::from_utf8(&self.buffer[self.index..]).unwrap()
    }
    pub fn get_current_line(&self) -> &'a str {
        let mut start = 0;
        for i in (0..self.index).rev() {
            if self.buffer[i] == b'\n' || self.buffer[i] == b'\r' {
                start = i;
                break;
            }
        }
        let mut end = self.buffer.len();
        for i in self.index..self.buffer.len() {
            if self.buffer[i] == b'\n' || self.buffer[i] == b'\r' {
                end = i;
                break;
            }
        }
        str::from_utf8(&self.buffer[start..end]).unwrap()
    }
    pub fn last(&self) -> &'a str {
        str::from_utf8(self.last).unwrap()
    }
    pub fn expect_symbol(&mut self, expected: &'static str) -> Result<(), ParseError<'a>> {
        let token = self.next()?;
        if token == expected {
            Ok(())
        }
        else {
            Err(ParseError::ExpectedSymbol { expected, found: token.as_str(), line: self.get_current_line() })
        }
    }
    pub fn expect_keyword(&mut self, expected: &'static str) -> Result<(), ParseError<'a>> {
        let token = self.next()?;
        if token == expected {
            Ok(())
        }
        else {
            Err(ParseError::ExpectedKeyword { expected, found: token.as_str(), line: self.get_current_line() })
        }
    }
    pub fn next_identifier(&mut self) -> Result<Token<'a>, ParseError<'a>> {

        let token = self.next()?;

        if !token.as_bytes()[0].is_ascii_alphabetic() {
            return Err(ParseError::ExpectedIdentifier { found: token.as_str(), line: self.get_current_line() });
        }
        else {
            Ok(token)
        }
    }
}
