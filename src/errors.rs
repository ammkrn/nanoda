use std::fmt::Debug;

/// Most of these are errors that get thrown in the event
/// that a pattern match expects something that it doesn't end up getting.
/// For instance, partial conversions or failed definitional equality/
/// inference checks. Ideally we would get rid of the ones related to partial functions,
/// but Rust's type system doesn't offer discrimination of enum variants
/// at the type level, and my experience trying to break each enum variant
/// out into its own struct suggested that the amount of extra code you would need
/// just to do explicit casting between types would be a huge hit to readability
/// and directness. 


pub fn quot_rec_bad_app<T : Debug>(loc : u32, arg_received : &T) -> ! {
    eprintln!("function tc::reduce_quot_rec; line {} should always get an `App` term, but got {:#?}\n", loc, arg_received);
    std::process::exit(-1);
}

pub fn unfold_definition_infallible_failed<T : Debug>(loc : u32, arg_received : &T) -> ! {
    eprintln!("function tc::unfold_definition_infallible line {}; should always get `Some`, but got a None with arg : {:#?}\n", loc, arg_received);
    std::process::exit(-1);
}

pub fn mutual_different_universes<T : Debug>(loc : u32, owise1 : &T, owise2 : &T) -> ! {
    eprintln!("function `check_inductive_types` line {}; mutually inductive types must live in the same universe, but u1 was {:#?}, while u2 was : {:#?}", loc, owise1, owise2);
    std::process::exit(-1);
}


pub fn use_dep_elim_not_sort<T : Debug>(loc : u32, owise : &T) -> ! {
    eprintln!("function `check_inductive_types` line {}; check `use_dep_elim` expected a Sort, but got {:#?}", loc, owise);
    std::process::exit(-1);
}


pub fn check_inductive_i_neq(loc : u32, i : usize, num_params : usize) -> ! {
    eprintln!("function `check_inductive_types` line {}; `i` must equal num params, but i was {}, while num_params was {}", loc, i, num_params);
    std::process::exit(-1);
}
pub fn check_inductive_bad_indices(loc : u32, idx : usize) -> ! {
    eprintln!("function `check_inductive_types` line {}; expected to find an element at {} of `nindices`, but it didn't exist!\n", loc, idx);
    std::process::exit(-1);
}


pub fn err_get_param_type<T : Debug>(loc : u32, owise : &T) -> ! {
    eprintln!("add_inductive line {}; function `get_param_type` expected a Local expr, but got {:#?}\n", loc, owise);
    std::process::exit(-1);
}

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
    eprintln!("tc line {}; the function check_type expected the following two expression to be definitionally equal, but they were not. Got \nE1 : {:?}\n\nE2 : {:?}\n\n", loc, got1, got2);
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

pub fn toplevel_err<T : Debug>(t : &T) -> ! {
   eprintln!("execution failed with error : {:?}\n", t);
   std::process::exit(-1)
}




pub type NanodaResult<T> = Result<T, NanodaErr>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NanodaErr {
    BadIndexErr(&'static str, u32, usize),
    NotSortErr(&'static str, u32),
    NotLocalErr(&'static str, u32),
    NotBinderErr(&'static str, u32),
    DupeLparamErr(&'static str, u32, usize),
    NonposOccurrenceErr(&'static str, u32),
    InvalidOccurrenceErr(&'static str, u32),
    UseDepElimNotSortErr,
    GetParamTypeErr,
    NoneErr(&'static str, u32, &'static str),
    CnstrBadParamTypeErr,
    CnstrBadTypeErr,
    CnstrUnivErr,
    ParseExhaustedErr(usize, u32),
    ParseIntErr(usize, u32, std::num::ParseIntError),
    ParseStringErr(usize, u32),
    TcNeqErr(&'static str, u32),
    


}

impl std::fmt::Display for NanodaErr {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            NanodaErr::BadIndexErr(file, loc, idx) => write!(f, "Got a fatal error at {} line {} for a bad index. Tried to get {}\n", file, loc, idx),
            NanodaErr::NotSortErr(file, loc)  => write!(f, "Got a fatal error at {} line {}; tried to get info about a `Sort` Expr, but passed ar was not a Sort.\n", file, loc),
            NanodaErr::NotLocalErr(file, loc)  => write!(f, "Got a fatal error at {} line {}; tried to get info about a `Local` Expr, but passed ar was not a Sort.\n", file, loc),
            NanodaErr::NotBinderErr(file, loc)  => write!(f, "Got a fatal error at {} line {}; Function expected a binder expression (Pi or Lambda), but got something else.\n", file, loc),
            NanodaErr::DupeLparamErr(file, loc, idx)  => write!(f, "Got a fatal error at {} line {}; Inductive type declarations should not contain duplicate univer parameters, but a duplicate was found at idx {}.\n", file, loc, idx),
            NanodaErr::NonposOccurrenceErr(file, loc) => write!(f, "Got a fatal error at {} line {}; function `check_positivity` found nonpositive occurence of inductive beind declared", file, loc),
            NanodaErr::InvalidOccurrenceErr(file, loc) => write!(f, "Got a fatal error at {} line {}; function `check_positivity` found an invalid occurence of inductive being declared", file, loc),
            NanodaErr::UseDepElimNotSortErr => write!(f, "inductive::use_dep_elim() was supposed to get a Sort, but didn't"),
            NanodaErr::GetParamTypeErr => write!(f, "inductive::get_param_type() was supposed to get a Local, but didn't"),
            NanodaErr::NoneErr(file, loc, msg) => write!(f, "Got a fatal err (None err) in {} line {}; {}", file, loc, msg),
            NanodaErr::CnstrBadParamTypeErr => write!(f, "inductive constructor's paramter was not well-typed!"),
            NanodaErr::CnstrUnivErr => write!(f, "inductive constructor's universe was too big!"),
            NanodaErr::CnstrBadTypeErr => write!(f, "inductive constructor's type was incorrect!"),
            NanodaErr::ParseExhaustedErr(line, source) => write!(f, "Parse error at source line {}, source line {} : source iterator unexpectedly yielded None (was out of elements)", line, source),
            NanodaErr::ParseIntErr(line, source, err) => write!(f, "Parse error at lean output line {}, source line {} : {}", line, source, err),
            NanodaErr::ParseStringErr(line, source) => write!(f, "Parse error at lean output line {}, source line {}", line, source),
            NanodaErr::TcNeqErr(file, loc) => write!(f, "Adding a declaration failed because it was not well-typed! {} line {}", file, loc),
        }
    }
}

impl std::error::Error for NanodaErr {}
