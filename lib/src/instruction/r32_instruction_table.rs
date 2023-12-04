use super::{
    decoder,
    funct3::{branch, load, op, op_imm, store},
    op_args::OpArgs,
    opcodes,
    util::C_5_BITS,
};
use crate::memory::{Memory, MemoryError};
use std::time::SystemTime;

const INSTRUCTION_SIZE: u32 = 4;
const FUNCT7_SWITCH: u8 = 0b0100000;

type CpuState = crate::cpu_r32i::CpuState<u32, 32>;

fn trap_opcode(op: &OpArgs) {
    let instruction = op.instruction;
    let state = &op.state;
    panic!("Illegal opcode trap when handling instruction {instruction:032b} {state:?}")
}

fn trap_memory_access(address: u32, op: &OpArgs) {
    let instruction = op.instruction;
    let state = &op.state;
    panic!("Illegal memory access trap when accessing address {instruction:032b} {address:032b} {state:?}")
}

fn apply_op<F: Fn(i32, i32) -> i32>(op: &mut OpArgs, f: F) {
    let source_register1 = op.rs1();
    let source_register2 = op.rs2();
    let destination_register = op.rd();
    let source_value1 = op.state.registers.geti(source_register1);
    let source_value2 = op.state.registers.geti(source_register2);
    let new_value = f(source_value1, source_value2);
    op.state
        .registers
        .set(destination_register, new_value as u32);
}

fn apply_op_unsigned<F: Fn(u32, u32) -> u32>(op: &mut OpArgs, f: F) {
    apply_op(op, |r1, r2| f(r1 as u32, r2 as u32) as i32)
}

fn apply_op_with_funct7_switch<F1: Fn(i32, i32) -> i32, F2: Fn(i32, i32) -> i32>(
    op: &mut OpArgs,
    f_switch: F1,
    f_no_switch: F2,
) {
    let funct7 = op.funct7();
    apply_op(op, |r1, r2| {
        if funct7 == FUNCT7_SWITCH {
            f_switch(r1, r2)
        } else if funct7 == 0 {
            f_no_switch(r1, r2)
        } else {
            panic!("funct7 must be zero or FUNCT7_SWITCH")
        }
    })
}

fn apply_op_imm<F: Fn(i32, i32) -> i32>(op: &mut OpArgs, f: F) {
    let source_register = op.rs1();
    let destination_register = op.rd();
    let immediate = op.i_imm();
    let source_value = op.state.registers.get(source_register) as i32;
    let new_value = f(source_value, immediate);
    op.state
        .registers
        .set(destination_register, new_value as u32);
}

fn apply_op_imm_unsigned<F: Fn(u32, u32) -> u32>(op: &mut OpArgs, f: F) {
    apply_op_imm(op, |r, i| f(r as u32, i as u32) as i32)
}

fn apply_op_imm_unsigned_truncated_to_5_bits<F: Fn(u32, u32) -> u32>(op: &mut OpArgs, f: F) {
    apply_op_imm_unsigned(op, |r, i| f(r, i & C_5_BITS))
}

fn apply_op_imm_funct7<F1: Fn(u32, u32) -> u32, F2: Fn(u32, u32) -> u32>(
    op: &mut OpArgs,
    f_switch: F1,
    f_no_switch: F2,
) {
    let funct7 = op.funct7();
    apply_op_imm_unsigned_truncated_to_5_bits(op, |r, i| {
        if funct7 == FUNCT7_SWITCH {
            f_switch(r, i)
        } else if funct7 == 0 {
            f_no_switch(r, i)
        } else {
            panic!("funct7 must be zero or FUNCT7_SWITCH")
        }
    })
}

/// A series of instructions that operate on a source register and an I-type (12-bit) immediate,
/// placing the result in rd.
fn op_imm(op: &mut OpArgs) {
    match op.funct3() {
        op_imm::ADDI => apply_op_imm(op, |r, i| r + i),
        op_imm::SLTI => apply_op_imm(op, |r, i| i32::from(r < i)),
        op_imm::SLTIU =>
        // This is the same as SLTI but the immediate is sign extended and then treated as an
        // unsigned and the comparison is done as an unsigned.
        {
            apply_op_imm_unsigned(op, |r, i| u32::from(r < i))
        }
        op_imm::ANDI => apply_op_imm(op, |r, i| r & i),
        op_imm::ORI => apply_op_imm(op, |r, i| r | i),
        op_imm::XORI => apply_op_imm(op, |r, i| r ^ i),
        op_imm::SLLI => {
            apply_op_imm_funct7(op, |_r, _i| panic!("SLL mode not zero"), |r, i| r << i)
        }
        op_imm::SRLI_OR_SRAI => {
            apply_op_imm_funct7(op, |r, i| ((r as i32) >> i) as u32, |r, i| r >> i)
        }
        8..=u8::MAX => panic!("funct3 parameter should not be > 0b111. This is an emulation bug."),
    };

    op.state.registers.pc += INSTRUCTION_SIZE;
}

