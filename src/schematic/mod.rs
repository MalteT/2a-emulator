use node::{Display, Node, Wire};
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::widgets::Widget;

use std::cell::RefCell;
use std::rc::Rc;

mod fns;
mod mp_ram;
mod types;

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
}

#[derive(Debug)]
pub struct Machine<'node> {
    displaying_part: Part,
    last_cache_id: usize,
    al1_out: Wire<'node, bool>,
    clk: Rc<RefCell<Input<'node, bool>>>,
    al1: Rc<RefCell<Or2<'node, bool, bool, bool>>>,
    al2: Rc<RefCell<And2<'node, bool, bool, bool>>>,
    al3: Rc<RefCell<Xor2<'node, bool, bool, bool>>>,
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
        let (al2_n, _al2) = And2::new("AL2", make_and2());
        let (al3_n, _al3) = Xor2::new("AL3", make_xor2());
        let (_bl1_n, _bl1) = And2::new("BL1", make_and2());
        let (_bl2_n, _bl2) = And2::new("BL2", make_and2());
        let (_bl3_n, _bl3) = Or2::new("BL3", make_or2());
        let (_br_n, _br) = InstructionRegister::new("BR", make_instruction_register());
        let (_am1_n, _am1) = Mux8x1::new("AM1", make_mux8x1());
        let (_am2_n, _am2) = Mux4x1::new("AM2", make_mux4x1());
        let (_am3_n, am3) = Mux2x1::new("AM3", make_mux2x1());
        let (_am4_n, _am4) = Mux2x1::new("AM4", make_mux2x1());
        let (_il1_n, _il1) = And4::new("IL1", make_and4());
        let (_il2_n, _il2) = Or2::new("IL2", make_or2());
        let (_iff1_n, iff1) = DFlipFlopC::new("IFF1", make_dflipflopc());
        let (_iff2_n, _iff2) = DFlipFlop::new("IFF2", make_dflipflop());
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
        let (_reg_n, _doa, _cf, _zf, _nf, ief, _dob) = Register::new("REG", make_register());
        let (_alu_n, _co, _zo, _no, _alu_out) =
            ArithmeticLogicalUnit::new("ALU", make_arithmetic_logical_unit());
        // Clk
        let (clk_n, clk) = Input::with_name("CLK");
        // Fake entry
        let (_fake_n, fake) = Fake::new(|| true);
        // Compose everything
        al1_n
            .borrow_mut()
            .plug_in0(clk.clone())
            .plug_in1(fake.clone());
        al2_n.borrow_mut().plug_in0(ief.clone()).plug_in1(al1.clone());
        al3_n.borrow_mut().plug_in0(fake.clone()).plug_in1(am3);

        Machine {
            displaying_part: Part::Al1,
            last_cache_id: 0,
            clk: clk_n,
            al1_out: al1,
            al1: al1_n,
            al2: al2_n,
            al3: al3_n,
        }
    }
    /// Get a part from the machine.
    pub fn get_part(&self, part: Part) -> Rc<RefCell<dyn NodeDisplay + 'node>> {
        match part {
            Part::Al1 => self.al1.clone(),
            Part::Al2 => self.al2.clone(),
            Part::Al3 => self.al3.clone(),
        }
    }
    /// Send a clk signal to the machine.
    pub fn clk(&mut self, signal: bool) {
        self.clk.borrow_mut().set(signal);
        // TODO: Change this to be usefull.
        self.invalidate_cache();
        self.al1_out.get(self.last_cache_id);
    }
    /// Invalidate the current cache.
    /// Usually after an input changed.
    pub fn invalidate_cache(&mut self) {
        self.last_cache_id = (self.last_cache_id + 1) % usize::max_value();
    }
}

impl Widget for Machine<'_> {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let mut x = area.x;
        let mut y = area.y;
        self.get_part(self.displaying_part)
            .borrow()
            .to_utf8_string()
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
