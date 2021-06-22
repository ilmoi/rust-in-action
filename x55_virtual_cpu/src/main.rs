struct CPU {
    registers: [u8; 16], //16 registers, total 16 bytes
    position_in_memory: usize,
    memory: [u8; 4096], //4kb of memory
    stack: [u16; 16], //stack only has 16 slots, beyond that get a stack overflow
    stack_pointer: usize,
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

            // if the opcode begins with 0x02, then we know that the remaining 3 nibbles contain an address we want to jump to
            let nnn = opcode & 0x0FFF;

            match (c, x, y, d) {
                (0, 0, 0, 0) => return,
                (0, 0, 0xE, 0xE) => self.ret(),
                (0x2, _, _, _) => self.call(nnn),
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

    fn call(&mut self, addr: u16) {
        // check for overflow
        if self.stack_pointer +1 >= self.stack.len() {
            panic!("stack overflow")
        }

        //store current mem location on the stack
        self.stack[self.stack_pointer] = self.position_in_memory as u16;
        //increment the stack counter
        self.stack_pointer += 1;
        //set the mem location to fn address
        self.position_in_memory = addr as usize;
    }

    fn ret(&mut self) {
        if self.stack_pointer == 0 {
            panic!("stack underflow")
        }

        self.stack_pointer -= 1;
        self.position_in_memory = self.stack[self.stack_pointer] as usize;
    }
}

fn main() {
    // instantiate an empty cpu
    let mut cpu = CPU {
        registers: [0; 16],
        position_in_memory: 0,
        memory: [0; 4096],
        stack: [0; 16],
        stack_pointer: 0,
    };

    // 2 function calls
    cpu.memory[0x000] = 0x21; cpu.memory[0x001] = 0x00;
    cpu.memory[0x002] = 0x21; cpu.memory[0x003] = 0x00;

    // the actual function (2nd line = return value)
    cpu.memory[0x100] = 0x80; cpu.memory[0x101] = 0x14;
    cpu.memory[0x102] = 0x00; cpu.memory[0x103] = 0xEE;

    cpu.registers[0] = 5;
    cpu.registers[1] = 10;

    cpu.run();

    println!("{}", cpu.registers[0])
}

