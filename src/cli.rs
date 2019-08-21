use std::sync::Arc;
use std::fs::read_to_string;

use std::path::PathBuf;

use parking_lot::RwLock;
use structopt::StructOpt;


use crate::name::{ Name, mk_anon };
use crate::pretty::pretty_printer::{ PrettyPrinter, PPOptions };
use crate::env::Env;

/// A basic example
#[derive(StructOpt, Debug)]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
#[structopt(name = "nanoda",
            about = "A type checker for the Lean theorem prover",
            author = "ammkrn",
            version = "0.0.1")]
pub struct Opt {
    //A flag, true if used in the command line. Note doc comment will
    //be used for the help message of the flag.
    //Activate debug mode (currently does nothing)
    #[structopt(short = "d", long = "debug")]
    pub debug: bool,

     
    /** tell r_type how many threads you want it to use.
        Use `1` to check in serial, though r_type is
        very much not optimized for serial execution.
        Recommended : 4-8. */
    #[structopt(short = "t", long = "threads", default_value = "4")]
    pub num_threads : u64,

    /** tell r_type you want to pretty print something; options and the
        list of definitions to print are set in config files, called
        `pretty_options.txt` and `pretty_names.txt`. r_type will look for these
        both in the current working directory, and in an optional subdirectory
        called `config`.
        The names of the definitions you want pretty printed should be line separated
        and are accepted in the same format as in Lean (IE has_add.rec).
        The pretty printer options should also be line separated, and are also
        accepted as in Lean. For example, to turn on implicits, r_type will accept
        either `set_option pp.implicit true` or `pp.implicit true`. 
        Both of these files will ignores (as comments) lines beginning with `#` */
    #[structopt(short = "p", long = "print")]
    pub print : bool,

    /** File(s) to type check. Passing only a filename will look in the
        current directory. A full path will look for the file in the
        specified location*/
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