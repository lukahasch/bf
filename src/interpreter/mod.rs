use std::collections::{HashMap, VecDeque};
use std::ops::Index;
use std::range::Range;

#[derive(Debug, Clone, PartialEq)]
pub struct Interpreter<'a> {
    pub bf: &'a [u8],
    pub pc: usize,
    pub cache: HashMap<usize, usize>,

    pub tape: Vec<u8>,
    pub location: usize,

    pub input: VecDeque<u8>,

    pub history: Vec<Delta>,
    pub keep_history: bool,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum Delta {
    Add(u8),
    Sub(u8),
    Move(isize),
    Jump(usize),
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum Output {
    RequiresInput,
    End,
    UnallowedCharacter(usize),
    TriedToMoveOutOfBounds,
}

impl<'a> Interpreter<'a> {
    pub fn new(bf: &'a [u8]) -> Self {
        Self {
            bf,
            pc: 0,
            cache: HashMap::new(),
            tape: vec![0; 30_000],
            location: 0,
            input: VecDeque::new(),
            history: Vec::new(),
            keep_history: false,
        }
    }

    pub fn keep_history(&mut self) -> &mut Self {
        self.keep_history = true;
        self
    }

    pub fn stop_keeping_histroy(&mut self) -> &mut Self {
        self.keep_history = false;
        self.history.clear();
        self
    }

    pub fn run(&mut self) -> Result<u8, Output> {
        while self.pc < self.bf.len() {
            match self.tick() {
                Ok(Some(byte)) => return Ok(byte),
                Ok(None) => continue,
                Err(e) => return Err(e),
            }
        }
        Err(Output::End)
    }

    pub fn run_steps(&mut self, mut steps: usize) -> Result<Option<u8>, Output> {
        while self.pc < self.bf.len() && steps > 0 {
            match self.tick() {
                Ok(Some(byte)) => return Ok(Some(byte)),
                Ok(None) => steps -= 1,
                Err(e) => return Err(e),
            }
        }
        if steps == 0 {
            return Ok(None);
        }
        Err(Output::End)
    }

    pub fn tick(&mut self) -> Result<Option<u8>, Output> {
        if self.pc >= self.bf.len() {
            return Err(Output::End);
        }

        let command = self.bf[self.pc];

        match command {
            b'>' => self.move_right(),
            b'<' => self.move_left(),
            b'+' => self.add(),
            b'-' => self.sub(),
            b'.' => self.output(),
            b',' => self.input(),
            b'[' => self.jump_forward(),
            b']' => self.jump_back(),
            b'{' => self.comment(),
            _ => Err(Output::UnallowedCharacter(self.pc)),
        }
    }

    pub fn cell(&mut self, index: usize) -> &mut u8 {
        if index >= self.tape.len() {
            self.tape.resize(index + 100, 0);
        }
        &mut self.tape[index]
    }

    pub fn cells<T>(&self, indeces: T) -> &<Vec<u8> as Index<T>>::Output
    where
        Vec<u8>: Index<T>,
    {
        &self.tape[indeces]
    }

    fn move_right(&mut self) -> Result<Option<u8>, Output> {
        self.pc += 1;
        self.location += 1;
        if self.keep_history {
            self.history.push(Delta::Move(1));
        }
        Ok(None)
    }

    fn move_left(&mut self) -> Result<Option<u8>, Output> {
        if self.location == 0 {
            return Err(Output::TriedToMoveOutOfBounds);
        }
        self.pc += 1;
        self.location -= 1;
        if self.keep_history {
            self.history.push(Delta::Move(-1));
        }
        Ok(None)
    }

    fn add(&mut self) -> Result<Option<u8>, Output> {
        self.pc += 1;
        *self.cell(self.location) = self.cell(self.location).wrapping_add(1);
        if self.keep_history {
            self.history.push(Delta::Add(1));
        }
        Ok(None)
    }

    fn sub(&mut self) -> Result<Option<u8>, Output> {
        self.pc += 1;
        *self.cell(self.location) = self.cell(self.location).wrapping_sub(1);
        if self.keep_history {
            self.history.push(Delta::Sub(1));
        }
        Ok(None)
    }

    fn output(&mut self) -> Result<Option<u8>, Output> {
        self.pc += 1;
        if self.keep_history {
            self.history.push(Delta::Add(0));
        }
        Ok(Some(*self.cell(self.location)))
    }

    fn input(&mut self) -> Result<Option<u8>, Output> {
        self.pc += 1;
        if let Some(byte) = self.input.pop_front() {
            *self.cell(self.location) = byte;
            if self.keep_history {
                self.history.push(Delta::Add(byte));
            }
            Ok(None)
        } else {
            Err(Output::RequiresInput)
        }
    }

    /// if the current cell is 0, jump to the matching ]
    fn jump_forward(&mut self) -> Result<Option<u8>, Output> {
        if *self.cell(self.location) != 0 {
            self.pc += 1;
            return Ok(None);
        }
        if let Some(&jump) = self.cache.get(&self.pc) {
            self.pc = jump;
            if self.keep_history {
                self.history.push(Delta::Jump(self.pc));
            }
            return Ok(None);
        }
        let mut depth = 1;
        let jump = self.pc;
        while depth > 0 {
            self.pc += 1;
            if self.pc >= self.bf.len() {
                return Err(Output::TriedToMoveOutOfBounds);
            }
            match self.bf[self.pc] {
                b'[' => depth += 1,
                b']' => depth -= 1,
                _ => {}
            }
        }
        self.pc += 1;
        self.cache.insert(jump, self.pc);
        if self.keep_history {
            self.history.push(Delta::Jump(self.pc));
        }
        Ok(None)
    }

    fn jump_back(&mut self) -> Result<Option<u8>, Output> {
        if *self.cell(self.location) == 0 {
            self.pc += 1;
            return Ok(None);
        }
        if let Some(&jump) = self.cache.get(&self.pc) {
            self.pc = jump;
            if self.keep_history {
                self.history.push(Delta::Jump(self.pc));
            }
            return Ok(None);
        }
        let mut depth = 1;
        let jump = self.pc;
        while depth > 0 {
            self.pc -= 1;
            if self.pc == 0 {
                return Err(Output::TriedToMoveOutOfBounds);
            }
            match self.bf[self.pc] {
                b']' => depth += 1,
                b'[' => depth -= 1,
                _ => {}
            }
        }
        self.cache.insert(jump, self.pc);
        if self.keep_history {
            self.history.push(Delta::Jump(self.pc));
        }
        Ok(None)
    }

    fn comment(&mut self) -> Result<Option<u8>, Output> {
        if let Some(jump) = self.cache.get(&self.pc) {
            self.pc = *jump;
            return Ok(None);
        }
        let mut depth = 1;
        let jump = self.pc;
        while depth > 0 {
            self.pc += 1;
            if self.pc >= self.bf.len() {
                return Err(Output::TriedToMoveOutOfBounds);
            }
            match self.bf[self.pc] {
                b'{' => depth += 1,
                b'}' => depth -= 1,
                _ => {}
            }
        }
        self.pc += 1;
        self.cache.insert(jump, self.pc);
        if self.keep_history {
            self.history.push(Delta::Jump(self.pc));
        }
        Ok(None)
    }

    pub fn load_input(&mut self, input: &[u8]) {
        self.input.extend(input.iter());
    }

    pub fn take_output(&mut self, output: &mut [u8]) -> Result<usize, Output> {
        for i in 0..output.len() {
            match self.run() {
                Ok(byte) => output[i] = byte,
                Err(Output::End) => return Ok(i + 1),
                Err(o) => return Err(o),
            }
        }
        Ok(output.len())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn inc() {
        let mut interpreter = Interpreter::new(b"+");
        assert_eq!(interpreter.run(), Err(Output::End));
        assert_eq!(interpreter.tape[0], 1);
    }

    #[test]
    fn dec() {
        let mut interpreter = Interpreter::new(b"-");
        assert_eq!(interpreter.run(), Err(Output::End));
        assert_eq!(interpreter.tape[0], 255);
    }

    #[test]
    fn r#move() {
        let mut interpreter = Interpreter::new(b">");
        assert_eq!(interpreter.run(), Err(Output::End));
        assert_eq!(interpreter.location, 1);
    }

    #[test]
    fn move_back() {
        let mut interpreter = Interpreter::new(b"><");
        assert_eq!(interpreter.run(), Err(Output::End));
        assert_eq!(interpreter.location, 0);
    }

    #[test]
    fn output() {
        let mut interpreter = Interpreter::new(b".");
        interpreter.tape[0] = 65; // ASCII for 'A'
        assert_eq!(interpreter.run(), Ok(65));
    }

    #[test]
    fn input() {
        let mut interpreter = Interpreter::new(b",");
        interpreter.input.push_back(65); // ASCII for 'A'
        assert_eq!(interpreter.run(), Err(Output::End));
        assert_eq!(interpreter.tape[0], 65);
    }

    #[test]
    fn jump_forward() {
        let mut interpreter = Interpreter::new(b"[+]");
        assert_eq!(interpreter.run_steps(100), Err(Output::End));
        assert_eq!(interpreter.tape[0], 0);
    }

    #[test]
    fn jump_back() {
        let mut interpreter = Interpreter::new(b"+++++[-]");
        assert_eq!(interpreter.run(), Err(Output::End));
        assert_eq!(interpreter.tape[0], 0);
    }

    #[test]
    fn move_out_of_bounds() {
        let mut interpreter = Interpreter::new(b"<");
        interpreter.location = 0;
        assert_eq!(interpreter.move_left(), Err(Output::TriedToMoveOutOfBounds));
    }

    #[test]
    fn unallowed_character() {
        let mut interpreter = Interpreter::new(b"@");
        assert_eq!(interpreter.run(), Err(Output::UnallowedCharacter(0)));
    }

    #[test]
    fn comment() {
        let mut interpreter = Interpreter::new(b"{+}");
        assert_eq!(interpreter.run(), Err(Output::End));
        assert_eq!(interpreter.tape[0], 0);
    }

    #[test]
    fn hello_world() {
        let mut interpreter = Interpreter::new(b"++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.");
        let mut output = [0; 13];
        assert_eq!(
            interpreter.take_output(&mut output),
            Ok(b"Hello World!\n".len())
        );
        assert_eq!(str::from_utf8(&output).unwrap(), "Hello World!\n");
    }
}
