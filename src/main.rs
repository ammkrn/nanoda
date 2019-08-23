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

// デフォールトで、`CompiledModification` を保持するためのHashMapは 
// core +- 2000 個の定義を保持出来るぐらい初期化されます。
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
        Ok(dur) => println!("\n### 検査終了です！ {:?}にアイテムを{}個検査しました. 我々の知る \
                               知る限りでは、全部合格でした! ###\n", dur, num_checked),
        Err(e) => println!("\n### 検査終了です！ アイテムを{}個検査しました. 我々の知る \
                               知る限りでは、全部合格でした! しかし、実行が経った時間を測る作業\
                               は失敗になってしまいました : {} ###\n", num_checked, e),
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

        // 並行文脈なら、アイテムをパース・環境に追加することは同時に出来ますが、パーシングと
        // 追加する作業はそれぞれ順序にやられなければならないんだから、自分の一人っ子のスレッド
        // でやられます。パーシングが終了された後、検査キューへ移動してってこと。
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

        // パーサースレッドも追加するスレッドも既にspawnしたので、ここで num_threads - 2
        // の個数を spawn します。
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


/// `EndMsg` をもらうまで、add_queue をポールして、中身の要素
/// を検査せずに環境へ追加して。キューを枯渇する後、check_queueへ
/// 言ってってこと。`None` の値がキューから引き出されたら、それって
/// 「パーサースレッドが要素を入れてくれることを待ってます」っていう
/// シグナルだ。
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


///`EndMsg`をもらうまで、キューをポールして、それからの
/// 定義を検査してっていう作業だ。`None` 値って 「add_queue」
/// が検査すべき要素を入れてくれることを待ってますっていう
/// メッセージです。
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

