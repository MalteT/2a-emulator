use node::node;
use node::Display;

pub type Xor2<'a, I1, I2, O> = Node2x1<'a, I1, I2, O>;
pub type And2<'a, I1, I2, O> = Node2x1<'a, I1, I2, O>;
pub type And4<'a, I1, I2, I3, I4, O> = Node4x1<'a, I1, I2, I3, I4, O>;

#[node{
    ascii("TODO"),
    utf8(file("../../displays/or2.utf8"), id, in0, in1, out),
    add_new(id)
}]
pub struct Or2 {
    pub id: &'static str,
    pub in0: Input,
    in1: Input,
    out: Output,
}

// TODO: ASCII
#[node {
    ascii(file("../../displays/input.utf8"), id, out),
    utf8(file("../../displays/input.utf8"), id, out),
    add_new(id),
}]
pub struct Input {
    pub id: &'static str,
    out: Output,
}

#[node {
    ascii("TODO {}", id),
    utf8("TODO {}", id),
    add_new(id),
}]
pub struct Node2x1 {
    pub id: &'static str,
    in0: Input,
    in1: Input,
    out: Output,
}

#[node {
    ascii("TODO {}", id),
    utf8(file("../../displays/and4.utf8"), id, in0, in1, in2, in3, out),
    add_new(id),
}]
pub struct Node4x1 {
    pub id: &'static str,
    in0: Input,
    in1: Input,
    in2: Input,
    in3: Input,
    out: Output,
}

#[node {
    ascii("TODO {}", id),
    utf8(file("../../displays/dflipflop.utf8"), id, input, clk, out),
    add_new(id),
}]
pub struct DFlipFlop {
    pub id: &'static str,
    input: Input,
    clk: Input,
    out: Output,
}

#[node {
    ascii("TODO {}", id),
    utf8(file("../../displays/dflipflopc.utf8"), id, input, clk, clear, out),
    add_new(id),
}]
pub struct DFlipFlopC {
    pub id: &'static str,
    input: Input,
    clk: Input,
    clear: Input,
    out: Output,
}

#[node {
    ascii("TODO {}", id),
    utf8("TODO {}", id),
    add_new(id),
}]
pub struct Mux2x1 {
    pub id: &'static str,
    in0: Input,
    in1: Input,
    select: Input,
    out: Output,
}

#[node {
    ascii("TODO {}", id),
    utf8(file("../../displays/mux4x1.utf8"), id, in0, in1, in2, in3, select0, select1, out),
    add_new(id),
}]
pub struct Mux4x1 {
    pub id: &'static str,
    in0: Input,
    in1: Input,
    in2: Input,
    in3: Input,
    select0: Input,
    select1: Input,
    out: Output,
}

#[node {
    ascii("TODO {}", id),
    utf8("TODO {}", id),
    add_new(id),
}]
pub struct Mux8x1 {
    pub id: &'static str,
    in0: Input,
    in1: Input,
    in2: Input,
    in3: Input,
    in4: Input,
    in5: Input,
    in6: Input,
    in7: Input,
    select0: Input,
    select1: Input,
    select2: Input,
    out: Output,
}

#[node {
    ascii("TODO {}", reg0),
    utf8(file("../../displays/register.utf8"), id, reg0, reg1, reg2, reg3, reg4, reg5, reg6, reg7),
    add_new(id),
}]
pub struct Register {
    pub id: &'static str,
    reg0: u8,
    reg1: u8,
    reg2: u8,
    reg3: u8,
    reg4: u8,
    reg5: u8,
    reg6: u8,
    reg7: u8,
    aa0: Input,
    aa1: Input,
    aa2: Input,
    write_enable: Input,
    write_select: Input,
    flags_write_enable: Input,
    f_carry: Input,
    f_zero: Input,
    f_negative: Input,
    data: Input,
    clk: Input,
    clear: Input,
    ab2: Input,
    ab1: Input,
    ab0: Input,
    doa: Output,
    cf: Output,
    zf: Output,
    nf: Output,
    ief: Output,
    dob: Output,
}

#[node {
    ascii("TODO {}", out),
    utf8("TODO {}", out),
    add_new(id),
}]
pub struct InstructionRegister {
    pub id: &'static str,
    memdi: Input,
    enable: Input,
    clear: Input,
    out: Output,
}

#[node {
    ascii("TODO {}", inp),
    utf8("TODO {}", inp),
    add_new(id),
}]
pub struct MicroprogramFlipFlopC {
    pub id: &'static str,
    inp: Input,
    clk: Input,
    clear: Input,
    mrgaa3: Output,
    mrgaa2: Output,
    mrgaa1: Output,
    mrgaa0: Output,
    mrgab3: Output,
    mrgab2: Output,
    mrgab1: Output,
    mrgab0: Output,
    mchflg: Output,
    malus3: Output,
    malus2: Output,
    malus1: Output,
    malus0: Output,
    mrgwe: Output,
    mrgws: Output,
    maluia: Output,
    maluib: Output,
    mac3: Output,
    mac2: Output,
    mac1: Output,
    mac0: Output,
    na4: Output,
    na3: Output,
    na2: Output,
    na1: Output,
    na0: Output,
    busen: Output,
    buswr: Output,
}

#[node {
    ascii("TODO {}", id),
    utf8("TODO {}", id),
    add_new(id),
}]
pub struct MemoryController {
    pub id: &'static str,
    enable: Input,
    write: Input,
    clk: Input,
    chip_enable: Output,
    output_enable: Output,
    write_enable: Output,
    wait: Output,
}

#[node {
    ascii("TODO"),
    utf8("TODO"),
    add_new(id),
}]
pub struct ArithmeticLogicalUnit {
    pub id: &'static str,
    cin: Input,
    a: Input,
    b: Input,
    malus3: Input,
    malus2: Input,
    malus1: Input,
    malus0: Input,
    cout: Output,
    zout: Output,
    nout: Output,
    out: Output,
}

#[node {
    ascii("FAKE"),
    utf8("FAKE"),
}]
pub struct Fake {
    out: Output,
}
