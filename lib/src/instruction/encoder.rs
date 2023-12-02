use super::funct3::{ADDI, SLTI, SLTIU};
use super::opcodes::OP_IMM;

const fn op_imm_funct3(
    destination_register: usize,
    source_register: usize,
    funct3: u8,
    imm: u16,
) -> u32 {
    if destination_register > 32 || source_register > 32 || funct3 > 0b111 {
        panic!("destination or source > 32, cannot construct instruction");
    }

    if imm > 0b1111_1111_1111 {
        panic!("immediate exceeds 12 bits");
    }

    let destination_register = destination_register as u32;
    let source_register = source_register as u32;

    (OP_IMM as u32)
        | (destination_register << 7)
        | ((funct3 as u32) << 12)
        | (source_register << 15)
        | (imm as u32) << 20
}

/// Construct an add-immediate instruction that will add a signed 12-bit immediate
/// to the register in rs1 and then place it in rd
pub const fn addi(destination_register: usize, source_register: usize, imm: u16) -> u32 {
    op_imm_funct3(destination_register, source_register, ADDI, imm)
}

/// Construct a set-less-than-immediate (SLTI) instruction that will set the destination
/// register to 1 if the register rs1 is < the sign extended immediate and set the destination
/// register to zero otherwise.
pub const fn slti(destination_register: usize, source_register: usize, imm: u16) -> u32 {
    op_imm_funct3(destination_register, source_register, SLTI, imm)
}

/// Construct a set-less-than-immediate-unsigned (SLTIU) instruction that will set the destination
/// register to 1 if the register rs1 is < the sign extended immediate and set the destination
/// register to zero otherwise. In SLTIU the comparison is done treating the arguments as unsigned.
pub const fn sltiu(destination_register: usize, source_register: usize, imm: u16) -> u32 {
    op_imm_funct3(destination_register, source_register, SLTIU, imm)
}

/// Construct a canonical no-op.
///
/// There are a few instructions that will cause no change except the PC to move forward, but the canonical encoding of a no-op is an ADDI with rd=0 rs1=0 and imm=0
pub const fn no_op() -> u32 {
    addi(0, 0, 0)
}

#[cfg(test)]
mod test {
    use super::super::decoder::*;
    use super::*;

    #[test]
    fn test_addi() {
        let no_op = addi(0, 0, 0);
        assert_eq!(opcode(no_op), OP_IMM);
        assert_eq!(funct3(no_op), ADDI);
        assert_eq!(rd(no_op), 0);
        assert_eq!(rs1(no_op), 0);
        assert_eq!(i_type_immediate_32(no_op), 0);

        let normal_immediate = addi(2, 4, 100);
        assert_eq!(opcode(normal_immediate), OP_IMM);
        assert_eq!(funct3(normal_immediate), ADDI);
        assert_eq!(rd(normal_immediate), 2);
        assert_eq!(rs1(normal_immediate), 4);
        assert_eq!(i_type_immediate_32(normal_immediate), 100);
    }

    #[test]
    fn test_slti() {
        unimplemented!();
    }

    #[test]
    fn test_sltiu() {
        unimplemented!();
    }
}
