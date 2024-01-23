#![allow(internal_features)]
#![feature(const_mut_refs)]
#![feature(generic_arg_infer)]
#![feature(core_intrinsics)]
#![feature(const_try)]
#![feature(const_trait_impl)]
#![feature(effects)]
pub mod cpu;
pub mod instruction;
pub mod machine;
pub mod memory;
pub mod util;
