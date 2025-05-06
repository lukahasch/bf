/// Cell structure:
/// [index] [b1] [b2] [b3] [b4] [f1] [f2] [f3] [f4] [f5] [f6]
/// Program structure:
/// [ [block1] [block2] [block3] [block4] ]
/// Program counter is always on [index]

pub struct Context<'a> {
    pub bf: &'a mut Vec<u8>,
    pub stack: Stack,
}

pub struct Stack;

pub fn construct(f: impl FnOnce(&mut Context)) -> Vec<u8> {
    let mut bf = Vec::new();
    let mut context = Context::new(&mut bf);
    context.increase(1);
    f(&mut context);
    bf
}

impl<'a> Context<'a> {
    pub fn new(bf: &'a mut Vec<u8>) -> Self {
        Self { bf, stack: Stack }
    }

    pub fn increase(&mut self, amount: u8) {
        for _ in 0..amount {
            self.bf.push(b'+');
        }
    }

    pub fn decrease(&mut self, amount: u8) {
        for _ in 0..amount {
            self.bf.push(b'-');
        }
    }

    pub fn move_right(&mut self, amount: u8) {
        for _ in 0..amount {
            self.bf.push(b'>');
        }
    }

    pub fn move_left(&mut self, amount: u8) {
        for _ in 0..amount {
            self.bf.push(b'<');
        }
    }

    pub fn output(&mut self) {
        self.bf.push(b'.');
    }

    pub fn input(&mut self) {
        self.bf.push(b',');
    }

    pub fn begin_loop<O>(&mut self, inner: impl FnOnce(&mut Self) -> O) -> O {
        self.bf.push(b'[');
        let result = inner(self);
        self.bf.push(b']');
        result
    }
}
