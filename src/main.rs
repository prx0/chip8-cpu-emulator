struct CPU {
    registers: [u8; 16],
    position_in_memory: usize, // Rust allows usize for indexing, so we'll use it over u16
    memory: [u8; 0x1000],
}

impl Default for CPU {
    fn default() -> Self {
        CPU {
            registers: [0; 16],
            position_in_memory: 0,
            memory: [0; 0x1000],
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
            match (c, x, y, d) {
                (0, 0, 0, 0) => { return; }
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
}

fn main() {
    let mut cpu = CPU::default();

    cpu.registers[0] = 5;
    cpu.registers[1] = 10;
    cpu.registers[2] = 10;
    cpu.registers[3] = 10;

    // 8 signifies that the operation involves two registers
    // x maps to cpu.registers[x]
    // y maps to cpu.registers[y]
    // 4 indicates the addition

    let mem = &mut cpu.memory;
    mem[0] = 0x80; mem[1] = 0x14; // Load opcode 0x8014, ask to adds register 1 to register 0
    mem[2] = 0x80; mem[3] = 0x24; // Load opcode 0x8024, ask to adds register 2 to register 0
    mem[4] = 0x80; mem[5] = 0x34; // Load opcode 0x8034, ask to adds register 3 to register 0

    cpu.run();

    assert_eq!(cpu.registers[0], 35);

    println!("5 + 10 + 10 + 10 = {}", cpu.registers[0]);
}
