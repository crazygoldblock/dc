


pub struct Tokenizer<'a> {
    buffer: &'a [u8],
    state: State,
    index: usize,
}
pub struct Token<'a> {
    token: &'a [u8],
}
impl<'a> Token<'a> {
    pub fn as_bytes(&self) -> &'a [u8] {
        self.token
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
        Tokenizer { buffer: input, state: State::Start, index: 0 }
    }
    pub fn peek(&mut self) -> Option<Token<'a>> {
        let state = self.state.clone();
        let index = self.index;

        let token = self.next();

        self.state = state;
        self.index = index;

        return token;
    }
    pub fn next(&mut self) -> Option<Token<'a>> {
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
                                return Some(Token{ token: &self.buffer[self.index-2..self.index]});
                            }
                            if let Some(n2) = self.buffer.get(self.index+2) {
                                if (current == b'<' && *n == b'<' && *n2 == b'=') ||
                                    (current == b'>' && *n == b'>' && *n2 == b'=') {
                                    self.index += 3;
                                    return Some(Token{ token: &self.buffer[self.index-3..self.index]});
                                }
                            }
                        }
                        self.index += 1;
                        return Some(Token{ token: &self.buffer[self.index-1..self.index]});
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
                        return Some(Token{ token: &self.buffer[start..self.index]});
                    }
                    self.index += 1;
                },
            }
        }

        match self.state {
            State::Start | State::BlockComment | State::LineComment => return None,
            State::Name { start } => {
                self.state = State::Start;
                return Some(Token{ token: &self.buffer[start..self.index]});
            },
        }
    }
    pub fn until_semicolon(&mut self) -> Option<Token<'a>> {
        let index = self.index;

        while self.next()? != ";" {}

       return Some(Token{ token: &self.buffer[index..self.index]});
    }
    pub fn combine(&mut self, len: usize) -> Option<Token<'a>> {
        let index = self.index;

        for _ in 0..len {
            self.next()?;
        }

        return Some(Token{ token: &self.buffer[index..self.index]});
    }
    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint { state: self.state.clone(), index: self.index }
    }
    pub fn restore(&mut self, checkpoint: Checkpoint) {
        self.state = checkpoint.state;
        self.index = checkpoint.index;
    }
}
