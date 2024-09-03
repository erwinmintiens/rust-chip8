pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const START_ADDR: u16 = 0x200; // Start of the program is at 0x200

pub struct Emulator {
    program_counter: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT], // Create list with <SCREEN_WIDTH * SCREEN_HEIGHT> number of booleans
    v_reg: [u8; NUM_REGS], // Create list with <NUM_REGS> number of registers which each contain 8 bits
    i_reg: u16,            // I register
    stack_pointer: u16,
    stack: [u16; STACK_SIZE], // Create list of <STACK_SIZE> number of u16 values
    keys: [bool; NUM_KEYS],   // Keep track of which keys are pressed
    delay_timer: u8,
    sound_timer: u8,
}

impl Emulator {
    pub fn new() -> Self {
        let mut emu = Emulator {
            program_counter: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            stack_pointer: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            delay_timer: 0,
            sound_timer: 0,
        };
        emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        emu
    }

    /// Set a v register to a specific value
    fn set_v_reg(&mut self, v_reg: usize, value: u8) {
        if v_reg > 15 {
            panic!(
                "v register value {} not accessible. Must be a value from 0 to 15.",
                v_reg
            );
        }
        self.v_reg[v_reg as usize] = value;
    }

    /// Get a v register value
    fn get_v_reg(&self, v_reg: usize) -> u8 {
        if v_reg > 15 {
            panic!(
                "v register value {} not accessible. Must be a value from 0 to 15.",
                v_reg
            );
        }
        self.v_reg[v_reg]
    }

    /// Clear the screen: set all list values to false
    fn clear_screen(&mut self) {
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
    }

    /// Push a value to the stack and increase the stack_pointer with 1
    fn push(&mut self, value: u16) {
        self.stack[self.stack_pointer as usize] = value;
        self.stack_pointer += 1;
    }

    /// Pop a value from the stack and decrease the stack_pointer with 1
    fn pop(&mut self) -> u16 {
        self.stack_pointer -= 1;
        self.stack[self.stack_pointer as usize]
    }

