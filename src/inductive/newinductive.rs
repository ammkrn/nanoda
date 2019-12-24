use crate::inductive::addinductive::AddInductiveFn;

use crate::name::Name;
use crate::level::Level;
use crate::env::ArcEnv;
use crate::expr::Expr;
use crate::errors::NanodaResult;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InductiveType {
    pub name : Name,
    pub type_ : Expr,
    pub constructors : Vec<Constructor>
}

impl InductiveType {
    pub fn new(name : Name, type_ : Expr, constructors : Vec<Constructor>) -> Self {
        InductiveType {
            name,
            type_,
            constructors
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Constructor {
    pub name : Name,
    pub type_ : Expr,
}


impl Constructor {
    pub fn new(name : &Name, type_ : &Expr) -> Self {
        Constructor {
            name : name.clone(),
            type_ : type_.clone()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InductiveDeclar {
    pub name : Name,
    pub lparams : Vec<Level>,
    pub num_params : usize,
    pub types : Vec<InductiveType>,
    pub is_unsafe : bool,
}



impl InductiveDeclar {
    pub fn new(name : Name,
               lparams : Vec<Level>, 
               num_params : usize, 
               types : Vec<InductiveType>,
               is_unsafe : bool) -> InductiveDeclar {
        InductiveDeclar {
            name,
            lparams,
            num_params,
            types,
            is_unsafe
        }
   }

   pub fn add_inductive_fn(&self, env : &ArcEnv) -> NanodaResult<()> {
       let level_params = self.lparams.clone();
       let is_unsafe = self.is_unsafe;
       let num_params = self.num_params;
       let ind_types = self.types.clone();

       let mut add_inductive = AddInductiveFn::new(self.name.clone(), level_params, num_params, is_unsafe, ind_types, env.clone());
       add_inductive.env_operator()
   }
}

pub fn get_all_inductive_names(v : &Vec<InductiveType>) -> Vec<Name> {
    v.into_iter().map(|d| d.name.clone()).collect::<Vec<Name>>()
}


