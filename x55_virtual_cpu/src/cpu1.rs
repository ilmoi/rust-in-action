struct CPU {
    current_operation: u16, //opcodes are 16 bits or 4 "nibbles" of 4 bits each

    registers: [u8; 2], //2 regs of 8 bits each
}

impl CPU {
    fn run(&mut self) {
        let opcode = self.current_operation;

        let c = ((opcode & 0xF000) >> 12) as u8;
        let x = ((opcode & 0x0F00) >> 8) as u8;
        let y = ((opcode & 0x00F0) >> 4) as u8;
        let d = (opcode & 0x000F) as u8;

        match (c,x,y,d) {
            (0x8, _, _, 0x4) => self.add_xy(x, y),
            _ => todo!(),
        }
    }

    fn add_xy(&mut self, x: u8, y: u8) {
        self.registers[x as usize] += self.registers[y as usize];
    }
}

fn main() {
    let mut cpu = CPU {
        current_operation: 0,
        registers: [0; 2],
    };

    cpu.current_operation = 0x8014;
    cpu.registers[0] = 5;
    cpu.registers[1] = 10;

    cpu.run();

    println!("{}", cpu.registers[0])
}
