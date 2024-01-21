use super::funct3::branch::{BEQ, BGE, BGEU, BLT, BLTU, BNE};
use super::funct3::load::{LB, LBU, LH, LHU, LW};
use super::funct3::op::{ADD_OR_SUB, AND, OR, SLL, SLT, SLTU, SRL_OR_SRA, XOR};
use super::funct3::op_imm::{ADDI, ANDI, ORI, SLLI, SLTI, SLTIU, SRLI_OR_SRAI, XORI};
use super::funct3::store::{SB, SH, SW};
use super::funct3::system::{CSRRW, ECALL_OR_EBREAK};
use super::opcodes::{AUIPC, BRANCH, FENCE, JAL, JALR, LOAD, LUI, OP, OP_IMM, STORE, SYSTEM};

const fn convert_i16_to_i12(value: i16) -> u16 {
    let negative = value < 0;
    let value = value as u16;

    if negative & (value & 0b1111_1000_0000_0000 != 0b1111_1000_0000_0000) {
        panic!("cannot convert a negative value to a i12 because it is out of range.")
    }

    if !negative & (value & 0b1111_0000_0000_0000 != 0) {
        panic!("cannot convert a positive value to an i12 because it is out of range");
    }

    value & 0b0000_1111_1111_1111
}

const fn i_type_opcode(
    opcode: u8,
    destination_register: usize,
    source_register: usize,
    funct3: u8,
    imm: u16,
) -> u32 {
    if destination_register > 32 || source_register > 32 {
        panic!("source or destination register out of range");
    }
    if funct3 > 0b111 {
        panic!("funct3 > 3 bits");
    }

    if opcode > 0b1111111 {
        panic!("opcode > 6 bits");
    }

    if imm > 0b1111_1111_1111 {
        panic!("immediate value out of range");
    }

    let destination_register = destination_register as u32;
    let source_register = source_register as u32;

    (opcode as u32)
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

    (opcode as u32)
        | (destination_register << 7)
        | ((funct3 as u32) << 12)
        | (source_register1 << 15)
        | (source_register2 << 20)
        | ((funct7 as u32) << 25)
}

const fn encode_lui(destination_register: usize, value: u32) -> u32 {
    if destination_register & !0b1111_1 != 0 {
        panic!("destination register is larger than 5 bits");
    }

    if value & 0b1111_1111_1111 != 0 {
        panic!("lower 12 bits of value in an LUI isntruction cannot be set");
    }

    value | ((destination_register as u32) << 7) | (LUI as u32)
}

const fn encode_auipc(destination_register: usize, value: u32) -> u32 {
    if destination_register & !0b1111_1 != 0 {
        panic!("destination register is larger than 5 bits");
    }

    if value & 0b1111_1111_1111 != 0 {
        panic!("lower 12 bits of value in an AUI isntruction cannot be set");
    }

    value | ((destination_register as u32) << 7) | (AUIPC as u32)
}

const fn encode_j_type_immediate(offset: i32) -> u32 {
    let negative = offset < 0;
    let value = offset as u32;

    // Negative values have all leading 1s, positive values have all leading zeros.
    if value & 0b1 != 0 {
        panic!("j-type immediate cannot have the lsb set");
    }

    if negative
        && ((value & 0b1111_1111_1110_0000_0000_0000_0000_0000u32)
            != 0b1111_1111_1110_0000_0000_0000_0000_0000u32)
    {
        panic!("negative j-type immediate must have all leading 1s");
    }

    if !negative && (value & 0b1111_1111_1110_0000_0000_0000_0000_0000u32) != 0 {
        panic!("j-type immediates must be 20 bytes and not have the lsb set");
    }

    let bit_twenty = value >> 31;
    let bit_eleven = (value & 0b0000_0000_0100_0000_0000) >> 10;
    let bits_ten_to_one = (value & 0b111_1111_1110) >> 1;
    let bits_19_to_12 = (value & 0b0111_1111_1000_0000_0000) >> 11;

    (bit_twenty << 31) | (bit_eleven << 20) | (bits_ten_to_one << 21) | (bits_19_to_12 << 12)
}

const fn encode_jal(address_offset: i32, destination_register: usize) -> u32 {
    if destination_register & !0b1111_1 != 0 {
        panic!("destination register is larger than 5 bits");
    }

    encode_j_type_immediate(address_offset) | ((destination_register as u32) << 7) | (JAL as u32)
}

