use node::{Display, Node, Wire};

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use std::sync::mpsc::{channel as mpsc_channel, Receiver, Sender};

mod fns;
mod mp_ram;
mod types;

pub use fns::*;
pub use mp_ram::MPRam;
pub use types::*;

pub struct Machine {
    //pub al1: Rc<RefCell<Or2<'node, bool, bool, bool, dyn FnMut(&bool, &bool) -> bool + 'node>>>,
// pub al2: Rc<RefCell<Al2>>,
// pub al3: Rc<RefCell<Al3>>,
}

// impl<'node> Machine<'node>
// {
//
//     pub fn compose() -> Machine<'node> {
//         // Create all nodes
//         let (al1_n, al1) = Or2::new("AL1", make_or2());
//         let (al2_n, al2) = And2::new("AL2", make_and2());
//         let (al3_n, al3) = Xor2::new("AL3", make_xor2());
//         let (bl1_n, bl1) = And2::new("BL1", make_and2());
//         let (bl2_n, bl2) = And2::new("BL2", make_and2());
//         let (bl3_n, bl3) = Or2::new("BL3", make_or2());
//         let (br_n, br) = InstructionRegister::new("BR", make_instruction_register());
//         let (am1_n, am1) = Mux8x1::new("AM1", make_mux8x1());
//         let (am2_n, am2) = Mux4x1::new("AM2", make_mux4x1());
//         let (am3_n, am3) = Mux2x1::new("AM3", make_mux2x1());
//         let (am4_n, am4) = Mux2x1::new("AM4", make_mux2x1());
//         let (il1_n, il1) = And4::new("IL1", make_and4());
//         let (il2_n, il2) = Or2::new("IL2", make_or2());
//         let (iff1_n, iff1) = DFlipFlopC::new("IFF1", make_dflipflopc());
//         let (iff2_n, iff2) = DFlipFlop::new("IFF2", make_dflipflop());
//         let (mpram_n, mpram) = MPRam::new(make_mpram());
//         let (
//             mpff_n,
//             mrgaa3,
//             mrgaa2,
//             mrgaa1,
//             mrgaa0,
//             mrgab3,
//             mrgab2,
//             mrgab1,
//             mrgab0,
//             mchflg,
//             malus3,
//             malus2,
//             malus1,
//             malus0,
//             mrgwe,
//             mrgws,
//             maluia,
//             maluib,
//             mac3,
//             mac2,
//             mac1,
//             mac0,
//             na4,
//             na3,
//             na2,
//             na1,
//             na0,
//             busen,
//             buswr,
//         ) = MicroprogramFlipFlopC::new("MPFF", make_mpff());
//         let (mctr_n, ce, oe, we, wait) = MemoryController::new("MCTR", make_memory_controller());
//         let (rm1_n, rm1) = Mux2x1::new("RM1", make_mux2x1());
//         let (rm2_n, rm2) = Mux2x1::new("RM2", make_mux2x1());
//         let (rm3_n, rm3) = Mux2x1::new("RM3", make_mux2x1());
//         let (rm4_n, rm4) = Mux2x1::new("RM4", make_mux2x1());
//         let (rm5_n, rm5) = Mux2x1::new("RM5", make_mux2x1());
//         let (rm6_n, rm6) = Mux2x1::new("RM6", make_mux2x1());
//         let (dm1_n, dm1) = Mux2x1::new("DM1", make_mux2x1());
//         let (dm2_n, dm2) = Mux2x1::new("DM2", make_mux2x1());
//         let (reg_n, doa, cf, zf, nf, ief, dob) = Register::new("REG", make_register());
//         let (alu_n, co, zo, no, alu_out) =
//             ArithmeticLogicalUnit::new("ALU", make_arithmetic_logical_unit());
//         // Fake entry
//         let (fake_n, fake) = Fake::new(|| true);
//         // Compose everything
//         al1_n.borrow_mut().plug_in0(fake.clone()).plug_in1(iff1.clone());
//         al2_n.borrow_mut().plug_in0(ief.clone()).plug_in1(al1);
//         al3_n.borrow_mut().plug_in0(fake.clone()).plug_in1(am3);
//
//         Machine {
//             al1: al1_n,
//         }
//     }
// }

pub fn channel<'a, O>(id: &'static str) -> (Sender<O>, Wire<'a, O>)
where
    O: Clone + fmt::Debug + Default + 'a,
{
    let (sender, receiver): (Sender<O>, Receiver<O>) = mpsc_channel();
    let mut last = Default::default();
    let f = move || {
        while let Ok(value) = receiver.try_recv() {
            last = value;
        }
        last.clone()
    };
    let (_, out) = Input::new(id, f);
    (sender, out)
}