    /// Reset the emulator to its original state
    pub fn reset(&mut self) {
        self.program_counter = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.stack_pointer = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn tick(&mut self) {
        let operation_code = self.fetch_opcode();
        self.execute_opcode(operation_code);
    }

    /// Fetch the operation code.
    /// This is a combination of the u8 at the program_counter in ram and the next u8 since opcodes are 16 bit.
    /// The program_counter is then increased with 2 to point at the start of the next operation code.
    fn fetch_opcode(&mut self) -> u16 {
        let higher_byte = self.ram[self.program_counter as usize] as u16;
        let lower_byte = self.ram[(self.program_counter + 1) as usize] as u16;
        let operation_code = (higher_byte << 8) | lower_byte;
        self.program_counter += 2;
        operation_code
    }

    /// Execute the given operation code.
    ///
    /// This function transforms the given 16 bit opcode to 4 hexadecimal digits and matches the
    /// values to execute the expected operation.
    fn execute_opcode(&mut self, opcode: u16) {
        let digit1 = (opcode & 0xF000) >> 12;
        let digit2 = (opcode & 0x0F00) >> 8;
        let digit3 = (opcode & 0x00F0) >> 4;
        let digit4 = opcode & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            // "NOP"; Do nothing
            (0, 0, 0, 0) => return,
            // Clear screen
            (0, 0, 0xE, 0) => {
                self.clear_screen();
            }
            // Return from subroutine
            (0, 0, 0xE, 0xE) => {
                self.program_counter = self.pop();
            }
            // Jump to
            (1, _, _, _) => {
                let nnn = opcode & 0xFFF;
                self.program_counter = nnn;
            }
            // Call subroutine
            (2, _, _, _) => {
                let nnn = opcode & 0xFFF;
                self.push(self.program_counter);
                self.program_counter = nnn;
            }
            // Skip if VX == 0xNN
            (3, _, _, _) => {
                let x = digit2 as usize;
                let nn = (opcode & 0xFF) as u8;
                if self.v_reg[x] == nn {
                    self.program_counter += 2;
                }
            }
            // Skip if VX != 0xNN
            (4, _, _, _) => {
                let x = digit2 as usize;
                let nn = (opcode & 0xFF) as u8;
                if self.v_reg[x] != nn {
                    self.program_counter += 2;
                }
            }
            // Skip if VX == VY
            (5, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] == self.v_reg[y] {
                    self.program_counter += 2;
                }
            }

            // Set VX to NN
            (6, _, _, _) => {
                let x = digit2 as usize;
                let nn = (opcode & 0xFF) as u8;
                self.set_v_reg(x, nn);
            }

            // VX += 0xNN
            (7, _, _, _) => {
                let x = digit2 as usize;
                let nn = (opcode & 0xFF) as u8;
                self.set_v_reg(x, self.get_v_reg(x).wrapping_add(nn));
            }

            // VX = VY
            (8, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.set_v_reg(x, self.get_v_reg(y));
            }

            /*
            TODO:
            8XY0 	VX = VY
            8XY1 	VX |= VY
            8XY2 	VX &= VY
            8XY3 	VX ^= VY
            8XY4 	VX += VY
            8XY5 	VX -= VY
            8XY6 	VX >>= 1
            8XY7 	VX = VY - VX
            8XYE 	VX <<= 1
            9XY0 	Skip if VX != VY
            ANNN 	I = 0xNNN
            BNNN 	Jump to V0 + 0xNNN
            CXNN 	VX = rand() & 0xNN
            DXYN 	Draw sprite at (VX, VY)
            EX9E 	Skip if key index in VX is pressed
            EXA1 	Skip if key index in VX isn't pressed
            FX07 	VX = Delay Timer
            FX0A 	Waits for key press, stores index in VX
            FX15 	Delay Timer = VX
            FX18 	Sound Timer = VX
            FX1E 	I += VX
            FX29 	Set I to address of font character in VX
            FX33 	Stores BCD encoding of VX into I
            FX55 	Stores V0 thru VX into RAM address starting at I
            FX65 	Fills V0 thru VX with RAM values starting at address in I
            */
            // In case opcode doesn't match, panic the program
            (_, _, _, _) => unimplemented!(
                "Unimplemented operation code: '{}' (hex='{}')",
                opcode,
                format!("{}{}{}{}", digit1, digit2, digit3, digit4)
            ),
        }
    }

    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        match self.sound_timer {
            0 => return,
            1 => {
                // TODO
                println!("BING");
            }
            _ => {
                self.sound_timer -= 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test setting a v register to a value using the dedicated method
    #[test]
    fn set_v_reg() {
        let mut emul = Emulator::new();
        emul.set_v_reg(12, 0x0089);
        assert_eq!(emul.v_reg[12], 0x0089);
    }

    /// Test setting a non existing v register to a value using the dedicated method, which should panic
    #[test]
    #[should_panic]
    fn set_v_reg_invalid() {
        let mut emul = Emulator::new();
        emul.set_v_reg(16, 0x0089);
    }

    mod test_opcode_execution {
        use super::*;

        /// Test non implemented opcode. This execution should panic.
        #[test]
        #[should_panic]
        fn non_implemented_opcode() {
            let mut emul = Emulator::new();
            emul.execute_opcode(0x0011)
        }

        /// Test do nothing opcode
        #[test]
        fn opcode_0000() {
            let mut emul = Emulator::new();
            emul.execute_opcode(0x0000);
        }

        /// Test clearing screen opcode
        #[test]
        fn opcode_00e0() {
            let mut emul = Emulator::new();
            emul.screen = [true; SCREEN_WIDTH * SCREEN_HEIGHT];
            emul.execute_opcode(0x00E0);
            assert_eq!(emul.screen, [false; SCREEN_WIDTH * SCREEN_HEIGHT]);
        }

        /// Test return from subroutine
        #[test]
        fn opcode_00ee() {
            let mut emul = Emulator::new();
            emul.stack[1] = 0x0011;
            emul.stack_pointer = 2;
            emul.execute_opcode(0x00EE);
            assert_eq!(emul.stack_pointer, 1);
            assert_eq!(emul.program_counter, 0x0011);
        }

        /// Test jump to
        #[test]
        fn opcode_1nnn() {
            let mut emul = Emulator::new();
            emul.execute_opcode(0x1234);
            assert_eq!(emul.program_counter, 0x234);
        }

        /// Test call subroutine
        #[test]
        fn opcode_2nnn() {
            let mut emul = Emulator::new();
            emul.program_counter = 123;
            emul.execute_opcode(0x2345);
            assert_eq!(emul.program_counter, 0x345);
            assert_eq!(emul.stack[0], 123);
            assert_eq!(emul.stack_pointer, 1);
        }

        /// Test skip if VX == 0xNN
        #[test]
        fn opcode_3nnn() {
            let mut emul = Emulator::new();
            emul.program_counter = 0;
            emul.v_reg[6] = 0x0078;
            emul.execute_opcode(0x3678);
            assert_eq!(emul.program_counter, 2); //v6 matches 0x0078, so program counter should increase by 2
            emul.execute_opcode(0x3689);
            assert_eq!(emul.program_counter, 2); //v6 does not match 0x0078, so program counter should not increase by 2
        }

        /// Test skip if VX != 0xNN
        #[test]
        fn opcode_4nnn() {
            let mut emul = Emulator::new();
            emul.program_counter = 0;
            emul.set_v_reg(11, 0x0033);
            emul.execute_opcode(0x4B33);
            assert_eq!(emul.program_counter, 0); // v11 does match 0x0033, program counter should not increase
            emul.execute_opcode(0x4B01);
            assert_eq!(emul.program_counter, 2); // v11 does not match 0x0001, program counter should increase with 2
        }

        /// Test skip if VX == VY
        #[test]
        fn opcode_5xy0() {
            let mut emul = Emulator::new();
            emul.program_counter = 0;
            emul.set_v_reg(8, 0x0093);
            emul.set_v_reg(15, 0x0093);
            emul.execute_opcode(0x58F0);
            assert_eq!(emul.program_counter, 2); // v8 matches v15 => program counter should increase with 2
            emul.set_v_reg(15, 0x0025);
            emul.execute_opcode(0x58F0);
            assert_eq!(emul.program_counter, 2); // v8 does not match v15 => program counter should stay the same
        }

        /// Test set VX to NN
        #[test]
        fn opcode_6xnn() {
            let mut emul = Emulator::new();
            emul.execute_opcode(0x6722);
            assert_eq!(emul.v_reg[7], 0x0022);
        }

        /// Test add NN to VX
        #[test]
        fn opcode_7xnn() {
            let mut emul = Emulator::new();
            emul.set_v_reg(7, 0x0055);
            emul.execute_opcode(0x7733);
            assert_eq!(emul.get_v_reg(7), 0x0088);
            emul.execute_opcode(0x7780);
            assert_eq!(emul.get_v_reg(7), 0x0008); // Test wrapping around
        }

        /// Test VX = VY
        #[test]
        fn opcode_8xy0() {
            let mut emul = Emulator::new();
            emul.set_v_reg(2, 0x0055);
            emul.set_v_reg(3, 0x0039);
            emul.execute_opcode(0x8230);
            assert_eq!(emul.get_v_reg(2), 0x0039);
        }
    }
}
