use super::*;

impl From<u8> for Constant {
    fn from(constant: u8) -> Self {
        Constant::Constant(constant)
    }
}

impl From<Label> for Constant {
    fn from(label: Label) -> Self {
        Constant::Label(label)
    }
}

impl From<Register> for Source {
    fn from(register: Register) -> Self {
        Source::Register(register)
    }
}

impl From<RegisterDi> for Source {
    fn from(registerdi: RegisterDi) -> Self {
        Source::RegisterDi(registerdi)
    }
}

impl From<RegisterDdi> for Source {
    fn from(registerddi: RegisterDdi) -> Self {
        Source::RegisterDdi(registerddi)
    }
}

impl From<MemAddress> for Source {
    fn from(memory_addr: MemAddress) -> Self {
        Source::MemAddress(memory_addr)
    }
}

impl From<Constant> for Source {
    fn from(constant: Constant) -> Self {
        Source::Constant(constant)
    }
}

impl From<Register> for Destination {
    fn from(register: Register) -> Self {
        Destination::Register(register)
    }
}

impl From<RegisterDi> for Destination {
    fn from(registerdi: RegisterDi) -> Self {
        Destination::RegisterDi(registerdi)
    }
}

impl From<RegisterDdi> for Destination {
    fn from(registerddi: RegisterDdi) -> Self {
        Destination::RegisterDdi(registerddi)
    }
}

impl From<MemAddress> for Destination {
    fn from(memory_addr: MemAddress) -> Self {
        Destination::MemAddress(memory_addr)
    }
}

impl From<Register> for RegisterDi {
    fn from(register: Register) -> Self {
        RegisterDi(register)
    }
}

impl From<Register> for RegisterDdi {
    fn from(register: Register) -> Self {
        RegisterDdi(register)
    }
}

impl From<RegisterDi> for RegisterDdi {
    fn from(registerdi: RegisterDi) -> Self {
        RegisterDdi(registerdi.0)
    }
}

impl From<Constant> for MemAddress {
    fn from(constant: Constant) -> Self {
        MemAddress::Constant(constant)
    }
}

impl From<Register> for MemAddress {
    fn from(register: Register) -> Self {
        MemAddress::Register(register)
    }
}

impl IntoIterator for Asm {
    type Item = Line;
    type IntoIter = ::std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.lines.into_iter()
    }
}

impl Stacksize {
    /// Default Stacksize if none is specified in the asm file.
    // XXX: Replace with Default impl when `const impl` is available.
    pub const fn default() -> Self {
        DEFAULT_STACKSIZE
    }
}

impl Programsize {
    /// Default Programsize if none is specified in the asm file.
    // XXX: Replace with Default impl when `const impl` is available.
    pub const fn default() -> Self {
        DEFAULT_PROGRAMSIZE
    }
}
