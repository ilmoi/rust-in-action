struct CPU {
    registers: [u8; 16],
    //16 registers, total 16 bytes
    position_in_memory: usize,
    memory: [u8; 4096], //4kb of memory
}

impl CPU {
    fn read_from_mem(&self) -> u16 {
        let part1 = self.memory[self.position_in_memory] as u16;
        let part2 = self.memory[self.position_in_memory + 1] as u16;
        (part1 << 8) | part2
    }

    fn run(&mut self) {
        loop {
            let opcode = self.read_from_mem();
            self.position_in_memory += 2; //move by 2 because that's the word size

            let c = ((opcode & 0xF000) >> 12) as u8;
            let x = ((opcode & 0x0F00) >> 8) as u8;
            let y = ((opcode & 0x00F0) >> 4) as u8;
            let d = (opcode & 0x000F) as u8;

            match (c, x, y, d) {
                (0, 0, 0, 0) => return,
                (0x8, _, _, 0x4) => self.add_xy(x, y),
                _ => todo!(),
            }
        }
    }

    fn add_xy(&mut self, x: u8, y: u8) {
        let val1 = self.registers[x as usize];
        let val2 = self.registers[y as usize];

        // if an overflow happens, "overflow detected" boolean will be set
        let (val, overflow_detected) = val1.overflowing_add(val2);

        self.registers[x as usize] = val;

        // decided to store the result of overflow detected in the last register
        if overflow_detected {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
    }
}

fn main() {
    // instantiate an empty cpu
    let mut cpu = CPU {
        registers: [0; 16],
        position_in_memory: 0,
        memory: [0; 4096],
    };

    // place an opcode into the first 2 bytes in memory
    cpu.memory[0] = 0x80;
    cpu.memory[1] = 0x14;
    cpu.registers[0] = 5;
    cpu.registers[1] = 10;

    cpu.run();

    println!("{}", cpu.registers[0])
}