/// A series of instructions that operate on two source registers, placing the result in rd.
fn op(op: &mut OpArgs) {
    match op.funct3() {
        op::ADD_OR_SUB => apply_op_with_funct7_switch(op, |r1, r2| r1 - r2, |r1, r2| r1 + r2),
        op::SLT => apply_op(op, |r1, r2| i32::from(r1 < r2)),
        op::SLTU =>
        // This is the same as SLTI but the immediate is sign extended and then treated as an
        // unsigned and the comparison is done as an unsigned.
        {
            apply_op_unsigned(op, |r1, r2| u32::from(r1 < r2))
        }
        op::AND => apply_op(op, |r1, r2| r1 & r2),
        op::OR => apply_op(op, |r, i| r | i),
        op::XOR => apply_op(op, |r, i| r ^ i),
        op::SLL => apply_op(op, |r1, r2| ((r1 as u32) << (r2 as u32)) as i32),
        op::SRL_OR_SRA => apply_op_with_funct7_switch(
            op,
            |r1, r2| ((r1 as u32) >> (r2 as u32)) as i32,
            |r1, r2| r1 >> (r2 as u32),
        ),
        8..=u8::MAX => panic!("funct3 parameter should not be > 0b111. This is an emulation bug."),
    };

    op.state.registers.pc += INSTRUCTION_SIZE;
}

/// Apply the branch instruction. All branch instructions take 2 registers and either advance
/// the PC normally or jump to PC + a b-type coded immediate depending on the result.
fn apply_branch<F: Fn(i32, i32) -> bool>(op: &mut OpArgs, f: F) {
    let source_one = op.rs1();
    let source_two = op.rs2();
    let offset = op.b_imm();

    if f(
        op.state.registers.geti(source_one),
        op.state.registers.geti(source_two),
    ) {
        op.state.registers.pc = ((op.state.registers.pc as i32) + offset) as u32;
    } else {
        op.state.registers.pc += INSTRUCTION_SIZE;
    }
}

/// Identical to apply_branch but comparisons are done on the unsigned interpretation of the
/// registers
fn apply_branch_unsigned<F: Fn(u32, u32) -> bool>(op: &mut OpArgs, f: F) {
    apply_branch(op, |r1, r2| f(r1 as u32, r2 as u32))
}

/// A series of instructions that operate on two source registers, jumping to PC + a B-type immediate
/// offset if a condition is met, otherwise advancing the program counter normally.
fn branch(op: &mut OpArgs) {
    match op.funct3() {
        branch::BEQ => apply_branch(op, |r1, r2| r1 == r2),
        branch::BNE => apply_branch(op, |r1, r2| r1 != r2),
        branch::BLT => apply_branch(op, |r1, r2| r1 < r2),
        branch::BGE => apply_branch(op, |r1, r2| r1 >= r2),
        branch::BLTU => apply_branch_unsigned(op, |r1, r2| r1 < r2),
        branch::BGEU => apply_branch_unsigned(op, |r1, r2| r1 >= r2),
        2 | 3 | 8..=u8::MAX => {
            panic!("funct3 parameter should not be > 0b111. This is an emulation bug.")
        }
    };

    op.state.registers.pc += INSTRUCTION_SIZE;
}

/// Apply the load function. This computes the address of the load and then passes the addres to a
/// custom F that applies the funct3 specific logic. The return is then written to rd.
fn apply_load<F: Fn(u32, &Memory) -> Result<i32, MemoryError>>(op: &mut OpArgs, f: F) {
    let source = op.rs1();
    let offset = op.i_imm();
    let destination = op.rd();
    let source_address = (op.state.registers.geti(source) + offset) as u32;
    let result = f(source_address, op.memory);

    match result {
        Ok(result) => {
            op.state.registers.seti(destination, result);
            op.state.registers.pc += INSTRUCTION_SIZE
        }
        Err(_) => trap_memory_access(source_address, op),
    }
}