// Encode a 13-bit signed immediate into it's B-type intstruction format.
// The LSB must be 0 because the 13 bits are packed into 12b
const fn encode_b_type_immediate(value: i16) -> u32 {
    let negative = value < 0;
    let value = value as u16;

    if value & 0b1 != 0 {
        panic!("LSB of a B-type immediate must not be set");
    }

    if negative && ((value & 0b1111_0000_0000_0000) != 0b1111_0000_0000_0000) {
        panic!("negative value lower than the minimum value for a B-type immediate");
    }

    if (!negative) && ((value & 0b1111_0000_0000_0000) != 0) {
        panic!("positive value too large for B-type immediate");
    }

    // The input is 13 bits compressed into 12 (assuming v & 1 == 0)
    let value = ((value & 0b1_1111_1111_1111) >> 1) as u32;

    let bit_twelve = (value & 0b1000_0000_0000) >> 11;
    let bit_eleven = (value & 0b0100_0000_0000) >> 10;
    let bits_ten_to_five = (value & 0b0011_1111_0000) >> 4;
    let lower_four_bits = value & 0b1111;

    (bit_twelve << 31) | (bit_eleven << 7) | (lower_four_bits << 8) | (bits_ten_to_five << 25)
}

const fn encode_branch(
    funct3: u8,
    source_register1: usize,
    source_register2: usize,
    branch_offset: i16,
) -> u32 {
    if source_register1 > 0b11111 {
        panic!("source register 1 outside of valid range");
    }

    if source_register2 > 0b11111 {
        panic!("source register 2 outside of valid range");
    }

    if funct3 > 0b111 {
        panic!("funct3 outside of valid range");
    }

    (BRANCH as u32)
        | ((funct3 as u32) << 12)
        | ((source_register1 as u32) << 15)
        | ((source_register2 as u32) << 20)
        | (encode_b_type_immediate(branch_offset))
}

const fn encode_load(
    funct3: u8,
    destination_register: usize,
    source_register: usize,
    offset: i16,
) -> u32 {
    if source_register > 0b11111 {
        panic!("source register outside of valid range");
    }

    if destination_register > 0b11111 {
        panic!("source register outside of valid range");
    }

    if funct3 > 0b111 {
        panic!("funct3 out of range");
    }

    let offset = convert_i16_to_i12(offset);

    (LOAD as u32)
        | ((destination_register as u32) << 7)
        | ((funct3 as u32) << 12)
        | ((source_register as u32) << 15)
        | ((offset as u32) << 20)
}

const fn encode_store(
    funct3: u8,
    source_register1: usize,
    source_register2: usize,
    offset: i16,
) -> u32 {
    if source_register1 > 0b11111 {
        panic!("source register outside of valid range");
    }

    if source_register2 > 0b11111 {
        panic!("source register 2 outside of valid range");
    }

    if funct3 > 0b111 {
        panic!("funct3 out of range");
    }

    let offset = convert_i16_to_i12(offset);
    let lower_5_bits = offset & 0b11111;
    let upper_7_bits = offset >> 5;

    (STORE as u32)
        | ((funct3 as u32) << 12)
        | ((source_register1 as u32) << 15)
        | ((source_register2 as u32) << 20)
        | ((lower_5_bits as u32) << 7)
        | ((upper_7_bits as u32) << 25)
}

const fn encode_fence(funct3: u8, pred: u8, succ: u8) -> u32 {
    if funct3 != 0 && funct3 != 1 {
        panic!("fence funct3 must be zero or 1");
    }

    if pred > 0b1111 {
        panic!("pred cannot be > 0b1111");
    }

    if succ > 0b1111 {
        panic!("succ cannot be > 0b1111");
    }

    (FENCE as u32) | ((funct3 as u32) << 7) | ((pred as u32) << 24) | ((succ as u32) << 20)
}

const fn encode_ecall() -> u32 {
    (SYSTEM as u32) | ((ECALL_OR_EBREAK as u32) << 7)
}

const fn encode_ebreak() -> u32 {
    (SYSTEM as u32) | ((ECALL_OR_EBREAK as u32) << 7) | (1 << 20)
}

const fn encode_csr(
    funct3: u8,
    csr: usize,
    destination_register: usize,
    source_register: usize,
) -> u32 {
    if destination_register > 0b11111 {
        panic!("dest register outside of valid range");
    }

    if source_register > 0b11111 {
        panic!("source register 1 outside of valid range");
    }

    if csr > 0b1111_1111_1111 {
        panic!("csr value out of range");
    }

    if funct3 > 0b111 {
        panic!("funct3 out of range");
    }

    (SYSTEM as u32)
        | ((destination_register as u32) << 7)
        | ((funct3 as u32) << 12)
        | ((source_register as u32) << 15)
        | ((csr as u32) << 20)
}

