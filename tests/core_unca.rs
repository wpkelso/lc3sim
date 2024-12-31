mod common;

use std::sync::LazyLock;

use common::penn_sim::CompileSet;
use paste::paste;

mod exec {
    use super::*;

    use common::penn_sim::load_os;
    use lc3sim_project::{
        defs::{LC3MemAddr, USER_SPACE},
        executors::{core::CoreLC3, populate_from_bin, LC3MemLoc, LC3},
        harnesses::{simple::FailIO, sync::lim_step_continue},
    };

    /// Prevent infinite loops when the implementation jumps incorrectly
    const EXEC_LIMIT: u64 = 100_000;

    macro_rules! cmp_test {
        ( $name:ident, $path:literal ) => {
            paste! {
                #[test]
                fn [<$name _exec>]() {
                    let mult_10 = static_compiled!($path);

                    let mut lc3 = CoreLC3::new();
                    load_os(&mut lc3);
                    populate_from_bin(&mut lc3, &**mult_10.obj());

                    // Confirm the memory loaded correctly
                    for (offset, word) in mult_10.obj_words().skip(1).enumerate() {
                        let pos = (offset as LC3MemAddr) + USER_SPACE;
                        assert_eq!(lc3.mem(pos), word);
                    }

                    // Step all the way through execution
                    assert!(lim_step_continue(&mut FailIO, &mut lc3, EXEC_LIMIT).unwrap());

                    // Confirm full memory match with penn-sim
                    let (output, mem_lines) = mult_10.post_process_mem_dump("");
                    assert_eq!(output, "");
                    for (lc3_mem, penn_mem) in lc3.iter().zip(mem_lines) {
                        assert_eq!(lc3_mem, penn_mem)
                    }
                }
            }
        };
    }

    cmp_test!(mult_10, "../test_data/unca/split_apart/mult_10.asm");
    cmp_test!(rev_string, "../test_data/unca/split_apart/rev_string.asm");
    cmp_test!(char_count, "../test_data/unca/split_apart/char_count.asm");
    cmp_test!(r1_pop, "../test_data/unca/split_apart/r1_pop.asm");
    cmp_test!(xor, "../test_data/unca/split_apart/xor.asm");
}