fn apply_load_unsigned<F: Fn(u32, &Memory) -> Result<u32, MemoryError>>(op: &mut OpArgs, f: F) {
    apply_load(op, |address, memory| Ok(f(address, memory)? as i32))
}

/// Loads copy the value at (rs1 + S-type signed immediate) to rd. The standard loads are
/// sign-extended while the LBU and LHU variants are not.
fn load(op: &mut OpArgs) {
    match op.funct3() {
        load::LB => apply_load(op, |address, memory| {
            let raw_memory = memory.get8(address as usize)? as i8;
            // Rust will sign-extend casts from signed types
            let sign_extended = raw_memory as i32;
            Ok(sign_extended)
        }),
        load::LH => apply_load(op, |address, memory| {
            let raw_memory = memory.get16(address as usize)? as i16;
            // Rust will sign-extend casts from signed types
            let sign_extended = raw_memory as i32;
            Ok(sign_extended)
        }),
        load::LW => apply_load(op, |address, memory| {
            Ok(memory.get32(address as usize)? as i32)
        }),
        load::LBU => apply_load_unsigned(op, |address, memory| {
            Ok(memory.get8(address as usize)? as u32)
        }),
        load::LHU => apply_load_unsigned(op, |address, memory| {
            Ok(memory.get16(address as usize)? as u32)
        }),
        3 | 6..=u8::MAX => {
            panic!("funct3 parameter should not be 0b011 or > 0b101. This could be an emulation bug or a bug in the opcode.")
        }
    }
}

/// Apply the store function. Stores place whatever is in rs2 into the address [rs1 + S-type signed
/// immediate]. In this function we compute the destination address and grab the value in the
/// source register, then hand off to a user supplied f to apply the funct3 behaviour.
fn apply_store<F: Fn(u32, u32, &mut Memory) -> Result<(), MemoryError>>(op: &mut OpArgs, f: F) {
    let destination = op.rs1();
    let offset = op.s_imm();
    let destination_address = (op.state.registers.geti(destination) + offset) as u32;
    let source_value = op.state.registers.get(op.rs2());
    match f(destination_address, source_value, op.memory) {
        Ok(()) => op.state.registers.pc += INSTRUCTION_SIZE,
        Err(_) => trap_memory_access(destination_address, op),
    }
}

fn store(op: &mut OpArgs) {
    match op.funct3() {
        store::SB => apply_store(op, |destination, val, memory| {
            memory.set8(destination as usize, val as u8)
        }),
        store::SH => apply_store(op, |destination, val, memory| {
            memory.set16(destination as usize, val as u16)
        }),
        store::SW => apply_store(op, |destination, val, memory| {
            memory.set32(destination as usize, val as u32)
        }),
        3..=u8::MAX => {
            panic!("funct3 parameter should not be > 0b011. This could be an emulation bug or a bug in the opcode.")
        }
    }
}

/// Load upper immediate (Places a u-type immediate containing the upper 20 bits of a 32-bit value
/// into rd. All other bits are set to zero)
fn lui(op: &mut OpArgs) {
    let destination_register = op.rd();
    let immediate = op.u_imm();
    op.state
        .registers
        .set(destination_register, immediate as u32);
    op.state.registers.pc += INSTRUCTION_SIZE;
}

/// Add upper immediate to PC. Similar to LUI but adds the loaded immediate to current the program counter
/// and places it in RD. This can be used to compute addresses for JALR instructions.
fn auipc(op: &mut OpArgs) {
    let destination_register = op.rd();
    let immediate = op.u_imm();
    op.state.registers.set(
        destination_register,
        op.state.registers.pc + (immediate as u32),
    );
    op.state.registers.pc += INSTRUCTION_SIZE;
    panic!("BUG: TODO: Handle unaligned jumps");
}

/// JAL (jump and link) adds the signed J-immediate value to the current PC after storing the
/// current PC + 4 in the destination register.
fn jal(op: &mut OpArgs) {
    let destination_register = op.rd();
    let imm_value = op.j_imm();
    let new_pc = ((op.state.registers.pc as i32) + imm_value) as u32;

    op.state
        .registers
        .set(destination_register, op.state.registers.pc + 4);
    op.state.registers.pc = new_pc;
    panic!("BUG: TODO: Handle unaligned jumps");
}

