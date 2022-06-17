struct CPU {
    registers: [u8; 16],
    position_in_memory: usize, // Rust allows usize for indexing, so we'll use it over u16
    memory: [u8; 0x1000],
    stack: [u16; 16], // Maximum heigtht is 16, raise a stack overflow after 16 nested function calls
    stack_pointer: usize, // Index values within the stack
}

impl Default for CPU {
    fn default() -> Self {
        CPU {
            registers: [0; 16],
            position_in_memory: 0,
            memory: [0; 0x1000],
            stack: [0; 16],
            stack_pointer: 0,
        }
    }
}

impl CPU {
    fn decoding_opcode(opcode: u16) -> (u8, u8, u8, u8) {
        let c = ((opcode & 0xF000) >> 12) as u8;
        let x = ((opcode & 0x0F00) >> 8) as u8;
        let y = ((opcode & 0x00F0) >> 4) as u8;
        let d = ((opcode & 0x000F) >> 0) as u8;
        (c, x, y, d)
    }

    fn read_opcode(&self) -> u16 {
        let p = self.position_in_memory;
        let op_byte1 = self.memory[p] as u16;
        let op_byte2 = self.memory[p + 1] as u16;
        op_byte1 << 8 | op_byte2
    }

    fn run(&mut self) {
        loop {
            let opcode = self.read_opcode();
            self.position_in_memory += 2;
            let (c, x, y, d) = CPU::decoding_opcode(opcode);
            let nnn = opcode & 0x0FFF;
            // let kk = (opcode & 0x00FF) as u8;
            match (c, x, y, d) {
                (0, 0, 0, 0) => { return; }
                (0, 0, 0xE, 0xE) => self.ret(),
                (0x2, _, _, _) => self.call(nnn),
                // CPU RIA/1 match 0x8__4 to perform an addition
                (0x8, _, _, 0x4) => self.add_xy(x, y),
                _ => todo!("opcode {:04x}", opcode),
            }
        }
    }

    fn add_xy(&mut self, x: u8, y: u8) {
        let arg1 = self.registers[x as usize];
        let arg2 = self.registers[y as usize];

        let (val, overflow) = arg1.overflowing_add(arg2);
        self.registers[x as usize] = val;

        // use the last register as carry flag to indicates that
        // an operation has overflowed the u8 register size
        if overflow {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
    }

    fn call(&mut self, addr: u16) {
        let sp = self.stack_pointer;
        let stack = &mut self.stack;

        if sp > stack.len() {
            panic!("Stack overflow!");
        }

        stack[sp] = self.position_in_memory as u16;
        self.stack_pointer += 1;
        self.position_in_memory = addr as usize;
    }

    fn ret(&mut self) {
        if self.stack_pointer == 0 {
            panic!("Stack underflow");
        }

        self.stack_pointer -= 1;
        let call_addr = self.stack[self.stack_pointer];
        self.position_in_memory = call_addr as usize;
    }
}

fn main() {
    let add_twice: [u8; 6] = [
        0x80, 0x14,
        0x80, 0x14,
        0x00, 0xEE,
    ];

    let mut cpu = CPU::default();

    cpu.registers[0] = 5;
    cpu.registers[1] = 10;

    // 8 signifies that the operation involves two registers
    // x maps to cpu.registers[x]
    // y maps to cpu.registers[y]
    // 4 indicates the addition

    let mem = &mut cpu.memory;
    mem[0x000] = 0x21; mem[0x001] = 0x00;
    mem[0x002] = 0x21; mem[0x003] = 0x00;
    mem[0x004] = 0x00; mem[0x005] = 0x00;

    mem[0x100..0x106].copy_from_slice(&add_twice);
    cpu.run();

    assert_eq!(cpu.registers[0], 45);

    println!("5 + (2 * 10) + (2 * 10) = {}", cpu.registers[0]);
}
