#![forbid(unsafe_code)]
#![allow(unused_parens)]
#![allow(non_snake_case)]

use std::sync::Arc;
use std::time::SystemTime;

use crossbeam_utils::thread;

use parking_lot::RwLock;

use structopt::StructOpt;

use crate::env::Env;
use crate::parser::LineParser;
use crate::utils::{ Either::*, RwQueue, ModQueue, CompiledQueue, END_MSG_CHK };
use crate::cli::{ Opt, pp_bundle };


pub mod utils;
pub mod errors;
pub mod name;
pub mod level;
pub mod expr;
pub mod reduction;
pub mod tc;
pub mod env;
pub mod quot;
pub mod inductive;
pub mod parser;
pub mod pretty;
pub mod cli;


// By default, make the 'modifications' hashmap large enough to accomodate
// core + ~2000 items (core is about 9000 items). If the passed export file
// has more modifications, the hashmap will just resize, but that's a
// (relatively) costly operation.
pub const EXPECTED_NUM_MODS : usize = 11_000;

fn main() {
    let opt = Opt::from_args();

    if opt.debug {
        println!("CLI returned these arguments : {:#?}", opt);
    }

    let export_file_strings = match opt.try_read_files() {
        Ok(strings) => strings,
        Err(e) => errors::export_file_parse_err(line!(), e)
    };

    let start_instant = SystemTime::now();

    let mut num_checked = 0usize;
    match opt.num_threads {
        0 | 1 => for s in export_file_strings {
            check_serial(s, opt.print, &mut num_checked);
        }
        owise => for s in export_file_strings {
            check_parallel(s, owise as usize, opt.print, &mut num_checked)
        }
    }

    match start_instant.elapsed() {
        Ok(dur) => println!("\n### Finished checking {} items in {:?}; to the best \
                               of our knowledge, all terms were well-typed! ###\n", num_checked, dur),
        Err(e)  => println!("\n### Finished checking {} items; to the best of our \
                               knowledge, all terms were well-typed!\n I wasn't able to time \
                               execution though; here was the error : {} ###", num_checked, e)
    }

}


fn check_serial(source : String, print : bool, num_mods : &mut usize) {
    let env = Arc::new(RwLock::new(Env::new(EXPECTED_NUM_MODS)));
    let add_queue = RwQueue::with_capacity(EXPECTED_NUM_MODS);
    let check_queue = RwQueue::with_capacity(EXPECTED_NUM_MODS);

    LineParser::parse_all_parallel(source, &add_queue, &env).expect("Parse failure!");
    loop_add(&add_queue, &check_queue, &env, 1);
    loop_check(&check_queue, &env);

    std::mem::replace(num_mods, *num_mods + env.read().num_mods());

    if print {
        pp_bundle(&env);
    }
}

fn check_parallel(source : String, num_threads : usize, print : bool, num_mods : &mut usize) {
    let env = Arc::new(RwLock::new(Env::new(EXPECTED_NUM_MODS)));
    let add_queue = RwQueue::with_capacity(EXPECTED_NUM_MODS);
    let check_queue = RwQueue::with_capacity(EXPECTED_NUM_MODS);

    let sco = thread::scope(|s| {

        let mut thread_holder = Vec::with_capacity(num_threads);

        // add and parse can be done separately, but both MUST be done in order, meaning that
        // when the parser thread ends, it has to go immediately into the check pool.
        // loop_add(&add_queue, &check_queue, &env, num_threads);
        let parser_thread = s.spawn(|_| {
            LineParser::parse_all_parallel(source, &add_queue, &env).expect("Parse failure!");
            loop_check(&check_queue, &env);
        });

        thread_holder.push(parser_thread);

        let adder = s.spawn(|_s| {
            loop_add(&add_queue, &check_queue, &env, num_threads);
            loop_check(&check_queue, &env);
        });

        thread_holder.push(adder);

        // We spawn (num_threads - 2) here since we've already spawned
        // a parser thread and an adder thread, both of which will
        // start looping through the `check` queue when their other 
        // job is finished
        for _ in 0..(num_threads - 2) {
            let checker = s.spawn(|_s| {
                loop_check(&check_queue, &env);
            }); 

            thread_holder.push(checker);
        }

    });

    std::mem::replace(num_mods, *num_mods + env.read().num_mods());

    if print {
        pp_bundle(&env);
    }

    match sco {
        Ok(_) => (),
        Err(_) => errors::scope_err(line!())
    }

}


// Constantly poll the `add_queue` to see if there's something
// to add. If the item popped off the queue is a Left(Mod), 
// add said mod. `None` means there aren't any items yet, but
// there will be later. Right(..) means adding is finished,
// and this thread is redirected to working on the `check_queue`
pub fn loop_add(add_queue : &ModQueue,
                check_queue : &CompiledQueue,
                env : &Arc<RwLock<Env>>,
                num_threads : usize) -> usize {
    let mut num_mods = 0usize;
    loop {
        match add_queue.pop() {
            Some(Left(elem)) => {
                num_mods += 1;
                let compiled = elem.compile(&env);
                compiled.add_only(&env);
                check_queue.push(Left(compiled));
            },
            Some(Right(_)) => {
                for _ in 0..(num_threads * 2) {
                    check_queue.push(END_MSG_CHK);
                }
                break
            },
            None => continue,
        }
    }

    num_mods

}

// Same as above. Constantly poll for new work, with Left(Compiled)
// indicating an item to be checked, `None` meaning 'try again later'
// and Right(..) meaning all checking has completed.
pub fn loop_check(check_queue : &CompiledQueue,
                  env : &Arc<RwLock<Env>>) {
    loop {
         match check_queue.pop() {
             Some(Left(elem)) => {
                 elem.check_only(&env);
             },
             Some(Right(_)) => break,
             None => continue
         }
     }
}