/// JALR (Indirect jump) adds a 12-bit signed immediate to whatever is at rs1, sets the LSB of that
/// result to zero (e.g, result = result & (!1)), and finally sets the PC to this new result.
/// rd is set to the original PC + 4 (the start of the next instruction). Regiser 0 can be used to
/// discard the result.
fn jalr(op: &mut OpArgs) {
    let source_register = op.rs1();
    let destination_register = op.rd();
    let source_value = op.state.registers.geti(source_register);
    let imm_value = op.i_imm();
    let new_pc = (source_value + imm_value) as u32;
    let new_pc = new_pc & !1;
    op.state
        .registers
        .set(destination_register, op.state.registers.pc + 4);
    op.state.registers.pc = new_pc;
}

fn fence(op: &mut OpArgs) {
    // Fence is implement as a no-op as we only execute a single hart and do not pre-cache
    // instruction implementations.
    op.state.registers.pc += INSTRUCTION_SIZE;
}

pub struct InstructionTable {}

impl InstructionTable {
    pub const fn new() -> Self {
        Self {}
    }

    pub fn step(&self, cpu_state: &mut CpuState, memory: &mut Memory, instruction: u32) {
        cpu_state.registers.rdtime = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let op_arg = &mut OpArgs {
            state: cpu_state,
            memory: memory,
            instruction: instruction,
        };

        match decoder::opcode(instruction) {
            opcodes::OP => op(op_arg),
            opcodes::OP_IMM => op_imm(op_arg),
            opcodes::JAL => jal(op_arg),
            opcodes::JALR => jalr(op_arg),
            opcodes::LUI => lui(op_arg),
            opcodes::AUIPC => auipc(op_arg),
            opcodes::BRANCH => branch(op_arg),
            opcodes::LOAD => load(op_arg),
            opcodes::STORE => store(op_arg),
            opcodes::FENCE => fence(op_arg),
            _ => trap_opcode(op_arg),
        }

        cpu_state.registers.rdcycle += 1;
        cpu_state.registers.rdinstret += 1;
    }
}

#[cfg(test)]
mod test {
    use super::super::encoder::{self, Instruction};
    use super::*;

    const fn pack_negative_into_12b(val: i16) -> u16 {
        if val < -2048 || val > 2047 {
            panic!("12b signed value is out of range");
        }

        (val as u16) & 0b0000_1111_1111_1111
    }

    #[test]
    fn create_table() {
        let _table = InstructionTable::new();
    }

    struct TestEnvironment {
        state: CpuState,
        memory: Memory,
        tbl: InstructionTable,
        last_pc: u32,
    }

    impl TestEnvironment {
        fn new() -> Self {
            TestEnvironment {
                state: CpuState::new(),
                memory: Memory::new(4096),
                tbl: InstructionTable::new(),
                last_pc: 0,
            }
        }

        fn step(&mut self, instruction: &Instruction) {
            self.last_pc = self.state.registers.pc;
            self.tbl
                .step(&mut self.state, &mut self.memory, instruction.encode());
        }

        /// Step function that applys some default checks
        fn dbg_step(&mut self, instruction: &Instruction) {
            self.step(instruction);
            assert_eq!(self.last_pc + 4, self.state.registers.pc);
        }

        fn set_register(&mut self, index: usize, value: i32) {
            self.state.registers.seti(index, value);
        }

        fn expect_register(&self, index: usize, value: i32) {
            assert_eq!(self.state.registers.geti(index), value)
        }

        fn expect_all_register(&self, value: i32) {
            for i in 0..32 {
                self.expect_register(i, value);
            }
        }
    }

    fn init() -> TestEnvironment {
        TestEnvironment::new()
    }

    #[test]
    fn execute_no_op() {
        let mut test = init();
        test.dbg_step(&encoder::no_op());
        test.expect_all_register(0);
    }

