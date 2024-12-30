mod common;

use std::sync::LazyLock;

use common::penn_sim::CompileSet;

mod exec {
    use super::*;

    use common::penn_sim::load_os;
    use lc3sim_project::{
        defs::{LC3MemAddr, USER_SPACE},
        executors::{core::CoreLC3, populate_from_bin, LC3MemLoc, LC3},
        harnesses::{simple::IgnoreIO, sync::step_continue},
        util::format_all_word_bits,
    };

    #[test]
    fn mult_10_exec() {
        let mult_10 = static_compiled!("../test_data/unca/split_apart/mult_10.asm");

        let mut lc3 = CoreLC3::new();
        load_os(&mut lc3);
        populate_from_bin(&mut lc3, &**mult_10.obj());

        // Confirm the memory loaded correctly
        for (offset, word) in mult_10.obj_words().skip(1).enumerate() {
            let pos = (offset as LC3MemAddr) + USER_SPACE;
            assert_eq!(lc3.mem(pos), word);
        }

        // Step all the way through execution
        step_continue(&mut IgnoreIO, &mut lc3).unwrap();

        // Confirm full memory match with penn-sim
        let (output, mem_lines) = mult_10.post_process_mem_dump("");
        assert_eq!(output, "");
        for (lc3_mem, penn_mem) in lc3.iter().zip(mem_lines) {
            assert_eq!(lc3_mem, penn_mem)
        }
    }
}
