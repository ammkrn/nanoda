use crate::name::Name;
use crate::level::Level;
use crate::env::ConstantVal;
use crate::expr::{ Expr, mk_var, InnerExpr::* };





#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RecInfo {
    pub m_C : Expr,
    pub m_minors : Vec<Expr>,
    pub m_indices : Vec<Expr>,
    pub m_major : Expr
}

impl RecInfo {
    pub fn new(m_C : Expr, m_minors : Vec<Expr>, m_indices : Vec<Expr>, m_major : Expr) -> Self {
        RecInfo {
            m_C,
            m_minors,
            m_indices,
            m_major
        }
    }

    // FIXME This is terrible.
    pub fn is_init(&self) -> bool {
        let big_var = mk_var(std::usize::MAX);
        let empty_vec = Vec::<Expr>::new();
        assert!((&self.m_C) != (&big_var));
        assert!((&self.m_minors) != (&empty_vec));
        assert!((&self.m_indices) != (&empty_vec));
        assert!((&self.m_major) != (&big_var));
        true
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecursorRule {
    pub constructor : Name,
    pub nfields : usize,
    pub rhs : Expr,
}

impl RecursorRule {
    // name is of the form `<base>.mk`
    pub fn new(constructor : Name, nfields : usize, rhs : Expr) -> Self {
        let result = RecursorRule {
            constructor,
            nfields,
            rhs
        };
        result
    }

    pub fn get_nfields(&self) -> usize {
        self.nfields
    }

    pub fn get_cnstr(&self) -> &Name {
        &self.constructor
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecursorVal {
    pub constant_val : ConstantVal,
    pub lparam_names : Vec<Name>,
    pub all : Vec<Name>,
    pub nparams : usize,
    pub nindices : usize,
    pub nmotives : usize,
    pub nminors : usize,
    pub rules : Vec<RecursorRule>,
    pub is_k : bool,
    pub is_unsafe : bool
}

impl RecursorVal {
    pub fn new(name : Name,
               lparams : Vec<Level>,
               lparam_names : Vec<Name>,
               type_ : Expr, 
               all : Vec<Name>,
               nparams : usize,
               nindices : usize,
               nmotives : usize,
               nminors : usize,
               rules : Vec<RecursorRule>,
               is_k : bool,
               is_unsafe : bool) -> Self {

        let constant_val = ConstantVal::new(name.clone(), lparams, type_);

        let result = RecursorVal {
            constant_val,
            lparam_names,
            all,
            nparams,
            nindices,
            nmotives,
            nminors,
            rules,
            is_k,
            is_unsafe
        };

        result
    }

    pub fn get_major_idx(&self) -> usize {
        self.nparams + self.nmotives + self.nminors + self.nindices
    }

    pub fn get_induct(&self) -> &Name {
        self.constant_val.name.get_prefix()
    }

    pub fn get_rec_rule_for(&self, major : &Expr) -> Option<RecursorRule> {
        if let Const { name, .. } = major.unfold_apps_fn().as_ref() {
            self.rules.iter().find(|r| r.get_cnstr() == name).cloned()
        } else {
            None
        }
    }
}