pub enum Instruction {
    OpImm {
        destination_register: usize,
        source_register: usize,
        funct3: u8,
        immediate: i16,
    },
    Op {
        destination_register: usize,
        source_register1: usize,
        source_register2: usize,
        funct3: u8,
        funct7: u8,
    },
    Lui {
        destination_register: usize,
        value: u32,
    },
    Auipc {
        destination_register: usize,
        value: u32,
    },
    Jal {
        destination_register: usize,
        address_offset: i32,
    },
    Jalr {
        destination_register: usize,
        source_register: usize,
        address_offset: i16,
    },
    Branch {
        funct3: u8,
        source_register1: usize,
        source_register2: usize,
        branch_offset: i16,
    },
    Store {
        funct3: u8,
        source_register1: usize,
        source_register2: usize,
        offset: i16,
    },
    Load {
        funct3: u8,
        source_register: usize,
        destination_register: usize,
        offset: i16,
    },
    Fence {
        pred: u8,
        succ: u8,
    },
    FenceI {},
    ECall,
    EBreak,
    CsrRw {
        source_register: usize,
        destination_register: usize,
        csr: usize,
    },
}

impl Instruction {
    pub const fn encode(&self) -> u32 {
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
                convert_i16_to_i12(immediate),
            ),
            &Instruction::Op {
                destination_register,
                source_register1,
                source_register2,
                funct3,
                funct7,
            } => op_opcode(
                OP as u8,
                destination_register,
                source_register1,
                source_register2,
                funct3,
                funct7,
            ),
            &Instruction::Lui {
                destination_register,
                value,
            } => encode_lui(destination_register, value),
            &Instruction::Auipc {
                destination_register,
                value,
            } => encode_auipc(destination_register, value),
            &Instruction::Jal {
                address_offset,
                destination_register,
            } => encode_jal(address_offset, destination_register),
            &Instruction::Jalr {
                destination_register,
                source_register,
                address_offset,
            } => i_type_opcode(
                JALR as u8,
                destination_register,
                source_register,
                0,
                convert_i16_to_i12(address_offset),
            ),
            &Instruction::Branch {
                funct3,
                source_register1,
                source_register2,
                branch_offset,
            } => encode_branch(funct3, source_register1, source_register2, branch_offset),
            &Instruction::Store {
                funct3,
                source_register1,
                source_register2,
                offset,
            } => encode_store(funct3, source_register1, source_register2, offset),
            &Instruction::Load {
                funct3,
                source_register,
                destination_register,
                offset,
            } => encode_load(funct3, destination_register, source_register, offset),
            &Instruction::Fence { pred, succ } => encode_fence(0, pred, succ),
            &Instruction::FenceI {} => encode_fence(1, 0, 0),
            &Instruction::ECall {} => encode_ecall(),
            &Instruction::EBreak {} => encode_ebreak(),
            &Instruction::CsrRw {
                csr,
                source_register,
                destination_register,
            } => encode_csr(CSRRW, csr, destination_register, source_register),
        }
    }
}

pub const fn op_imm(
    destination_register: usize,
    source_register: usize,
    funct3: u8,
    immediate: i16,
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
    immediate: i16,
) -> Instruction {
    op_imm(destination_register, source_register, ADDI, immediate)
}

/// Construct a set-less-than-immediate (SLTI) instruction that will set the destination
/// register to 1 if the register rs1 is < the sign extended immediate and set the destination
/// register to zero otherwise.
pub const fn slti(destination_register: usize, source_register: usize, imm: i16) -> Instruction {
    op_imm(destination_register, source_register, SLTI, imm)
}

/// Construct a set-less-than-immediate-unsigned (SLTIU) instruction that will set the destination
/// register to 1 if the register rs1 is < the sign extended immediate and set the destination
/// register to zero otherwise. In SLTIU the comparison is done treating the arguments as unsigned.
pub const fn sltiu(destination_register: usize, source_register: usize, imm: i16) -> Instruction {
    op_imm(destination_register, source_register, SLTIU, imm)
}

/// Construct an XORI (exclusive or immediate) which sets the destination register to the bitwise
/// XOR of the rs1 register with the sign extended immediate supplied.
pub const fn xori(destination_register: usize, source_register: usize, imm: i16) -> Instruction {
    op_imm(destination_register, source_register, XORI, imm)
}

