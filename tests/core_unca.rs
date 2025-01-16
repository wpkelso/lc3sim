mod common;

use paste::paste;

mod exec {
    use super::*;

    use common::penn_sim::{load_os, MemDump};
    use lc3sim_project::{
        defs::{LC3MemAddr, USER_SPACE},
        executors::{consolidated::ConsolidatedLC3, core::CoreLC3, populate_from_bin, LC3},
        harnesses::{simple::FailIO, sync::lim_step_continue},
    };

    /// Prevent infinite loops when the implementation jumps incorrectly
    const EXEC_LIMIT: u64 = 100_000;

    macro_rules! cmp_test {
        ( $name:ident, $path:literal, $executor:expr ) => {
            paste! {
                #[test]
                fn [<$name _exec>]() {
                    let program_under_test = static_compiled!($path);

                    let mut lc3 = $executor;
                    load_os(&mut lc3);
                    populate_from_bin(&mut lc3, &**program_under_test.obj());

                    // Confirm the memory loaded correctly
                    for (offset, word) in program_under_test.obj_words().skip(1).enumerate() {
                        let pos = (offset as LC3MemAddr) + USER_SPACE;
                        assert_eq!(lc3.mem(pos), word);
                    }

                    // Step all the way through execution
                    assert!(lim_step_continue(&mut FailIO, &mut lc3, EXEC_LIMIT).unwrap());

                    // Confirm full memory match with penn-sim
                    let MemDump { output_lines, memory } = static_output!($path, "");
                    assert_eq!(output_lines, "");
                    for (lc3_mem, penn_mem) in lc3.iter().zip(memory) {
                        assert_eq!(&lc3_mem, penn_mem)
                    }
                }
            }
        };
    }

    macro_rules! cmp_test_all {
        ( $name:ident, $path:literal ) => {
            paste! {
                cmp_test!( [<$name _core>], $path, CoreLC3::new() );
                cmp_test!( [<$name _consolidated>], $path, ConsolidatedLC3::boxed() );
            }
        };
    }

    cmp_test_all!(mult_10, "../test_data/unca/split_apart/mult_10.asm");
    cmp_test_all!(rev_string, "../test_data/unca/split_apart/rev_string.asm");
    cmp_test_all!(char_count, "../test_data/unca/split_apart/char_count.asm");
    cmp_test_all!(r1_pop, "../test_data/unca/split_apart/r1_pop.asm");
    cmp_test_all!(xor, "../test_data/unca/split_apart/xor.asm");
}
