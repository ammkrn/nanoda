use std::cmp::{ Ordering, Ordering::* };

use std::sync::Arc;
use hashbrown::HashMap;
use parking_lot::RwLock;

use crate::name::Name;
use crate::level::Level;
use crate::expr::{ Expr, unique_const_names };
use crate::recursor::RecursorVal;
use crate::inductive::newinductive::InductiveDeclar;
use crate::tc::TypeChecker;
use crate::pretty::components::Notation;
use crate::utils::ShortCircuit::*;
use crate::errors::{ NanodaResult, NanodaErr::* };

use ConstantInfo::*;
use ReducibilityHint::*;

pub type ArcEnv = Arc<RwLock<Env>>;

// Why Declarations and ConstantInfo end up holding essentially the same
// information; there's no reason they need to be in here twice (after adding)
#[derive(Clone)]
pub struct Env {
    pub new_declarations: HashMap<Name, DeclarationKind>,
    pub notations : HashMap<Name, Notation>,
    pub constant_infos : HashMap<Name, ConstantInfo>,
    pub quot_is_init : bool,
}

impl std::cmp::PartialEq for Env {
    fn eq(&self, _other : &Env) -> bool { true }
}

impl std::cmp::Eq for Env {}

impl std::fmt::Debug for Env {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ENV")
    }
}

impl Env {
    pub fn new(num_mods : usize) -> Self {
        Env {
            new_declarations : HashMap::with_capacity(num_mods),
            notations : HashMap::with_capacity(500),
            constant_infos : HashMap::new(),
            quot_is_init : false,
        }
    }

    pub fn num_declars(&self) -> usize {
        self.new_declarations.len()
    }

    pub fn add_constant_info(&mut self, n : Name, c : ConstantInfo) {
        self.constant_infos.insert(n, c);
    }

    pub fn get_constant_info(&self, n : &Name) -> Option<&ConstantInfo> {
        self.constant_infos.get(n)
    }


    pub fn get_first_constructor_name(&self, n : &Name) -> Option<&Name> {
        match self.get_constant_info(n) {
            Some(InductiveInfo(InductiveVal { cnstrs, .. })) => cnstrs.get(0),
            _ => None,
        }
    }

    pub fn add_notation(&mut self, n : &Name, notation: Notation) {
        match self.notations.get(n) {
            Some(_) => (),
            None => { self.notations.insert(n.clone(), notation); }
        }
    }

    // check_no_dupe_name
    pub fn check_name(&self, n : &Name) {
        if self.new_declarations.contains_key(n) {
            panic!("declaration name {} was already declared!\n", n);
        }
    }
}

pub fn ensure_no_dupe_lparams(v : &Vec<Level>) -> NanodaResult<()> {
    for (idx, elem) in v.iter().enumerate() {
        let slice = &v[idx+1..];
        if slice.contains(elem) {
            return Err(DupeLparamErr(file!(), line!(), idx))
        }
    }
    Ok(())
}



// declaration.h ~69; This is just stored in OTHER val items.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConstantVal {
    pub name : Name,
    pub lparams : Vec<Level>,
    pub type_ : Expr
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AxiomVal {
    pub constant_val : ConstantVal,
    is_unsafe : bool
}