    #[test]
    fn execute_addi() {
        let mut test = init();

        // Test INCR r1
        test.dbg_step(&encoder::addi(1, 1, 1));
        test.expect_register(1, 1);
        test.dbg_step(&encoder::addi(1, 1, 1));
        test.expect_register(1, 2);
        test.dbg_step(&encoder::addi(1, 1, 1));
        test.expect_register(1, 3);

        // Test r2 = r1 + 1
        test.dbg_step(&encoder::addi(2, 1, 1));
        test.expect_register(2, 4);
        test.expect_register(1, 3);

        // Test r2 = r1 - 1
        test.dbg_step(&encoder::addi(2, 1, pack_negative_into_12b(-1)));
        test.expect_register(1, 3);
        test.expect_register(2, 2);

        // Test r3 = r2 + 4
        test.dbg_step(&encoder::addi(3, 2, pack_negative_into_12b(4)));
        test.expect_register(1, 3);
        test.expect_register(2, 2);
        test.expect_register(3, 6);
    }

    #[test]
    fn execute_addi_overflow() {
        unimplemented!();
    }

    #[test]
    fn execute_slti() {
        let mut test = init();

        // Test positive
        test.set_register(1, 4);
        test.dbg_step(&encoder::slti(2, 1, 4));
        test.expect_register(1, 5);
        test.expect_register(2, 0);

        test.dbg_step(&encoder::slti(2, 1, 5));
        test.expect_register(1, 5);
        test.expect_register(2, 0);

        test.dbg_step(&encoder::slti(2, 1, 6));
        test.expect_register(1, 5);
        test.expect_register(2, 1);

        // Test max int
        test.set_register(1, 2047);
        test.dbg_step(&encoder::slti(2, 1, 2047));
        test.expect_register(2, 0);

        test.set_register(1, 2046);
        test.dbg_step(&encoder::slti(2, 1, 2047));
        test.expect_register(2, 1);

        // Test negative
        test.set_register(1, -5000);
        test.dbg_step(&encoder::slti(2, 1, pack_negative_into_12b(-2048)));
        test.expect_register(2, 1);

        test.set_register(1, -2000);
        test.dbg_step(&encoder::slti(2, 1, pack_negative_into_12b(-2048)));
        test.expect_register(2, 0);
    }

    #[test]
    fn execute_sltiu() {
        let mut test = init();

        test.set_register(1, 0);
        test.dbg_step(&encoder::sltiu(2, 1, 0));
        test.expect_register(2, 0);

        test.dbg_step(&encoder::sltiu(2, 1, 1));
        test.expect_register(2, 1);

        test.set_register(1, 2);
        test.dbg_step(&encoder::sltiu(2, 1, 1));
        test.expect_register(2, 0);

        test.set_register(1, 2048);
        test.dbg_step(&encoder::sltiu(2, 1, 2047));
        test.expect_register(2, 0);

        test.set_register(1, 2046);
        test.dbg_step(&encoder::sltiu(2, 1, 2047));
        test.expect_register(2, 1);
    }

    #[test]
    fn execute_andi() {
        let mut test = init();

        test.set_register(1, 0);
        test.dbg_step(&encoder::andi(2, 1, 1));
        test.expect_register(2, 0);

        test.set_register(1, 0b0110);
        test.dbg_step(&encoder::andi(2, 1, 0b0101));
        test.expect_register(2, 0b0100);
    }

    #[test]
    fn execute_ori() {
        let mut test = init();

        test.set_register(1, 0);
        test.dbg_step(&encoder::ori(2, 1, 1));
        test.expect_register(2, 1);

        test.set_register(1, 0b0110);
        test.dbg_step(&encoder::ori(2, 1, 0b0101));
        test.expect_register(2, 0b0111);
    }

    #[test]
    fn execute_xori() {
        let mut test = init();

        test.set_register(1, 0);
        test.dbg_step(&encoder::xori(2, 1, 1));
        test.expect_register(2, 1);

        test.set_register(1, 0b0110);
        test.dbg_step(&encoder::xori(2, 1, 0b0101));
        test.expect_register(2, 0b0011);
    }

    #[test]
    fn execute_slli() {
        let mut test = init();
        test.set_register(1, 0);
        test.dbg_step(&encoder::slli(2, 1, 1));
        test.expect_register(2, 0);
        test.set_register(1, 0b0110);
        test.dbg_step(&encoder::slli(2, 1, 1));
        test.expect_register(2, 0b1100);
        test.dbg_step(&encoder::slli(2, 1, 2));
        test.expect_register(2, 0b11000);
        test.dbg_step(&encoder::slli(2, 1, 3));
        test.expect_register(2, 0b110000);
        test.dbg_step(&encoder::slli(2, 1, 32));
        test.expect_register(2, 0b0);
        test.set_register(1, -1);
        test.dbg_step(&encoder::slli(2, 1, 16));
        test.expect_register(2, 0b1111_1111_1111_1111_0000_0000_0000_0000u32 as i32);
    }

