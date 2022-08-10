#![allow(unused_variables)]

#[derive(Debug)]
pub struct Interrupts {
    irq: bool,
    nmi: bool
}

impl Interrupts {
    pub fn new() -> Interrupts {
        Interrupts {
            irq: false,
            nmi: false,
        }
    }
    pub fn get_irq_assert(&self) -> bool {
        self.irq
    }
    pub fn get_nmi_assert(&self) -> bool {
        self.nmi
    }
    pub fn assert_irq(&mut self) {
        self.irq = true;
    }
    pub fn deassert_irq(&mut self) {
        self.irq = false;
    }
    pub fn assert_nmi(&mut self) {
        self.nmi = true;
    }
    pub fn deassert_nmi(&mut self) {
        self.nmi = false;
    }
}