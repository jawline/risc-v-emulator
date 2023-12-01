use super::opcodes::OP_IMM;
use super::funct3::ADDI;

pub const fn addi(
    destination_register: usize,
    source_register: usize,
    imm: u16,
) -> u32 {
    if destination_register > 32 || source_register > 32 {
        panic!("destination or source > 32, cannot construct instruction");
    }

    if (imm & 0b1111_0000_0000_000) != 0 {
        panic!("immediate exceeds 12 bits");
    }

    let destination_register = destination_register as u32;
    let source_register = source_register as u32;

    (OP_IMM as u32)
        | (destination_register << 7)
        | ((ADDI as u32) << 12)
        | (source_register << 15)
        | (imm as u32) << 20
}

pub const fn no_op() -> u32 {
    addi(0, 0, 0)
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::decoder::*;

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
}
