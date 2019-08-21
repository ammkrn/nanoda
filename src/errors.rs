use std::fmt::Debug;

/// これらのエラーは大半の時に、パターン一致が変なものをもらう時に投げられたものです。
/// 例えば、部分的関数が処理出来ない物をもらう文脈で。理想的に、こういうエラーを
/// 型システムで処理出来ますが、現在、rust は列挙型の種類を静的に識別できなので
/// (GADT のように)その行動をやるのはすごくノイズとなる明示的な型変換・型定義
/// が必要となるからやりません。

pub fn err_get_serial<T : Debug>(loc : u32, owise : &T) -> ! {
    eprintln!("expr line {}; Expr::get_serial is a partial function defined only on expresisons made with the `Local` constructor, but it was called with {:?}\n", loc, owise);
    std::process::exit(-1);
}

pub fn err_lc_binding<T : Debug>(loc : u32, owise : &T) -> ! {
    eprintln!("expr line {}; Expr::get_serial is a partial function defined only on expresisons made with the `Local` constructor, but it was called with {:?}\n", loc, owise);
    std::process::exit(-1);
}

pub fn err_binding_lc<T : Debug>(loc : u32, owise : &T) -> ! {
    eprintln!("`expr line {}; From` conversion for Level -> Binding is a partial function defined only on arguments of the form Expr::Local, but it was called with the following expression {:?}\n\n", loc, owise);
    std::process::exit(-1);
}
                
pub fn err_swap_local_binding_name<T : Debug>(loc : u32, owise : &T) -> !{
    eprintln!("expr line {}; Expr::swap_local_binding_name is a partial function defined only on expresisons made with the `Local` constructor, but it was called with {:?}\n", loc, owise);
    std::process::exit(-1);
}

pub fn err_offset_cache(loc : u32, idx : usize, len : usize) -> ! {
    eprintln!("expr line {}; OffsetCache failed to retrieve HashMap at index {}; vec length was {}\n", loc, idx, len);
    std::process::exit(-1);
}

pub fn err_normalize_pis<T : Debug>(loc : u32, got : &T) -> ! {
    eprintln!("expr line {}; Expected a `Sort` term in inductive mod, got {:?}\n", loc, got);
    std::process::exit(-1);
}

pub fn err_infer_var<T : Debug>(loc : u32, got : &T) -> ! {
    eprintln!("tc line {}; infer function got a variable term, but that should never happen. received this term : {:?}\n", loc, got);
    std::process::exit(-1);
}

pub fn err_infer_const<T : Debug>(loc : u32, name : &T) -> ! {
    eprintln!("tc line {}; infer_const function expected a declaration to be in the environment, but it was missing. Looked for {:?}\n", loc, name);
    std::process::exit(-1);
}

pub fn err_infer_universe<T : Debug>(loc : u32, got : &T) -> ! {
    eprintln!("tc line {}; infer_universe function expected to be passed a term of type Sort, but got something else. Got term {:?}\n", loc, got);
    std::process::exit(-1);
}

pub fn err_infer_apps<T : Debug>(loc : u32, got : &T) -> ! {
    eprintln!("tc line {}; infer_apps function expected to be match a Pi term, but got something else. Got term {:?}\n", loc, got);
    std::process::exit(-1);
}

pub fn err_req_def_eq<T : Debug>(loc : u32, got1 : &T, got2 : &T) -> ! {
    eprintln!("tc line {}; function require_def_eq received the following two functions expecting them to be found definitionally equal, but they were found not to be. Got E1 : {:?}\n\nE2 : {:?}\n\n", loc, got1, got2);
    std::process::exit(-1);
}

pub fn err_check_type<T : Debug>(loc : u32, got1 : &T, got2 : &T) -> ! {
    eprintln!("tc line {}; the function check_type expected the following two expression to be definitionally equal, but they were not. Got E1 : {:?}\n\nE2 : {:?}\n\n", loc, got1, got2);
    std::process::exit(-1);
}

pub fn err_rr_const<T : Debug>(loc : u32, got : &T) -> ! {
    eprintln!("rr line {}; creation of new reduction rule expected to get a Const expression, but got {:?}\n", loc, got);
    std::process::exit(-1);
}

pub fn err_add_rule<T : Debug>(loc : u32, name : &T) -> ! {
    eprintln!("env line {}; in reduction module, expected to find a major premise corresponding to name {:?}, but got nothing.", loc, name);
    std::process::exit(-1)
}

pub fn err_param_name<T : Debug>(loc : u32, got : &T) -> ! {
    eprintln!("level line {}; Level::param_name() is a partial function defined only for Param variants. Got {:?}\n", loc, got);
    std::process::exit(-1)
}


pub fn join_panic(loc : u32) -> ! {
    eprintln!("main line {}; a worker thread in the `check_parallel` function panicked! More information should be available in the console.", loc);
    std::process::exit(-1)
}


pub fn scope_err(loc : u32) -> ! {
    eprintln!("main line {}; a worker thread in the `check_parallel` function panicked! More information should be available in the console.", loc);
    std::process::exit(-1)
}


pub fn export_file_parse_err<T : std::fmt::Display>(loc : u32, err : T) -> ! {
    eprintln!("cli line {}; failed to parse at least one of the specified export files. Please check that the file exists at the specified path. Error details : {}\n", loc, err);
    std::process::exit(-1)
}

pub fn partial_is_pi<T : Debug>(loc : u32, item : T) -> ! {
    eprintln!("expr line {}; bad call to partial function `binder_is_pi`; expected Pi or Labmda, got {:?}\n", loc, item);
    std::process::exit(-1);
}

pub fn err_parse_kind<T : Debug>(t : &T) -> String {
   format!("unrecognized match on item kind while parsing. Expected 'N' 'U', or 'E', got {:?}\n", t)
}

