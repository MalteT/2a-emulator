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

impl From<RegisterDI> for Source {
    fn from(registerdi: RegisterDI) -> Self {
        Source::RegisterDI(registerdi)
    }
}

impl From<RegisterDDI> for Source {
    fn from(registerddi: RegisterDDI) -> Self {
        Source::RegisterDDI(registerddi)
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

impl From<RegisterDI> for Destination {
    fn from(registerdi: RegisterDI) -> Self {
        Destination::RegisterDI(registerdi)
    }
}

impl From<RegisterDDI> for Destination {
    fn from(registerddi: RegisterDDI) -> Self {
        Destination::RegisterDDI(registerddi)
    }
}

impl From<MemAddress> for Destination {
    fn from(memory_addr: MemAddress) -> Self {
        Destination::MemAddress(memory_addr)
    }
}

impl From<Register> for RegisterDI {
    fn from(register: Register) -> Self {
        RegisterDI(register)
    }
}

impl From<Register> for RegisterDDI {
    fn from(register: Register) -> Self {
        RegisterDDI(register)
    }
}

impl From<RegisterDI> for RegisterDDI {
    fn from(registerdi: RegisterDI) -> Self {
        RegisterDDI(registerdi.0)
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