/// Construct an ORI (or immediate) which sets the destination register to the bitwise
/// OR of the rs1 register with the sign extended immediate supplied.
pub const fn ori(destination_register: usize, source_register: usize, imm: i16) -> Instruction {
    op_imm(destination_register, source_register, ORI, imm)
}

/// Construct an ANDI (and immediate) which sets the destination register to the bitwise
/// AND of the rs1 register with the sign extended immediate supplied.
pub const fn andi(destination_register: usize, source_register: usize, imm: i16) -> Instruction {
    op_imm(destination_register, source_register, ANDI, imm)
}

/// Construct an SLLI (shift left by immediate) which sets the destination register to the bitwise
/// left shift of the rs1 register by the immediate specified.
pub const fn slli(destination_register: usize, source_register: usize, imm: i16) -> Instruction {
    if imm > 0b11111 {
        panic!("SRLI cannot have a shift immediate of greater than 0b11111");
    }

    op_imm(destination_register, source_register, SLLI, imm & 0b11111)
}

/// Construct an SRLI (shift left by immediate) which sets the destination register to the bitwise
/// right shift of the rs1 register by the immediate specified.
pub const fn srli(destination_register: usize, source_register: usize, imm: i16) -> Instruction {
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
pub const fn srai(destination_register: usize, source_register: usize, imm: i16) -> Instruction {
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

/// Construct a three register (two sources and one destination) op instruction
pub const fn op(
    destination_register: usize,
    source_register1: usize,
    source_register2: usize,
    funct3: u8,
    funct7: u8,
) -> Instruction {
    Instruction::Op {
        destination_register,
        source_register1,
        source_register2,
        funct3,
        funct7,
    }
}

/// Construct an add instruction that will add rs1 and rs2 and place the result in rd
pub const fn add(
    destination_register: usize,
    source_register1: usize,
    source_register2: usize,
) -> Instruction {
    op(
        destination_register,
        source_register1,
        source_register2,
        ADD_OR_SUB,
        0,
    )
}

/// Construct a subtract instruction that will subtract rs2 from rs1 and place the result in rd
pub const fn sub(
    destination_register: usize,
    source_register1: usize,
    source_register2: usize,
) -> Instruction {
    op(
        destination_register,
        source_register1,
        source_register2,
        ADD_OR_SUB,
        0b0100000,
    )
}

/// Construct a set less than of two registers
pub const fn slt(
    destination_register: usize,
    source_register1: usize,
    source_register2: usize,
) -> Instruction {
    op(
        destination_register,
        source_register1,
        source_register2,
        SLT,
        0b0,
    )
}

/// Construct a set less than unsigned of two registers
pub const fn sltu(
    destination_register: usize,
    source_register1: usize,
    source_register2: usize,
) -> Instruction {
    op(
        destination_register,
        source_register1,
        source_register2,
        SLTU,
        0b0,
    )
}

/// Construct a bitwise and of two registers
pub const fn and(
    destination_register: usize,
    source_register1: usize,
    source_register2: usize,
) -> Instruction {
    op(
        destination_register,
        source_register1,
        source_register2,
        AND,
        0b0,
    )
}

/// Construct a bitwise or of two registers
pub const fn or(
    destination_register: usize,
    source_register1: usize,
    source_register2: usize,
) -> Instruction {
    op(
        destination_register,
        source_register1,
        source_register2,
        OR,
        0b0,
    )
}

/// Construct a bitwise xor of two registers
pub const fn xor(
    destination_register: usize,
    source_register1: usize,
    source_register2: usize,
) -> Instruction {
    op(
        destination_register,
        source_register1,
        source_register2,
        XOR,
        0b0,
    )
}

/// Construct a logical shift left of rs1 by rs2
pub const fn sll(
    destination_register: usize,
    source_register1: usize,
    source_register2: usize,
) -> Instruction {
    op(
        destination_register,
        source_register1,
        source_register2,
        SLL,
        0b0,
    )
}

/// Construct a logical shift right of rs1 by rs2
pub const fn srl(
    destination_register: usize,
    source_register1: usize,
    source_register2: usize,
) -> Instruction {
    op(
        destination_register,
        source_register1,
        source_register2,
        SRL_OR_SRA,
        0b0,
    )
}

/// Construct an arithmetic shift right of rs1 by rs2
pub const fn sra(
    destination_register: usize,
    source_register1: usize,
    source_register2: usize,
) -> Instruction {
    op(
        destination_register,
        source_register1,
        source_register2,
        SRL_OR_SRA,
        0b0100000,
    )
}

/// Construct a load-upper-immediate (set destination register to value, where value cannot have
/// its lower 12 bits set).
pub const fn lui(destination_register: usize, value: u32) -> Instruction {
    Instruction::Lui {
        destination_register,
        value,
    }
}

/// Construct an add-upper-immediate-to-pc (set destination_register to value + pc where value
/// cannot have its lower 12 bits set).
pub const fn auipc(destination_register: usize, value: u32) -> Instruction {
    Instruction::Auipc {
        destination_register,
        value,
    }
}

/// Construct a jump and link instruction (set pc to pc + signed 20 bit address_offset. set rd to
/// old pc + 4)
pub const fn jal(destination_register: usize, address_offset: i32) -> Instruction {
    Instruction::Jal {
        address_offset,
        destination_register,
    }
}

/// Construct a jump and link register instruction (set pc to rs1 + signed 12 bit address_offset.
/// set rd to old pc + 4)
pub const fn jalr(
    destination_register: usize,
    source_register: usize,
    address_offset: i16,
) -> Instruction {
    Instruction::Jalr {
        address_offset,
        destination_register,
        source_register,
    }
}

/// Construct a branch-equal operation that will branch to pc + (signed 13-bit branch offset) if
/// rs1 and rs2 are equal. Otherwise it proceeds to the next instruction.
pub const fn beq(
    source_register1: usize,
    source_register2: usize,
    branch_offset: i16,
) -> Instruction {
    Instruction::Branch {
        funct3: BEQ as u8,
        source_register1,
        source_register2,
        branch_offset,
    }
}

/// Construct a branch-not-equal operation that will branch to pc + (signed 13-bit branch offset) if
/// rs1 and rs2 are not equal. Otherwise it proceeds to the next instruction.
pub const fn bne(
    source_register1: usize,
    source_register2: usize,
    branch_offset: i16,
) -> Instruction {
    Instruction::Branch {
        funct3: BNE as u8,
        source_register1,
        source_register2,
        branch_offset,
    }
}

/// Construct a branch-less-than operation that will branch to pc + (signed 13-bit branch offset) if
/// rs1 is less than rs2. Otherwise it proceeds to the next instruction.
pub const fn blt(
    source_register1: usize,
    source_register2: usize,
    branch_offset: i16,
) -> Instruction {
    Instruction::Branch {
        funct3: BLT as u8,
        source_register1,
        source_register2,
        branch_offset,
    }
}

/// Construct a branch-greater-than-or-equal operation that will branch to pc + (signed 13-bit branch offset) if
/// rs1 is greater than or equal to rs2. Otherwise it proceeds to the next instruction.
pub const fn bge(
    source_register1: usize,
    source_register2: usize,
    branch_offset: i16,
) -> Instruction {
    Instruction::Branch {
        funct3: BGE as u8,
        source_register1,
        source_register2,
        branch_offset,
    }
}

/// Construct a branch-less-than-unsigned operation that will branch to pc + (signed 13-bit branch offset) if
/// rs1 is less than rs2 (unsigned). Otherwise it proceeds to the next instruction.
pub const fn bltu(
    source_register1: usize,
    source_register2: usize,
    branch_offset: i16,
) -> Instruction {
    Instruction::Branch {
        funct3: BLTU as u8,
        source_register1,
        source_register2,
        branch_offset,
    }
}

/// Construct a branch-greater-than-or-equal-unsigned operation that will branch to pc + (signed 13-bit branch offset) if
/// rs1 is greater than or equal to rs2 (unsigned). Otherwise it proceeds to the next instruction.
pub const fn bgeu(
    source_register1: usize,
    source_register2: usize,
    branch_offset: i16,
) -> Instruction {
    Instruction::Branch {
        funct3: BGEU as u8,
        source_register1,
        source_register2,
        branch_offset,
    }
}

/// Construct a load-byte operation that will load a byte from [source_register + offset]
/// and place it in the destination register.
pub const fn lb(source_register: usize, destination_register: usize, offset: i16) -> Instruction {
    Instruction::Load {
        funct3: LB as u8,
        source_register,
        destination_register,
        offset,
    }
}

/// Construct a load-half-word operation that will load a short from [source_register + offset]
/// and place it in the destination register.
pub const fn lh(source_register: usize, destination_register: usize, offset: i16) -> Instruction {
    Instruction::Load {
        funct3: LH as u8,
        source_register,
        destination_register,
        offset,
    }
}

/// Construct a load-byte-unsigned operation that will load a byte from [source_register + offset]
/// and place it in the destination register. The byte will not be sign extended.
pub const fn lbu(source_register: usize, destination_register: usize, offset: i16) -> Instruction {
    Instruction::Load {
        funct3: LBU as u8,
        source_register,
        destination_register,
        offset,
    }
}

/// Construct a load-half-word operation that will load a short from [source_register + offset]
/// and place it in the destination register. The half will not be sign extended.
pub const fn lhu(source_register: usize, destination_register: usize, offset: i16) -> Instruction {
    Instruction::Load {
        funct3: LHU as u8,
        source_register,
        destination_register,
        offset,
    }
}

/// Construct a load-word operation that will load a short from [source_register + offset]
/// and place it in the destination register.
pub const fn lw(source_register: usize, destination_register: usize, offset: i16) -> Instruction {
    Instruction::Load {
        funct3: LW as u8,
        source_register,
        destination_register,
        offset,
    }
}

/// Construct a store-byte operation that will store a byte from source_register2 at [source_register1 + offset]
/// and place it in the destination register.
pub const fn sb(source_register1: usize, source_register2: usize, offset: i16) -> Instruction {
    Instruction::Store {
        funct3: SB as u8,
        source_register1,
        source_register2,
        offset,
    }
}

/// Construct a store-half operation that will store a byte from source_register2 at [source_register1 + offset]
/// and place it in the destination register.
pub const fn sh(source_register1: usize, source_register2: usize, offset: i16) -> Instruction {
    Instruction::Store {
        funct3: SH as u8,
        source_register1,
        source_register2,
        offset,
    }
}

/// Construct a store-half operation that will store a byte from source_register2 at [source_register1 + offset]
/// and place it in the destination register.
pub const fn sw(source_register1: usize, source_register2: usize, offset: i16) -> Instruction {
    Instruction::Store {
        funct3: SW as u8,
        source_register1,
        source_register2,
        offset,
    }
}

/// Construct a canonical no-op.
///
/// There are a few instructions that will cause no change except the PC to move forward, but the canonical encoding of a no-op is an ADDI with rd=0 rs1=0 and imm=0
pub const fn no_op() -> Instruction {
    addi(0, 0, 0)
}

/// Construct a fence instruction. This is a no-op in our emulator.
pub const fn fence() -> Instruction {
    Instruction::Fence { pred: 0, succ: 0 }
}

/// Construct a fence.i operation. This is a no-op in our emulator.
pub const fn fence_i() -> Instruction {
    Instruction::FenceI {}
}

/// Construct a ecall operation. This is a chip specific environment call.
pub const fn ecall() -> Instruction {
    Instruction::ECall {}
}

/// Construct a ebreak operation. This is a chip specific debug breakpoint.
pub const fn ebreak() -> Instruction {
    Instruction::EBreak {}
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

    fn construct_test_branch(
        instruction: &Instruction,
        funct3_expected: u8,
        source_register1_expected: usize,
        source_register2_expected: usize,
        branch_offset_expected: i16,
    ) {
        let example = instruction.encode();
        assert_eq!(opcode(example), BRANCH);
        assert_eq!(funct3(example), funct3_expected);
        assert_eq!(rs1(example), source_register1_expected);
        assert_eq!(rs2(example), source_register2_expected);
        assert_eq!(b_type_immediate_32(example), branch_offset_expected as i32);
    }

    fn construct_test_load(
        instruction: &Instruction,
        funct3_expected: u8,
        source_register_expected: usize,
        dest_register_expected: usize,
        offset_expected: i16,
    ) {
        let example = instruction.encode();
        assert_eq!(opcode(example), LOAD);
        assert_eq!(funct3(example), funct3_expected);
        assert_eq!(rs1(example), source_register_expected);
        assert_eq!(rd(example), dest_register_expected);
        assert_eq!(i_type_immediate_32(example), offset_expected as i32);
    }

    fn construct_test_store(
        instruction: &Instruction,
        funct3_expected: u8,
        source_register_expected: usize,
        source_register_2_expected: usize,
        offset_expected: i16,
    ) {
        let example = instruction.encode();
        assert_eq!(opcode(example), STORE);
        assert_eq!(funct3(example), funct3_expected);
        assert_eq!(rs1(example), source_register_expected);
        assert_eq!(rs2(example), source_register_2_expected);
        assert_eq!(s_type_immediate_32(example), offset_expected as i32);
    }

    fn construct_test_jalr(
        instruction: &Instruction,
        rd_expected: usize,
        rs1_expected: usize,
        imm: i32,
    ) {
        let example = instruction.encode();
        assert_eq!(opcode(example), JALR);
        assert_eq!(funct3(example), 0);
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

    fn test_op(
        instruction: &Instruction,
        rd_expected: usize,
        rs1_expected: usize,
        rs2_expected: usize,
        funct3_expected: u8,
        funct7_expected: u8,
    ) {
        let example = instruction.encode();
        assert_eq!(opcode(example), OP);
        assert_eq!(funct3(example), funct3_expected);
        assert_eq!(funct7(example), funct7_expected);
        assert_eq!(rd(example), rd_expected);
        assert_eq!(rs1(example), rs1_expected);
        assert_eq!(rs2(example), rs2_expected);
    }

    #[test]
    fn test_add() {
        test_op(&add(0, 0, 0), 0, 0, 0, ADD_OR_SUB, 0);
        test_op(&add(2, 4, 3), 2, 4, 3, ADD_OR_SUB, 0);
    }

    #[test]
    fn test_sub() {
        test_op(&sub(0, 0, 0), 0, 0, 0, ADD_OR_SUB, 0b0100000);
        test_op(&sub(2, 4, 3), 2, 4, 3, ADD_OR_SUB, 0b0100000);
    }

    #[test]
    fn test_slt() {
        test_op(&slt(0, 0, 0), 0, 0, 0, SLT, 0b0);
        test_op(&slt(2, 4, 3), 2, 4, 3, SLT, 0b0);
    }

    #[test]
    fn test_sltu() {
        test_op(&sltu(0, 0, 0), 0, 0, 0, SLTU, 0b0);
        test_op(&sltu(2, 4, 3), 2, 4, 3, SLTU, 0b0);
    }

    #[test]
    fn test_and() {
        test_op(&and(0, 0, 0), 0, 0, 0, AND, 0b0);
        test_op(&and(2, 4, 3), 2, 4, 3, AND, 0b0);
    }

    #[test]
    fn test_or() {
        test_op(&or(0, 0, 0), 0, 0, 0, OR, 0b0);
        test_op(&or(2, 4, 3), 2, 4, 3, OR, 0b0);
    }

    #[test]
    fn test_xor() {
        test_op(&xor(0, 0, 0), 0, 0, 0, XOR, 0b0);
        test_op(&xor(2, 4, 3), 2, 4, 3, XOR, 0b0);
    }

    #[test]
    fn test_sll() {
        test_op(&sll(0, 0, 0), 0, 0, 0, SLL, 0b0);
        test_op(&sll(2, 4, 3), 2, 4, 3, SLL, 0b0);
    }

    #[test]
    fn test_srl() {
        test_op(&srl(0, 0, 0), 0, 0, 0, SRL_OR_SRA, 0b0);
        test_op(&srl(2, 4, 3), 2, 4, 3, SRL_OR_SRA, 0b0);
    }

    #[test]
    fn test_sra() {
        test_op(&sra(0, 0, 0), 0, 0, 0, SRL_OR_SRA, 0b0100000);
        test_op(&sra(2, 4, 3), 2, 4, 3, SRL_OR_SRA, 0b0100000);
    }

    #[test]
    fn test_lui() {
        let value = 0b1101_1111_0101_1010_0101_0000_0000_0000u32;
        let op = lui(5, value).encode();
        assert_eq!(opcode(op), LUI);
        assert_eq!(rd(op), 5);
        assert_eq!(op >> 12, value >> 12);
    }

    #[test]
    fn test_auipc() {
        let value = 0b1101_1111_0101_1010_0101_0000_0000_0000u32;
        let op = auipc(5, value).encode();
        assert_eq!(opcode(op), AUIPC);
        assert_eq!(rd(op), 5);
        assert_eq!(op >> 12, value >> 12);
    }

    #[test]
    fn test_jal() {
        let op = jal(3, 500).encode();
        assert_eq!(opcode(op), JAL);
        assert_eq!(rd(op), 3);
        assert_eq!(j_type_immediate_32(op), 500);
        let op = jal(2, -500).encode();
        assert_eq!(opcode(op), JAL);
        assert_eq!(rd(op), 2);
        assert_eq!(j_type_immediate_32(op), -500);
    }

    #[test]
    fn test_jalr() {
        construct_test_jalr(&jalr(1, 2, 500), 1, 2, 500);
        construct_test_jalr(&jalr(1, 2, -500), 1, 2, -500);
    }

    #[test]
    fn test_beq() {
        construct_test_branch(&beq(1, 2, 500), BEQ as u8, 1, 2, 500);
        construct_test_branch(&beq(1, 2, -500), BEQ as u8, 1, 2, -500);
    }

    #[test]
    fn test_bne() {
        construct_test_branch(&bne(1, 2, 500), BNE as u8, 1, 2, 500);
        construct_test_branch(&bne(1, 2, -500), BNE as u8, 1, 2, -500);
    }

    #[test]
    fn test_blt() {
        construct_test_branch(&blt(1, 2, 500), BLT as u8, 1, 2, 500);
        construct_test_branch(&blt(1, 2, -500), BLT as u8, 1, 2, -500);
    }

    #[test]
    fn test_bge() {
        construct_test_branch(&bge(1, 2, 500), BGE as u8, 1, 2, 500);
        construct_test_branch(&bge(1, 2, -500), BGE as u8, 1, 2, -500);
    }

    #[test]
    fn test_bltu() {
        construct_test_branch(&bltu(1, 2, 500), BLTU as u8, 1, 2, 500);
        construct_test_branch(&bltu(1, 2, -500), BLTU as u8, 1, 2, -500);
    }

    #[test]
    fn test_bgeu() {
        construct_test_branch(&bgeu(1, 2, 500), BGEU as u8, 1, 2, 500);
        construct_test_branch(&bgeu(1, 2, -500), BGEU as u8, 1, 2, -500);
    }

    #[test]
    fn test_lb() {
        construct_test_load(&lb(1, 2, 500), LB as u8, 1, 2, 500);
        construct_test_load(&lb(1, 2, -500), LB as u8, 1, 2, -500);
    }

    #[test]
    fn test_lh() {
        construct_test_load(&lh(1, 2, 500), LH as u8, 1, 2, 500);
        construct_test_load(&lh(1, 2, -500), LH as u8, 1, 2, -500);
    }

    #[test]
    fn test_lbu() {
        construct_test_load(&lbu(1, 2, 500), LBU as u8, 1, 2, 500);
        construct_test_load(&lbu(1, 2, -500), LBU as u8, 1, 2, -500);
    }

    #[test]
    fn test_lhu() {
        construct_test_load(&lhu(1, 2, 500), LHU as u8, 1, 2, 500);
        construct_test_load(&lhu(1, 2, -500), LHU as u8, 1, 2, -500);
    }

    #[test]
    fn test_lw() {
        construct_test_load(&lw(1, 2, 500), LW as u8, 1, 2, 500);
        construct_test_load(&lw(1, 2, -500), LW as u8, 1, 2, -500);
    }

    #[test]
    fn test_sb() {
        construct_test_store(&sb(1, 2, 500), SB as u8, 1, 2, 500);
        construct_test_store(&sb(1, 2, -500), SB as u8, 1, 2, -500);
    }

    #[test]
    fn test_sh() {
        construct_test_store(&sh(1, 2, 500), SH as u8, 1, 2, 500);
        construct_test_store(&sh(1, 2, -500), SH as u8, 1, 2, -500);
    }

    #[test]
    fn test_sw() {
        construct_test_store(&sw(1, 2, 500), SW as u8, 1, 2, 500);
        construct_test_store(&sw(1, 2, -500), SW as u8, 1, 2, -500);
    }

    #[test]
    fn test_ecall() {
        let op = Instruction::ECall.encode();
        assert_eq!(opcode(op), SYSTEM);
        assert_eq!(funct3(op), ECALL_OR_EBREAK);
        assert_eq!(i_type_immediate_32(op), 0);
    }

    #[test]
    fn test_ebreak() {
        let op = Instruction::EBreak.encode();
        assert_eq!(opcode(op), SYSTEM);
        assert_eq!(funct3(op), ECALL_OR_EBREAK);
        assert_eq!(i_type_immediate_32(op), 1);
    }

    // TODO: The OpImm instructions would be better with some negative tests
    // TODO: All signed and unsigned immediates should have tests for the extrema (MAX_INT and 0 or
    // MIN_INT)
    // TODO: With some reasonable logic to generate N bit integers, it should be pretty easy to
    // quickcheck the encoder.
}
