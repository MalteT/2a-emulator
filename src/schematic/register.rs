use super::Signal;

/// The register block.
/// Containing `R0` through `R7`
#[derive(Debug, Clone)]
pub struct Register {
    content: [u8; 8],
}

impl Register {
    /// Create a new register block.
    pub fn new() -> Self {
        let content = [0; 8];
        Register { content }
    }
    /// Get current data output A of the register.
    pub fn doa(&self, signal: Signal<'_>) -> u8 {
        let (a2, a1, a0) = if signal.mrgaa3() {
            (false, signal.op01(), signal.op00())
        } else {
            (signal.mrgaa2(), signal.mrgaa1(), signal.mrgaa0())
        };
        let addr = (a2 as usize) << 2 + (a1 as usize) << 1 + a0 as usize;
        self.content[addr]
    }
    /// Get current data output B of the register.
    pub fn dob(&self, signal: Signal<'_>) -> u8 {
        let (b2, b1, b0) = if signal.mrgab3() {
            (false, signal.op11(), signal.op10())
        } else {
            (signal.mrgab2(), signal.mrgab1(), signal.mrgab0())
        };
        let addr = (b2 as usize) << 2 + (b1 as usize) << 1 + b0 as usize;
        self.content[addr]
    }
}
