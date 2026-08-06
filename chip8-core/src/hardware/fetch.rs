use super::Hardware;

impl Hardware {
    pub(crate) fn fetch(&mut self) -> u16 {
        let i1:  u16  = self.memory[self.pc as usize] as u16;
        let i2:  u16  = self.memory[(self.pc + 1) as usize] as u16;
        let res: u16 = (i1 << 8) | i2;
        self.pc += 2;
        res
    }
}
