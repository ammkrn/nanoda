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


#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimallocator::Mimalloc = mimallocator::Mimalloc;

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
            num_checked += check_serial(s, opt.print);
        }
        owise => for s in export_file_strings {
            num_checked += check_parallel(s, owise as usize, opt.print)
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


fn check_serial(source : String, print : bool) -> usize {
    let env = Arc::new(RwLock::new(Env::new(EXPECTED_NUM_MODS)));
    let add_queue = RwQueue::with_capacity(EXPECTED_NUM_MODS);
    let check_queue = RwQueue::with_capacity(EXPECTED_NUM_MODS);

    if let Err(e) =  LineParser::parse_all(source, &add_queue, &env) {
        errors::export_file_parse_err(line!(), e)
    }

    loop_add(&add_queue, &check_queue, &env, 1);
    loop_check(&check_queue, &env);

    if print {
        pp_bundle(&env);
    }

    let n = env.read().num_declars();
    n
}

fn check_parallel(source : String, num_threads : usize, print : bool) -> usize {
    let env = Arc::new(RwLock::new(Env::new(EXPECTED_NUM_MODS)));
    let add_queue = RwQueue::with_capacity(EXPECTED_NUM_MODS);
    let check_queue = RwQueue::with_capacity(EXPECTED_NUM_MODS);

    let scope_ = thread::scope(|s| {

        let mut thread_holder = Vec::with_capacity(num_threads);

        // add and parse can be done separately/concurrently, but both MUST be done 
        // in order. So, when parsing ends, that thread goes immediately to
        // the check pool instead of adding.
        thread_holder.push(s.spawn(|_| {
            if let Err(e) =  LineParser::parse_all(source, &add_queue, &env) {
                errors::export_file_parse_err(line!(), e)
            }
            loop_check(&check_queue, &env);
        }));


        thread_holder.push(s.spawn(|_s| {
            loop_add(&add_queue, &check_queue, &env, num_threads);
            loop_check(&check_queue, &env);
        }));

        // We spawn (num_threads - 2) checker threads here since 
        // parser and adder will check when they're done.
        for _ in 0..(num_threads - 2) {
            thread_holder.push(s.spawn(|_s| {
                loop_check(&check_queue, &env);
            })); 
        }

    });

    if scope_.is_err() {
        errors::scope_err(line!())
    }

    if print {
        pp_bundle(&env);
    }

    let n = env.read().num_declars();
    n
}


// Constantly poll the `add_queue` to see if there's something
// to add. If the item popped off the queue is a Left(Mod), 
// add said mod. `None` means there aren't any items yet, but
// there will be later. Right(..) means adding is finished,
// and this thread is redirected to working on the `check_queue`
pub fn loop_add(add_queue : &ModQueue,
                check_queue : &CompiledQueue,
                env : &Arc<RwLock<Env>>,
                num_threads : usize) {
    loop {
        match add_queue.pop() {
            Some(Left(elem)) => {
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
}

// Same as above. Constantly poll for new work, with Left(Compiled)
// indicating an item to be checked, `None` meaning 'try again later'
// and Right(..) meaning all checking has completed.
pub fn loop_check(check_queue : &CompiledQueue,
                  env : &Arc<RwLock<Env>>) {
    loop {
         match check_queue.pop() {
             Some(Left(elem)) => elem.check_only(&env),
             Some(Right(_)) => break,
             None => continue
         }
     }
}

