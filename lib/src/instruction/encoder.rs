use super::funct3::op_imm::{ADDI, ANDI, ORI, SLLI, SLTI, SLTIU, SRLI_OR_SRAI, XORI};
use super::opcodes::OP_IMM;

const fn i_type_opcode(
    opcode: u8,
    destination_register: usize,
    source_register: usize,
    funct3: u8,
    imm: u16,
) -> u32 {
    if destination_register > 32
        || source_register > 32
        || funct3 > 0b111
        || opcode > 0b111111
        || imm > 0b1111_1111_1111
    {
        panic!("illegal operand");
    }

    let destination_register = destination_register as u32;
    let source_register = source_register as u32;

    (OP_IMM as u32)
        | (destination_register << 7)
        | ((funct3 as u32) << 12)
        | (source_register << 15)
        | (imm as u32) << 20
}

const fn op_opcode(
    opcode: u8,
    destination_register: usize,
    source_register1: usize,
    source_register2: usize,
    funct3: u8,
    funct7: u8,
) -> u32 {
    if destination_register > 32
        || source_register1 > 32
        || source_register2 > 32
        || funct3 > 0b111
        || funct7 > 0b1111111
    {
        panic!("illegal operand");
    }

    let destination_register = destination_register as u32;
    let source_register1 = source_register1 as u32;
    let source_register2 = source_register2 as u32;

    (OP_IMM as u32)
        | (destination_register << 7)
        | ((funct3 as u32) << 12)
        | (source_register1 << 15)
        | (source_register2 << 20)
        | ((funct7 as u32) << 25)
}

pub enum Instruction {
    OpImm {
        destination_register: usize,
        source_register: usize,
        funct3: u8,
        immediate: u16,
    },
    Op {
        destination_register: usize,
        source_register1: usize,
        source_register2: usize,
        funct3: u8,
        funct7: u8,
    },
}

impl Instruction {
    pub fn encode(&self) -> u32 {
        match self {
            &Instruction::OpImm {
                destination_register,
                source_register,
                funct3,
                immediate,
            } => i_type_opcode(
                OP_IMM as u8,
                destination_register,
                source_register,
                funct3,
                immediate,
            ),
            &Instruction::Op {
                destination_register,
                source_register1,
                source_register2,
                funct3,
                funct7,
            } => op_opcode(
                OP_IMM as u8,
                destination_register,
                source_register1,
                source_register2,
                funct3,
                funct7,
            ),
        }
    }
}

pub const fn op_imm(
    destination_register: usize,
    source_register: usize,
    funct3: u8,
    immediate: u16,
) -> Instruction {
    Instruction::OpImm {
        destination_register,
        source_register,
        funct3,
        immediate,
    }
}

/// Construct an add-immediate instruction that will add a signed 12-bit immediate
/// to the register in rs1 and then place it in rd
pub const fn addi(
    destination_register: usize,
    source_register: usize,
    immediate: u16,
) -> Instruction {
    op_imm(destination_register, source_register, ADDI, immediate)
}

/// Construct a set-less-than-immediate (SLTI) instruction that will set the destination
/// register to 1 if the register rs1 is < the sign extended immediate and set the destination
/// register to zero otherwise.
pub const fn slti(destination_register: usize, source_register: usize, imm: u16) -> Instruction {
    op_imm(destination_register, source_register, SLTI, imm)
}

/// Construct a set-less-than-immediate-unsigned (SLTIU) instruction that will set the destination
/// register to 1 if the register rs1 is < the sign extended immediate and set the destination
/// register to zero otherwise. In SLTIU the comparison is done treating the arguments as unsigned.
pub const fn sltiu(destination_register: usize, source_register: usize, imm: u16) -> Instruction {
    op_imm(destination_register, source_register, SLTIU, imm)
}

/// Construct an XORI (exclusive or immediate) which sets the destination register to the bitwise
/// XOR of the rs1 register with the sign extended immediate supplied.
pub const fn xori(destination_register: usize, source_register: usize, imm: u16) -> Instruction {
    op_imm(destination_register, source_register, XORI, imm)
}

