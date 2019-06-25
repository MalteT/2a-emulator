use node::{Display, Node, Wire};
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::widgets::Widget;

use std::cell::RefCell;
use std::convert::TryInto;
use std::rc::Rc;

mod fns;
mod mp_ram;
mod types;

use crate::tui::grid::StrGrid;
pub use fns::*;
pub use mp_ram::MPRam;
pub use types::*;

/// Trait alias until stabilization.
pub trait NodeDisplay: Node + Display {}
impl<T: Node + Display> NodeDisplay for T {}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Ord, Eq)]
pub enum Part {
    Al1,
    Al2,
    Al3,
    Il1,
    Il2,
    Iff1,
    Iff2,
    InterruptLogic,
}

#[derive(Debug)]
pub struct Machine<'node> {
    displaying_part: Part,
    last_cache_id: usize,
    iff1_out: Wire<'node, bool>,
    clk: Rc<RefCell<Input<'node, bool>>>,
    reset: Rc<RefCell<Input<'node, bool>>>,
    edge_int: Rc<RefCell<Input<'node, bool>>>,
    al1: Rc<RefCell<Or2<'node, bool, bool, bool>>>,
    al2: Rc<RefCell<And2<'node, bool, bool, bool>>>,
    al3: Rc<RefCell<Xor2<'node, bool, bool, bool>>>,
    il1: Rc<RefCell<And4<'node, bool, bool, bool, bool, bool>>>,
    il2: Rc<RefCell<Or2<'node, bool, bool, bool>>>,
    iff1: Rc<RefCell<DFlipFlopC<'node, bool, bool, bool, bool>>>,
    iff2: Rc<RefCell<DFlipFlop<'node, bool, bool, bool>>>,
}

pub enum Signal {
    Rising,
    Falling,
    High,
    Low,
}

impl<'node> Machine<'node> {
    pub fn compose() -> Machine<'node> {
        // Create all nodes
        let (al1_n, al1) = Or2::new("AL1", make_or2());
        let (al2_n, al2) = And2::new("AL2", make_and2());
        let (al3_n, al3) = Xor2::new("AL3", make_xor2());
        let (_bl1_n, _bl1) = And2::new("BL1", make_and2());
        let (_bl2_n, _bl2) = And2::new("BL2", make_and2());
        let (_bl3_n, _bl3) = Or2::new("BL3", make_or2());
        let (_br_n, _br) = InstructionRegister::new("BR", make_instruction_register());
        let (am1_n, _am1) = Mux8x1::new("AM1", make_mux8x1());
        let (am2_n, am2) = Mux4x1::new("AM2", make_mux4x1());
        let (_am3_n, _am3) = Mux2x1::new("AM3", make_mux2x1());
        let (_am4_n, _am4) = Mux2x1::new("AM4", make_mux2x1());
        let (il1_n, il1) = And4::new("IL1", make_and4());
        let (il2_n, il2) = Or2::new("IL2", make_or2());
        let (iff1_n, iff1) = DFlipFlopC::new("IFF1", make_dflipflopc());
        let (iff2_n, iff2) = DFlipFlop::new("IFF2", make_dflipflop());
        let (_mpram_n, _mpram) = MPRam::new(make_mpram());
        let (
            _mpff_n,
            _mrgaa3,
            _mrgaa2,
            _mrgaa1,
            _mrgaa0,
            _mrgab3,
            _mrgab2,
            _mrgab1,
            _mrgab0,
            _mchflg,
            _malus3,
            _malus2,
            _malus1,
            _malus0,
            _mrgwe,
            _mrgws,
            _maluia,
            _maluib,
            _mac3,
            _mac2,
            _mac1,
            _mac0,
            _na4,
            _na3,
            _na2,
            _na1,
            _na0,
            _busen,
            _buswr,
        ) = MicroprogramFlipFlopC::new("MPFF", make_mpff());
        let (_mctr_n, _ce, _oe, _we, _wait) =
            MemoryController::new("MCTR", make_memory_controller());
        let (_rm1_n, _rm1) = Mux2x1::new("RM1", make_mux2x1());
        let (_rm2_n, _rm2) = Mux2x1::new("RM2", make_mux2x1());
        let (_rm3_n, _rm3) = Mux2x1::new("RM3", make_mux2x1());
        let (_rm4_n, _rm4) = Mux2x1::new("RM4", make_mux2x1());
        let (_rm5_n, _rm5) = Mux2x1::new("RM5", make_mux2x1());
        let (_rm6_n, _rm6) = Mux2x1::new("RM6", make_mux2x1());
        let (_dm1_n, _dm1) = Mux2x1::new("DM1", make_mux2x1());
        let (_dm2_n, _dm2) = Mux2x1::new("DM2", make_mux2x1());
        let (_reg_n, _doa, cf, zf, nf, ief, _dob) = Register::new("REG", make_register());
        let (_alu_n, co, zo, no, _alu_out) =
            ArithmeticLogicalUnit::new("ALU", make_arithmetic_logical_unit());
        // Clk, Reset, Interrupts
        let (clk_n, clk) = Input::with_name("CLK");
        let (reset_n, reset) = Input::with_name("reset");
        let (edge_int_n, edge_int) = Input::with_name("reset");
        // Fake entry
        let (_fake_n, fake) = Fake::new(|| true);
        // High and Low
        let (_high_n, high) = Const::new("HIGH", || true);
        let (_low_n, low) = Const::new("LOW", || false);