    #[test]
    fn execute_srli() {
        let mut test = init();
        test.set_register(1, 0);
        test.dbg_step(&encoder::srli(2, 1, 1));
        test.expect_register(2, 0);
        test.set_register(1, 0b0110);
        test.dbg_step(&encoder::srli(2, 1, 1));
        test.expect_register(2, 0b0011);
        test.dbg_step(&encoder::srli(2, 1, 2));
        test.expect_register(2, 0b0001);
        test.dbg_step(&encoder::srli(2, 1, 3));
        test.expect_register(2, 0b0000);
        test.set_register(1, -1);
        test.dbg_step(&encoder::srli(2, 1, 16));
        test.expect_register(2, 0b0000_0000_0000_0000_1111_1111_1111_1111u32 as i32);
    }

    #[test]
    fn execute_srai() {
        let mut test = init();
        test.set_register(1, 0);
        test.dbg_step(&encoder::srai(2, 1, 1));
        test.expect_register(2, 0);
        test.set_register(1, 0b0110);
        test.dbg_step(&encoder::srai(2, 1, 1));
        test.expect_register(2, 0b0011);
        test.dbg_step(&encoder::srai(2, 1, 2));
        test.expect_register(2, 0b0001);
        test.dbg_step(&encoder::srai(2, 1, 3));
        test.expect_register(2, 0b0000);
        test.set_register(1, -1);
        test.dbg_step(&encoder::srai(2, 1, 16));
        test.expect_register(2, 0b1111_1111_1111_1111_1111_1111_1111_1111u32 as i32);
    }

    #[test]
    fn execute_add() {
        unimplemented!();
    }

    #[test]
    fn execute_add_overflow() {
        unimplemented!();
    }

    #[test]
    fn execute_sub() {
        unimplemented!();
    }

    #[test]
    fn execute_sub_underflow() {
        unimplemented!();
    }

    #[test]
    fn execute_slt() {
        unimplemented!();
    }

    #[test]
    fn execute_sltu() {
        unimplemented!();
    }

    #[test]
    fn execute_and() {
        unimplemented!();
    }

    #[test]
    fn execute_or() {
        unimplemented!();
    }

    #[test]
    fn execute_xor() {
        unimplemented!();
    }

    #[test]
    fn execute_sll() {
        unimplemented!();
    }

    #[test]
    fn execute_srl() {
        unimplemented!();
    }

    #[test]
    fn execute_sra() {
        unimplemented!();
    }

    #[test]
    fn execute_lui() {
        unimplemented!();
    }

    #[test]
    fn execute_auipc() {
        unimplemented!();
    }

    #[test]
    fn execute_jal() {
        unimplemented!();
    }

    #[test]
    fn execute_jalr() {
        unimplemented!();
    }

    #[test]
    fn execute_jalr_result_is_misaligned() {
        unimplemented!();
    }

    #[test]
    fn execute_beq() {
        unimplemented!();
    }

    #[test]
    fn execute_bne() {
        unimplemented!();
    }

    #[test]
    fn execute_blt() {
        unimplemented!();
    }

    #[test]
    fn execute_bge() {
        unimplemented!();
    }

    #[test]
    fn execute_bltu() {
        unimplemented!();
    }

    #[test]
    fn execute_bgeu() {
        unimplemented!();
    }

    #[test]
    fn execute_lb() {
        unimplemented!();
    }

    #[test]
    fn execute_lh() {
        unimplemented!();
    }

    #[test]
    fn execute_lw() {
        unimplemented!();
    }

    #[test]
    fn execute_lbu() {
        unimplemented!();
    }

    #[test]
    fn execute_lhu() {
        unimplemented!();
    }

    #[test]
    fn execute_sb() {
        unimplemented!();
    }

    #[test]
    fn execute_sh() {
        unimplemented!();
    }

    #[test]
    fn execute_sw() {
        unimplemented!();
    }

    #[test]
    fn execute_fence() {
        unimplemented!();
    }

    #[test]
    fn execute_fence_i() {
        unimplemented!();
    }
}
