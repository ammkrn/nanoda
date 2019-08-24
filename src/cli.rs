use std::sync::Arc;
use std::fs::read_to_string;

use std::path::PathBuf;

use parking_lot::RwLock;
use structopt::StructOpt;


use crate::name::{ Name, mk_anon };
use crate::pretty::pretty_printer::{ PrettyPrinter, PPOptions };
use crate::env::Env;

#[derive(StructOpt, Debug)]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
#[structopt(name = "nanoda",
            about = "Lean 定理支援システムの型検査装置",
            author = "ammkrn",
            version = "0.0.1")]
pub struct Opt {
    #[structopt(short = "d", long = "debug")]
    pub debug: bool,

     
    /** スレッドの個数を指定する。1 を渡せば、直列で実行出来ますが、
        nanoda は並行実行を考慮して最適化されたんです。おすすめのスレッド
        個数は 4 以上です。
        */
    #[structopt(short = "t", long = "threads", default_value = "4")]
    pub num_threads : u64,

    /** プリティープリンター(PP)を有効。PP の設定・プリントされる定義のリスト
        は config ファイルで制御されます。./config にあるファイルに詳しく
        説明されます。
        */
    #[structopt(short = "p", long = "print")]
    pub print : bool,

    /** 検査したいファイルのリスト。名前しか渡されなければ、作業ダイレクトリー
        に探して、フルパスが渡されたらその位置に探します。
        */
    #[structopt(name = "FILE x N", parse(from_os_str))]
    files: Vec<PathBuf>,
}

impl Opt {
    pub fn try_read_files(&self) -> Result<Vec<String>, std::io::Error>{
        self.files.iter().map(|x| try_read_cwd(x)).collect()
    }

}

fn try_read_cwd(suggestion : &PathBuf) -> Result<String, std::io::Error> {
    match std::env::current_dir() {
        Err(_) => read_to_string(suggestion),
        Ok(mut path) => {
            path.push(suggestion.clone());
            read_to_string(path)
        }
    }
}

// I'll fix these at some point; at the moment we're (very)
// fast and loose with the parsing, and parsing fails silently.
fn find_true_else_false(s : &str) -> bool {
    if s.contains("true") {
        return true
    } else {
        false
    }
}

fn find_first_usize(s : &str) -> Option<usize> {
    for ws in s.split_whitespace() {
        match ws.parse::<usize>() {
            Ok(n) => return Some(n),
            _ => continue
        }
    }

    None
}

pub fn try_read_pp_options() -> Option<PPOptions> {
    let mut cwd = std::env::current_dir().ok()?;
    let mut cwd_separate_cfg = cwd.clone();
    cwd.push(PathBuf::from("pp_options.txt"));
    cwd_separate_cfg.push(PathBuf::from("config/pp_options.txt"));

    let mut empty_options = PPOptions::new_default();

    // try to read in both locations
    for line in read_to_string(cwd)
                .ok()
                .or(read_to_string(cwd_separate_cfg).ok())?
                .lines() {
        match line {
            s if s.starts_with('#') => (),
            s if s.contains("pp.all") => empty_options.all = find_true_else_false(s),
            s if s.contains("pp.implicit") => empty_options.implicit = find_true_else_false(s),
            s if s.contains("pp.notation") => empty_options.notation = find_true_else_false(s),
            s if s.contains("pp.proofs") => empty_options.proofs = find_true_else_false(s),
            s if s.contains("pp.locals_full_names") => empty_options.locals_full_names = find_true_else_false(s),
            s if s.contains("pp.indent") => empty_options.indent = find_first_usize(s)?,
            s if s.contains("pp.width") => empty_options.width = find_first_usize(s)?,
            _ => ()
        }
    }

    Some(empty_options)


}

pub fn try_read_pp_file() -> Option<(Vec<Name>, Vec<String>)> {
    let mut cwd = std::env::current_dir().ok()?;
    let mut cwd_separate_cfg = cwd.clone();
    cwd.push(PathBuf::from("pp_names.txt"));
    cwd_separate_cfg.push(PathBuf::from("config/pp_names.txt"));

    let (mut names, mut errs) = (Vec::new(), Vec::new());

    for line in read_to_string(cwd)
                .ok()
                .or(read_to_string(cwd_separate_cfg).ok())?
                .lines() {
        match line.parse::<Name>() {
            Ok(n) => names.push(n),
            Err(_) => errs.push(String::from(line))
        }
    }

    Some((names, errs))
}

// Just prints to stdout until I figure out what I actually
// want to do with this.
pub fn pp_bundle(env : &Arc<RwLock<Env>>) {
    match try_read_pp_file() {
        None => (),
        Some((ns, _)) => {
            if ns.is_empty() {
                println!("\nNo items to pretty print\n");
            } else {
                let pp_options = try_read_pp_options();
                //let mut outputs = Vec::<String>::with_capacity(ns.len());
                println!("\nBEGIN PRETTY PRINTER OUTPUT : \n");
                for n in ns.iter() {
                    let rendered = PrettyPrinter::print_declar(pp_options.clone(), n, &env);
                    println!("{}\n", rendered);
                }
                println!("END PRETTY PRINTER OUTPUT : \n");
            }
        } 
    }
}


impl std::str::FromStr for Name {
    type Err = String;
    fn from_str(s : &str) -> Result<Name, String> {
        let mut base = mk_anon();

        if s.is_empty() {
            return Err(format!("Cannot pretty print the empty/anonymous Lean name!"))
        }

        let fragments = s.split_terminator('.');

        for f in fragments {
            match f.parse::<u64>() {
                Ok(n) => { base = base.extend_num(n); },
                _ => {
                    if f.is_empty() {
                        return Err(format!("Name cannot be empty!"))
                    } else if f.starts_with('#') {
                        return Err(format!("Commented out"))
                    } else {
                        base = base.extend_str(f);
                    }
                }
            }
        }

        Ok(base)
    }
}