        // Compose everything
        al1_n
            .borrow_mut()
            .plug_in0(clk.clone())
            .plug_in1(fake.clone());
        al2_n
            .borrow_mut()
            .plug_in0(ief.clone())
            .plug_in1(al1.clone());
        al3_n.borrow_mut().plug_in0(fake.clone()).plug_in1(am2);
        am1_n
            .borrow_mut()
            .plug_in0(low.clone())
            .plug_in1(high.clone())
            .plug_in2(al3.clone())
            .plug_in3(cf.clone())
            .plug_in4(co.clone())
            .plug_in5(zo.clone())
            .plug_in6(no.clone())
            .plug_in7(al2.clone());
        am2_n.borrow_mut()
            .plug_in0(high.clone())
            .plug_in1(cf.clone())
            .plug_in2(zf.clone())
            .plug_in3(nf.clone());
        il1_n
            .borrow_mut()
            .plug_in0(iff1.clone())
            .plug_in1(fake.clone())
            .plug_in2(fake.clone())
            .plug_in3(fake.clone());
        il2_n
            .borrow_mut()
            .plug_in0(iff2.clone())
            .plug_in1(reset.clone());
        iff1_n
            .borrow_mut()
            .plug_input(high.clone())
            .plug_clk(edge_int)
            .plug_clear(il2);
        iff2_n.borrow_mut().plug_input(il1).plug_clk(clk.clone());

        Machine {
            displaying_part: Part::InterruptLogic,
            last_cache_id: 0,
            clk: clk_n,
            reset: reset_n,
            edge_int: edge_int_n,
            iff1_out: iff1,
            al1: al1_n,
            al2: al2_n,
            al3: al3_n,
            il1: il1_n,
            il2: il2_n,
            iff1: iff1_n,
            iff2: iff2_n,
        }
    }
    /// Get a part from the machine.
    pub fn get_part(&self, part: Part) -> Rc<RefCell<dyn NodeDisplay + 'node>> {
        match part {
            Part::Al1 => self.al1.clone(),
            Part::Al2 => self.al2.clone(),
            Part::Al3 => self.al3.clone(),
            Part::Il1 => self.il1.clone(),
            Part::Il2 => self.il2.clone(),
            Part::Iff1 => self.iff1.clone(),
            Part::Iff2 => self.iff2.clone(),
            Part::InterruptLogic => panic!("Cannot get composition of parts"),
        }
    }
    /// Send a clk signal to the machine.
    pub fn clk(&mut self, signal: bool) {
        self.clk.borrow_mut().set(signal);
        // TODO: Change this to be usefull.
        self.invalidate_cache();
        self.iff1_out.get(self.last_cache_id);
    }
    /// Send a reset signal to the machine.
    pub fn reset(&mut self, signal: bool) {
        self.reset.borrow_mut().set(signal);
        // TODO: Change this to be usefull.
        self.invalidate_cache();
        self.iff1_out.get(self.last_cache_id);
    }
    /// Send an edge interrupt signal to the machine.
    pub fn edge_int(&mut self, signal: bool) {
        self.edge_int.borrow_mut().set(signal);
        // TODO: Change this to be usefull.
        self.invalidate_cache();
        self.iff1_out.get(self.last_cache_id);
    }
    /// Invalidate the current cache.
    /// Usually after an input changed.
    pub fn invalidate_cache(&mut self) {
        self.last_cache_id = (self.last_cache_id + 1) % usize::max_value();
    }
    /// Select which part of the machine to show.
    pub fn show(&mut self, part: Part) {
        self.displaying_part = part;
    }
}

impl Widget for Machine<'_> {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let mut x = area.x;
        let mut y = area.y;
        match self.displaying_part {
            Part::InterruptLogic => {
                let il1 = self.get_part(Part::Il1).borrow().to_utf8_string();
                let il2 = self.get_part(Part::Il2).borrow().to_utf8_string();
                let iff1 = self.get_part(Part::Iff1).borrow().to_utf8_string();
                let iff2 = self.get_part(Part::Iff2).borrow().to_utf8_string();
                let mut s: StrGrid = include_str!("../../displays/interrupt.utf8.template")
                    .try_into()
                    .unwrap();
                s.put(1, &il1).expect("il1 fits into interruptlogic");
                s.put(2, &iff2).expect("iff2 fits into interruptlogic");
                s.put(3, &il2).expect("il2 fits into interruptlogic");
                s.put(4, &iff1).expect("iff1 fits into interruptlogic");
                s.to_string()
            }
            _ => self
                .get_part(self.displaying_part)
                .borrow()
                .to_utf8_string(),
        }
        .lines()
        .take(area.height as usize)
        .for_each(|line| {
            x = area.x;
            line.char_indices()
                .take(area.width as usize)
                .for_each(|(_, c)| {
                    let style = match c {
                        '○' => Style::default().fg(Color::Gray),
                        '●' => Style::default().fg(Color::Yellow),
                        _ => Style::default(),
                    };
                    buf.set_string(x, y, c.to_string(), style);
                    x += 1;
                });
            y += 1;
        });
    }
}
