//! CLI for pure LC3 testing purposes.

use std::{
    env::args,
    fs::File,
    io::{stdin, BufReader},
};

use lc3sim_project::{
    defs::{LC3Word, RegAddr},
    executors::{core::CoreLC3, populate_from_bin, LC3},
};

fn main() {
    let mut lc3 = CoreLC3::new();
    for arg in args().skip(2) {
        populate_from_bin(&mut lc3, BufReader::new(File::open(arg).unwrap()));
    }

    let start_line = LC3Word::from_str_radix(&args().nth(1).unwrap(), 16).unwrap();
    while lc3.pc() != start_line {
        lc3.step().unwrap()
    }

    loop {
        for reg in 0..8 {
            println!("Reg {reg}: {:#X}", lc3.reg(RegAddr::panic_from_u8(reg)));
        }

        println!(
            "Current instruction {:#X}: {:?}",
            lc3.pc(),
            lc3.cur_inst().unwrap()
        );

        stdin().read_line(&mut String::new()).unwrap();
        lc3.step().unwrap();
    }
}
