use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};

pub const BRAINFUCK_TO_INSTRUCTION_RATIO: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Instruction {
    Add(u8) = b'+',
    Sub(u8) = b'-',
    MoveRight(u8) = b'>',
    MoveLeft(u8) = b'<',
    Output = b'.',
    Input = b',',
    Jump(u32) = b'[',
    JumpBack(u32) = b']',

    #[cfg(feature = "debug")]
    Debug = b'?',
}

#[cfg(not(feature = "debug"))]
pub type Inst = Instruction;

#[cfg(feature = "debug")]
pub type Inst = (Instruction, usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interpreter {
    pub memory: Vec<u8>,
    pub position: usize,
    pub pc: usize,
    pub instructions: Box<[Inst]>,
    pub input: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Output {
    Output(u8),
    Input,
    End,
}

impl Instruction {
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            b'+' => Some(Instruction::Add(1)),
            b'-' => Some(Instruction::Sub(1)),
            b'>' => Some(Instruction::MoveRight(1)),
            b'<' => Some(Instruction::MoveLeft(1)),
            b'.' => Some(Instruction::Output),
            b',' => Some(Instruction::Input),
            b'[' => Some(Instruction::Jump(0)),
            b']' => Some(Instruction::JumpBack(0)),
            #[cfg(feature = "debug")]
            b'?' => Some(Instruction::Debug),
            _ => None,
        }
    }

    pub fn update_with(&mut self, byte: u8) -> bool {
        match (self, byte) {
            (Instruction::Add(n), b'+') => {
                *n += 1;
                true
            }
            (Instruction::Add(n), b'-') => {
                *n = n.saturating_sub(1);
                true
            }
            (Instruction::Sub(n), b'-') => {
                *n += 1;
                true
            }
            (Instruction::Sub(n), b'+') => {
                *n = n.saturating_sub(1);
                true
            }
            (Instruction::MoveRight(n), b'>') => {
                *n += 1;
                true
            }
            (Instruction::MoveRight(n), b'<') => {
                *n = n.saturating_sub(1);
                true
            }
            (Instruction::MoveLeft(n), b'<') => {
                *n += 1;
                true
            }
            (Instruction::MoveLeft(n), b'>') => {
                *n = n.saturating_sub(1);
                true
            }
            _ => false,
        }
    }

    pub fn parse(str: &[u8]) -> Result<(usize, Self), &'static str> {
        let byte = str.get(0).ok_or("expected byte found EOF")?;
        let mut instruction = Self::from_byte(*byte).ok_or("unexpected char")?;
        let mut count = 1;
        while let Some(&nex_byte) = str.get(count)
            && instruction.update_with(nex_byte)
        {
            count += 1;
        }
        Ok((count, instruction))
    }

    #[cfg(not(feature = "debug"))]
    pub fn update_jumps(instructions: &mut [Instruction]) -> Result<(), &'static str> {
        let mut stack = Vec::new();
        for i in 0..instructions.len() {
            match instructions[i] {
                Instruction::Jump(_) => stack.push(i),
                Instruction::JumpBack(_) => {
                    if let Some(start) = stack.pop() {
                        instructions[start] = Instruction::Jump(i as u32);
                        instructions[i] = Instruction::JumpBack(start as u32);
                    } else {
                        return Err("unmatched ']' found");
                    }
                }
                _ => {}
            }
        }
        if !stack.is_empty() {
            return Err("unmatched '[' found");
        }
        Ok(())
    }

    #[cfg(feature = "debug")]
    pub fn update_jumps(
        instructions: &mut [(Instruction, usize)],
    ) -> Result<(), (&'static str, usize)> {
        let mut stack = Vec::new();
        for i in 0..instructions.len() {
            match instructions[i].0 {
                Instruction::Jump(_) => stack.push(i),
                Instruction::JumpBack(_) => {
                    if let Some(start) = stack.pop() {
                        instructions[start].0 = Instruction::Jump(i as u32);
                        instructions[i].0 = Instruction::JumpBack(start as u32);
                    } else {
                        return Err(("unmatched ']' found", i));
                    }
                }
                _ => {}
            }
        }
        if !stack.is_empty() {
            return Err(("unmatched '[' found", 0));
        }
        Ok(())
    }

    pub fn execute(&self, inter: &mut Interpreter) -> Option<Output> {
        match self {
            Instruction::Add(n) => {
                inter.memory[inter.position] = inter.memory[inter.position].saturating_add(*n)
            }
            Instruction::Sub(n) => {
                inter.memory[inter.position] = inter.memory[inter.position].saturating_sub(*n)
            }
            Instruction::MoveRight(n) => {
                inter.position = inter.position.saturating_add(*n as usize);
            }
            Instruction::MoveLeft(n) => {
                inter.position = inter.position.saturating_sub(*n as usize);
            }
            Instruction::Output => {
                return Some(Output::Output(inter.memory[inter.position]));
            }
            Instruction::Input => {
                if let Some(input) = inter.input.pop() {
                    inter.memory[inter.position] = input;
                } else {
                    inter.pc -= 1;
                    return Some(Output::Input);
                }
            }
            Instruction::Jump(start) => {
                if inter.memory[inter.position] == 0 {
                    inter.pc = *start as usize;
                }
            }
            Instruction::JumpBack(start) => {
                if inter.memory[inter.position] != 0 {
                    inter.pc = *start as usize;
                }
            }
            #[cfg(feature = "debug")]
            Instruction::Debug => {
                return inter.debug();
            }
        }
        None
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            memory: vec![0; 30_000],
            position: 0,
            pc: 0,
            instructions: Vec::new().into_boxed_slice(),
            input: Vec::new(),
        }
    }

    pub fn with_str(mut self, bf: &[u8]) -> Result<Self, (&'static str, usize)> {
        let len = bf.len() / BRAINFUCK_TO_INSTRUCTION_RATIO;
        let mut instructions = Vec::with_capacity(len);
        let mut index = 0;
        while index < bf.len() {
            let (count, instruction) = match Instruction::parse(&bf[index..]) {
                Ok((count, instruction)) => (count, instruction),
                Err(err) => return Err((err, index)),
            };

            #[cfg(not(feature = "debug"))]
            instructions.push(instruction);

            #[cfg(feature = "debug")]
            instructions.push((instruction, index));

            index += count;
        }
        #[cfg(not(feature = "debug"))]
        Instruction::update_jumps(&mut instructions).map_err(|e| (e, 0))?;

        #[cfg(feature = "debug")]
        Instruction::update_jumps(&mut instructions)?;

        self.instructions = instructions.into_boxed_slice();
        Ok(self)
    }

    pub fn poll(&mut self) -> Output {
        if self.pc >= self.instructions.len() {
            return Output::End;
        }
        while let Some(instruction) = self.instructions.get(self.pc).map(Clone::clone) {
            self.pc += 1;
            #[cfg(not(feature = "debug"))]
            match instruction.execute(self) {
                Some(out) => return out,
                None => continue,
            }
            #[cfg(feature = "debug")]
            match instruction.0.execute(self) {
                Some(out) => return out,
                None => continue,
            }
        }
        Output::End
    }

    pub fn input(&mut self, input: &[u8]) {
        self.input.extend_from_slice(input);
    }

    pub fn debug(&mut self) -> Option<Output> {
        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(_) => return None,
        };

        let original_terminal_state = (
            crossterm::terminal::is_raw_mode_enabled().unwrap_or(false),
            crossterm::cursor::position().ok(),
        );

        if original_terminal_state.0 == false {
            enable_raw_mode().ok()?;
        }
        execute!(
            terminal.backend_mut(),
            EnterAlternateScreen,
            DisableMouseCapture
        )
        .ok()?;

        let result = self.debug_loop(&mut terminal);

        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )
        .ok()?;
        if original_terminal_state.0 == false {
            disable_raw_mode().ok()?;
        }
        if let Some(pos) = original_terminal_state.1 {
            execute!(
                terminal.backend_mut(),
                crossterm::cursor::MoveTo(pos.0, pos.1)
            )
            .ok()?;
        }

        result
    }

    fn debug_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Option<Output> {
        let mut paused = true;
        let mut last_tick = Instant::now();

        loop {
            if paused {
                terminal
                    .draw(|f| {
                        let chunks = Layout::default()
                            .direction(Direction::Vertical)
                            .constraints(
                                [
                                    Constraint::Percentage(10),
                                    Constraint::Percentage(80),
                                    Constraint::Percentage(10),
                                ]
                                .as_ref(),
                            )
                            .split(f.size());

                        let memory_str = format!(
                            "Memory: {:?}",
                            &self.memory
                                [self.position.saturating_sub(5)..self.position.saturating_add(5)]
                        );
                        let pc_str = format!("PC: {}", self.pc);
                        let pos_str = format!("Position: {}", self.position);
                        let instruction_str =
                            format!("Instruction: {:?}", self.instructions.get(self.pc));

                        let paragraph = Paragraph::new(format!(
                            "{}\n{}\n{}\n{}",
                            memory_str, pc_str, pos_str, instruction_str
                        ))
                        .block(Block::default().title("Debug").borders(Borders::ALL))
                        .wrap(Wrap { trim: true });
                        f.render_widget(paragraph, chunks[1]);

                        let help_text =
                            Paragraph::new("Press 'q' to quit, 'c' to continue, 's' to step")
                                .block(Block::default().title("Help").borders(Borders::ALL));
                        f.render_widget(help_text, chunks[0]);
                    })
                    .ok()?;

                let timeout = Duration::from_millis(250);
                if event::poll(timeout).ok()? {
                    if let Event::Key(key) = event::read().ok()? {
                        match key.code {
                            KeyCode::Char('q') => return None,    // Exit debug mode
                            KeyCode::Char('c') => paused = false, // Continue execution
                            KeyCode::Char('s') => break,          // Step to the next instruction
                            _ => {}
                        }
                    }
                }
            } else {
                break;
            }
            if last_tick.elapsed() >= Duration::from_millis(250) {
                last_tick = Instant::now();
            }
        }

        // Execute a single instruction and return
        if self.pc < self.instructions.len() {
            #[cfg(not(feature = "debug"))]
            {
                if let Some(instruction) = self.instructions.get(self.pc).map(Clone::clone) {
                    self.pc += 1;
                    return instruction.execute(self);
                } else {
                    return Some(Output::End);
                }
            }

            #[cfg(feature = "debug")]
            {
                if let Some(instruction) = self.instructions.get(self.pc).map(Clone::clone) {
                    self.pc += 1;
                    return instruction.0.execute(self);
                } else {
                    return Some(Output::End);
                }
            }
        } else {
            Some(Output::End)
        }
    }
}