impl AxiomVal {
    pub fn new(name : Name, lparams : Vec<Level>, type_ : Expr, is_unsafe : Option<bool>) -> Self {
        let constant_val = ConstantVal::new(name, lparams, type_);
        AxiomVal {
            constant_val,
            is_unsafe : is_unsafe.unwrap_or(false)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DefinitionVal {
    pub constant_val : ConstantVal,
    pub value : Expr,
    pub hint : ReducibilityHint,
    pub is_unsafe : bool
}

impl DefinitionVal {
    pub fn new(env : ArcEnv, name : Name, lvls : Vec<Level>, ty : Expr, value : Expr) -> Self {
        let height_usize = match unique_const_names(&value)
                           .iter()
                           .filter_map(|name| env.read().get_hint(name))
                           .filter_map(|hint| hint.as_usize())
                           .max() {
                               Some(h) => h + 1,
                               _ => 1
                           };

        let hint = Regular(height_usize);
        let constant_val = ConstantVal::new(name, lvls, ty);
        DefinitionVal {
            constant_val,
            value : value,
            hint,
            is_unsafe : false
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TheoremVal {
    pub constant_val : ConstantVal,
    pub value : Expr
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OpaqueVal {
    pub constant_val : ConstantVal,
    pub value : Expr
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QuotVal {
    constant_val : ConstantVal,
}

impl QuotVal {
    pub fn from_const_val(c : ConstantVal) -> Self {
        QuotVal {
            constant_val : c,
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InductiveVal {
    pub constant_val : ConstantVal,
    pub nparams : usize,
    pub nindices : usize,
    pub all : Vec<Name>,
    pub cnstrs : Vec<Name>,
    pub is_rec : bool,
    pub is_unsafe : bool,
    pub is_reflexive : bool
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConstructorVal {
    pub constant_val : ConstantVal,
    pub induct : Name,
    pub cidx : usize,
    pub nparams : usize,
    pub nfields : usize,
    pub is_unsafe : bool,
}

impl ConstructorVal {
    // declaration.cpp ~78
    // extends constant_val
    pub fn new(name : Name,
               lparams : Vec<Level>,
               type_ : Expr,
               induct : Name,  
               cidx : usize, 
               nparams : usize, 
               nfields : usize, 
               is_unsafe : bool) -> Self {
        let constant_val = ConstantVal::new(name.clone(), lparams.clone(), type_.clone());

        let result = ConstructorVal {
            constant_val,
            induct,
            cidx,
            nparams,
            nfields,
            is_unsafe
        };
        result
    }

}



// CPP declaration.h ~424
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConstantInfo {
    AxiomInfo(AxiomVal),
    DefinitionInfo(DefinitionVal),
    TheoremInfo(TheoremVal),
    OpaqueInfo(OpaqueVal),
    QuotInfo(QuotVal),
    InductiveInfo(InductiveVal),
    ConstructorInfo(ConstructorVal),
    RecursorInfo(RecursorVal),
}

impl From<DeclarationKind> for ConstantInfo {
    fn from(d : DeclarationKind) -> ConstantInfo {
        match d {
            DeclarationKind::AxiomDeclar { val } => AxiomInfo(val),
            DeclarationKind::DefinitionDeclar { val } => DefinitionInfo(val),
            DeclarationKind::TheoremDeclar { .. } => unimplemented!(),
            DeclarationKind::OpaqueDeclar { .. } => unimplemented!(),
            DeclarationKind::QuotDeclar { val } => QuotInfo(val),
            DeclarationKind::MutualDefnDeclar { .. } => unimplemented!(),
            DeclarationKind::InductiveDeclar_ { .. } => unimplemented!()
        }
    }
}

impl ConstantInfo {

    pub fn get_hint(&self) -> ReducibilityHint {
        match self {
            DefinitionInfo(DefinitionVal { hint, .. }) => *hint,
            _ => unreachable!("Should never call 'get_hints' on a non-def ")
        }
    }
    // While all 'Const' items have a type, only definitions and theorems 
    // have a value level Expr item. (IE Pi x, lambda x...)
    pub fn has_value(&self, _allow_opaque : Option<bool>) -> bool {
        let allow_opaque = _allow_opaque.unwrap_or(false);

        match self {
            TheoremInfo(..) | DefinitionInfo(..) => true,
            OpaqueInfo(..) if allow_opaque => true,
            _ => false
        }
    }

    fn get_value_core(&self, maybe_debug_bool_allow_opaque : bool) -> Expr {
        assert!(self.has_value(Some(maybe_debug_bool_allow_opaque)));
        // maybe_debug is apparently always false
        match self {
            TheoremInfo(TheoremVal { value, .. }) => value.clone(),
            DefinitionInfo(DefinitionVal { value, .. }) => value.clone(),
            OpaqueInfo(OpaqueVal { .. }) if maybe_debug_bool_allow_opaque => {
                unreachable!("maybe_debug should always be false");
                //value.clone()
            },
            _ => unreachable!()
        }
    }

    pub fn get_value(&self) -> Expr {
        self.get_value_core(false)
    }

    pub fn get_constant_val(&self) -> &ConstantVal {
        match self {
            AxiomInfo(x) => &x.constant_val,
            DefinitionInfo(x) => &x.constant_val,
            TheoremInfo(x) => &x.constant_val,
            OpaqueInfo(x) => &x.constant_val,
            InductiveInfo(x) => &x.constant_val,
            ConstructorInfo(x) => &x.constant_val,
            RecursorInfo(x) => &x.constant_val,
            QuotInfo(x) => &x.constant_val,
        }
    }

    pub fn is_unsafe(&self) -> bool {
        match self {
            AxiomInfo(x) => x.is_unsafe,
            DefinitionInfo(x) => x.is_unsafe,
            TheoremInfo(_) => false,
            OpaqueInfo(_) => false,
            QuotInfo(_) => false,
            InductiveInfo(x) => x.is_unsafe,
            ConstructorInfo(x) => x.is_unsafe,
            RecursorInfo(x) => x.is_unsafe,

        }
    }
}

impl ConstantVal {
    pub fn new(name : Name, lparams : Vec<Level>, type_ : Expr) -> Self {
        ConstantVal {
            name,
            lparams,
            type_
        }
    }
}

impl InductiveVal {
    // name must equal all_used(0)
    // extends constant_val
    pub fn new(name : Name,
               lparams : Vec<Level>,
               type_ : Expr,
               nparams : usize,
               nindices : usize,
               all : Vec<Name>,
               cnstrs : Vec<Name>,
               is_rec : bool,
               is_unsafe : bool,
               is_reflexive : bool) -> Self {
        assert!(&name == &all[0]);
        let constant_val = ConstantVal::new(name, lparams, type_);
        InductiveVal {
            constant_val,
            nparams,
            nindices,
            all,
            cnstrs,
            is_rec,
            is_unsafe,
            is_reflexive
        }
    }
}



// CPP declaration.h ~192
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DeclarationKind {
    AxiomDeclar { val : AxiomVal },
    DefinitionDeclar { val : DefinitionVal },
    TheoremDeclar { val : TheoremVal },
    OpaqueDeclar { val : OpaqueVal },
    QuotDeclar { val : QuotVal },
    MutualDefnDeclar { defns : Vec<DefinitionVal> },
    InductiveDeclar_ { val : InductiveDeclar },
}

impl DeclarationKind {
    pub fn get_lparams(&self) -> Vec<Level> {
        unimplemented!()
        //match self {
        //    DeclarationKind::AxiomDeclar { val } => Vec::from(val.constant_val.lparams.clone()),
        //    DeclarationKind::DefinitionDeclar { val } => Vec::from(val.constant_val.lparams.clone()),
        //    DeclarationKind::TheoremDeclar { val } => Vec::from(val.constant_val.lparams.clone()),
        //    DeclarationKind::OpaqueDeclar { val } => Vec::from(val.constant_val.lparams.clone()),
        //    DeclarationKind::QuotDeclar { id } => unimplemented!(),
        //    DeclarationKind::MutualDefnDeclar { defns } => unimplemented!(),
        //    DeclarationKind::InductiveDeclar { .. } => unimplemented!(),
        //}
    }

    // Will have to change with mutuals
    pub fn get_type(&self) -> Expr {
        match self {
          DeclarationKind::AxiomDeclar { val } => {
              val.constant_val.type_.clone()
          }
          DeclarationKind::DefinitionDeclar { val } => {
              val.constant_val.type_.clone()
          }
          DeclarationKind::TheoremDeclar { .. } => {
              unimplemented!()
          }
          DeclarationKind::OpaqueDeclar { .. } => {
              unimplemented!()
          }
          DeclarationKind::QuotDeclar { val } => {
              val.constant_val.type_.clone()
          }
          DeclarationKind::MutualDefnDeclar { .. } => {
              unimplemented!()
          },
          // Mutual not yet implemented; can have a simple
          // get_type() function if we assert that this is
          // the only type for a given inductive declaration.
          DeclarationKind::InductiveDeclar_ { val } => {
              assert_eq!(val.types.len(), 1);
              val.types.get(0)
              .map(|x| x.type_.clone())
              .expect("DeclarationKind::get_type")
          }
 
        }
 

    }
    pub fn get_name(&self) -> Name {
        match self {
          DeclarationKind::AxiomDeclar { val } => {
              val.constant_val.name.clone()
          }
          DeclarationKind::DefinitionDeclar { val } => {
              val.constant_val.name.clone()
          }
          DeclarationKind::TheoremDeclar { .. } => {
              unimplemented!()
          }
          DeclarationKind::OpaqueDeclar { .. } => {
              unimplemented!()
          }
          DeclarationKind::QuotDeclar { val } => {
              val.constant_val.name.clone()
          }
          DeclarationKind::MutualDefnDeclar { .. } => unimplemented!(),
          DeclarationKind::InductiveDeclar_ { val } => {
              val.name.clone()
          }
 
        }
    }

    pub fn add_to_env(&self, env : ArcEnv, _check : bool) -> NanodaResult<()> {
        match self {
            DeclarationKind::AxiomDeclar { val } => {
                if _check {
                    check_constant_val_no_tc(env.clone(), val.constant_val.clone(), val.is_unsafe)?
                } 

                let name_key = val.constant_val.name.clone();
                env.write().constant_infos.insert(name_key, ConstantInfo::from(self.clone()));
                Ok(())
            },
            DeclarationKind::DefinitionDeclar { val } => {
                if val.is_unsafe {
                    if _check {
                        let safe_only = false;
                        let mut tc = TypeChecker::new(Some(safe_only), env.clone());
                        check_constant_val_wtc(val.constant_val.clone(), &mut tc)?;
                    }

                    let name_key = val.constant_val.name.clone();
                    env.write().constant_infos.insert(name_key, ConstantInfo::from(self.clone()));

                    if _check {
                        let safe_only = false;
                        let mut tc = TypeChecker::new(Some(safe_only), env.clone());
                        let val_type = tc.check(&val.value, val.constant_val.lparams.clone());
                        if (tc.is_def_eq(&val_type, &val.constant_val.type_) == NeqShort) {
                            return Err(TcNeqErr(file!(), line!()))
                        }
                    }
                    Ok(())
                } else {
                    if _check {
                        let mut tc = TypeChecker::new(None, env.clone());
                        check_constant_val_wtc(val.constant_val.clone(), &mut tc)?;
                        let val_type = tc.check(&val.value, val.constant_val.lparams.clone());
 
                        if (tc.is_def_eq(&val_type, &val.constant_val.type_) == NeqShort) {
                            return Err(TcNeqErr(file!(), line!()))
                        } 
                    }
                    let name_key = val.constant_val.name.clone();
                    env.write().constant_infos.insert(name_key, ConstantInfo::from(self.clone()));
                    Ok(())
                }
            },
            DeclarationKind::TheoremDeclar { .. } => {
                unimplemented!()
                //if _check {
                //    let mut tc = TypeChecker::new(None, env.clone());
                //    check_constant_val_wtc(val.constant_val.clone(), &mut tc);
                //    // revisit : pass lparams
                //    let val_type = tc.check(&val.value, HashSet::new());
                //    if (tc.is_def_eq(&val_type, &val.constant_val.type_) == NeqShort) {
                //        return Err(TcNeqErr(file!(), line!()))
                //    }
                //}
                //let name_key = val.constant_val.name.clone();
                //env.write().constant_infos.insert(name_key, ConstantInfo::from(self.clone()));
            },
            DeclarationKind::OpaqueDeclar { .. } => {
                unimplemented!()
            }
            DeclarationKind::QuotDeclar { val } => {
                let name_key = val.constant_val.name.clone();
                let as_constant_info = ConstantInfo::QuotInfo(val.clone());
                env.write().constant_infos.insert(name_key, as_constant_info);
                env.write().init_quot();
                Ok(())
            }
            DeclarationKind::MutualDefnDeclar { .. } => {
                unimplemented!()
            }
            DeclarationKind::InductiveDeclar_ { val } => {
                val.add_inductive_fn(&env)
            }
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReducibilityHint {
    Regular(usize),
    Opaque,
    Abbreviation,
}

impl ReducibilityHint {
    pub fn as_usize(&self) -> Option<usize> {
        match self {
            Regular(x) => Some(*x),
            _ => None
        }
    }
   
   pub fn compare(self, other : ReducibilityHint) -> Ordering {
        match (self, other) {
            (Regular(h1), Regular(h2)) if h1 == h2 => Equal,
            (Regular(h1), Regular(h2)) if h1 > h2 => Greater,
            (Regular(h1), Regular(h2)) if h1 < h2 => Less,
            (Opaque, Opaque) | (Abbreviation, Abbreviation) => Equal,
            (Opaque, _) => Less,
            (_, Opaque) => Greater,
            (Abbreviation, _) => Greater,
            (_, Abbreviation) => Less,
            _ => unreachable!()
        }
    }
}

impl Env {
    pub fn init_quot(&mut self) {
        self.quot_is_init = true;
    }
    pub fn get_hint(&self, name : &Name) -> Option<ReducibilityHint> {
        match self.constant_infos.get(name) {
            Some(ConstantInfo::DefinitionInfo(def_val)) => {
                Some(def_val.hint)
            },
            Some(_) => None,
            None => None
        }
    }


}


pub fn check_constant_val_wtc(c : ConstantVal, tc : &mut TypeChecker) -> NanodaResult<()> {
    // FIXME enable this, but it ATM it clashes with the serial
    // parser.
    //tc.env.read().check_name(&c.name);
    ensure_no_dupe_lparams(&c.lparams)?;
    assert!(!c.type_.has_locals());
    //tc.env.read().ensure_no_locals(&c.type_);
    let sort = tc.check(&c.type_, c.lparams.clone());
    tc.ensure_sort(&sort);
    //tc.ensure_sort(&sort, &c.type_);
    Ok(())

}
pub fn check_constant_val_no_tc(env : ArcEnv, c : ConstantVal, safe_only : bool) -> NanodaResult<()> {
    let mut tc = TypeChecker::new(Some(safe_only), env.clone());
    check_constant_val_wtc(c, &mut tc)
}