/// Construct an ORI (or immediate) which sets the destination register to the bitwise
/// OR of the rs1 register with the sign extended immediate supplied.
pub const fn ori(destination_register: usize, source_register: usize, imm: u16) -> Instruction {
    op_imm(destination_register, source_register, ORI, imm)
}

/// Construct an ANDI (and immediate) which sets the destination register to the bitwise
/// AND of the rs1 register with the sign extended immediate supplied.
pub const fn andi(destination_register: usize, source_register: usize, imm: u16) -> Instruction {
    op_imm(destination_register, source_register, ANDI, imm)
}

/// Construct an SLLI (shift left by immediate) which sets the destination register to the bitwise
/// left shift of the rs1 register by the immediate specified.
pub const fn slli(destination_register: usize, source_register: usize, imm: u16) -> Instruction {
    if imm > 0b11111 {
        panic!("SRLI cannot have a shift immediate of greater than 0b11111");
    }

    op_imm(destination_register, source_register, SLLI, imm & 0b11111)
}

/// Construct an SRLI (shift left by immediate) which sets the destination register to the bitwise
/// right shift of the rs1 register by the immediate specified.
pub const fn srli(destination_register: usize, source_register: usize, imm: u16) -> Instruction {
    if imm > 0b11111 {
        panic!("SRLI cannot have a shift immediate of greater than 0b11111");
    }

    op_imm(
        destination_register,
        source_register,
        SRLI_OR_SRAI,
        imm & 0b11111,
    )
}

/// Construct an SRLI (shift left by immediate) which sets the destination register to the
/// arithmetic right shift of the rs1 register by the immediate specified.
pub const fn srai(destination_register: usize, source_register: usize, imm: u16) -> Instruction {
    if imm > 0b11111 {
        panic!("SRLI cannot have a shift immediate of greater than 0b11111");
    }

    op_imm(
        destination_register,
        source_register,
        SRLI_OR_SRAI,
        0b0100000_00000 | (imm & 0b11111),
    )
}

/// Construct a canonical no-op.
///
/// There are a few instructions that will cause no change except the PC to move forward, but the canonical encoding of a no-op is an ADDI with rd=0 rs1=0 and imm=0
pub const fn no_op() -> Instruction {
    addi(0, 0, 0)
}

#[cfg(test)]
mod test {
    use super::super::decoder::*;
    use super::*;

    fn test_op_imm(
        instruction: &Instruction,
        funct3_expected: u8,
        rd_expected: usize,
        rs1_expected: usize,
        imm: i32,
    ) {
        let example = instruction.encode();
        assert_eq!(opcode(example), OP_IMM);
        assert_eq!(funct3(example), funct3_expected);
        assert_eq!(rd(example), rd_expected);
        assert_eq!(rs1(example), rs1_expected);
        assert_eq!(i_type_immediate_32(example), imm as i32);
    }

    #[test]
    fn test_addi() {
        test_op_imm(&addi(0, 0, 0), ADDI, 0, 0, 0);
        test_op_imm(&addi(2, 4, 100), ADDI, 2, 4, 100);
    }

    #[test]
    fn test_slti() {
        test_op_imm(&slti(2, 4, 100), SLTI, 2, 4, 100);
    }

    #[test]
    fn test_sltiu() {
        test_op_imm(&sltiu(2, 4, 100), SLTIU, 2, 4, 100);
    }

    #[test]
    fn test_xori() {
        test_op_imm(&xori(2, 4, 100), XORI, 2, 4, 100);
    }

    #[test]
    fn test_ori() {
        test_op_imm(&ori(2, 4, 100), ORI, 2, 4, 100);
    }

    #[test]
    fn test_andi() {
        test_op_imm(&andi(2, 4, 100), ANDI, 2, 4, 100);
    }

    #[test]
    fn test_slli() {
        test_op_imm(&slli(2, 4, 3), SLLI, 2, 4, 0b000000000011);
    }

    #[test]
    fn test_srli() {
        test_op_imm(&srli(2, 4, 3), SRLI_OR_SRAI, 2, 4, 0b000000000011);
    }

    #[test]
    fn test_srai() {
        test_op_imm(&srai(2, 4, 3), SRLI_OR_SRAI, 2, 4, 0b010000000011);
    }
}
