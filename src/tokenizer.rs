


pub struct Tokenizer<'a> {
    buffer: &'a str,
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
    pub fn new<'b>(input: &'b str) -> Tokenizer<'b> {
        Tokenizer { buffer: input, state: State::Start, index: 0 }
    }
    pub fn peek(&mut self) -> Option<&'a str> {
        let state = self.state.clone();
        let index = self.index;

        let token = self.next();

        self.state = state;
        self.index = index;

        return token;
    }
    pub fn next(&mut self) -> Option<&'a str> {
        loop {
            if self.index == self.buffer.len() {
                break;
            }
            let current = self.buffer.as_bytes()[self.index];
            let next = self.buffer.as_bytes().get(self.index+1);

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
                                (current == b'>' && (*n == b'>' || *n == b'=')) {
                                self.index += 2;
                                return Some(&self.buffer[self.index-2..self.index]);
                            }
                        }
                        self.index += 1;
                        return Some(&self.buffer[self.index-1..self.index]);
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
                        return Some(&self.buffer[start..self.index]);
                    }
                    self.index += 1;
                },
            }
        }

        match self.state {
            State::Start | State::BlockComment | State::LineComment => return None,
            State::Name { start } => {
                self.state = State::Start;
                return Some(&self.buffer[start..self.index]);
            },
        }
    }
}
