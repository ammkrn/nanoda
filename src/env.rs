
use std::sync::Arc;
use hashbrown::{ HashMap, HashSet };
use parking_lot::RwLock;

use crate::name::Name;
use crate::level::Level;
use crate::expr::{ Expr, unique_const_names, univ_params_subset, mk_const };
use crate::reduction::{ ReductionRule, ReductionMap };
use crate::quot::Quot;
use crate::inductive::Inductive;
use crate::tc::TypeChecker;
use crate::pretty::components::Notation;

use Modification::*;
use CompiledModification::*;


/// Generic wrapper used to describe items to be added to
/// the environment in some capacity, including axioms, 
/// parts of inductive declarations, and parts of 
/// quotient. See the method `tc::def_height()` for a description
/// of what height is.
#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub name: Name,
    pub univ_params: Arc<Vec<Level>>,
    pub ty: Expr,
    pub height: u16,
    pub builtin: bool,
}

/// Environment containing the declarations, reduction rules, 
/// and notations that make up the context for a set of Lean 
/// items. Essentially, "the place where everything goes", and
/// "the place you go to get stuff". We interact with this
/// through an atomically reference counted RwLock so we can 
/// interact with it from different threads, but because Arc<RwLock<T>>
/// dereferences to <T>, and ParkingLot's RwLock implementation
/// doesn't need to return a result, that part of it is usually 
/// transparent.
#[derive(Clone)]
pub struct Env {
    pub declarations: HashMap<Name, Declaration>,
    pub reduction_map: ReductionMap,
    pub notations : HashMap<Name, Notation>,
}

/// What you see is what you get. Has a name, a vector of universe
/// parameters, and its type.
#[derive(Clone)]
pub struct Axiom {
    pub name : Name,
    pub univ_params : Arc<Vec<Level>>,
    pub ty : Expr
}


impl Axiom {
    pub fn new(name : Name, univ_params : Arc<Vec<Level>>, ty : Expr) -> Self {
        Axiom {
            name,
            univ_params,
            ty
        }
    }
}


/// Lean definition, as you would introduce with the `def` or `definition`
/// keywords. Has a name, universe parameters, a type,  and a value. Lemmas
/// are also considered definitions. Follows the pattern: 
#[derive(Debug, Clone)]
pub struct Definition {
    pub name : Name,
    pub univ_params : Arc<Vec<Level>>,
    pub ty : Expr,
    pub val : Expr
}

impl Definition {
    pub fn new(name : Name, 
               univ_params : Arc<Vec<Level>>, 
               ty : Expr, 
               val : Expr) -> Self {
        Definition {
            name,
            univ_params,
            ty,
            val
        }
    }

}


impl Declaration {
    pub fn mk(name: Name,
               univ_params: Arc<Vec<Level>>,
               ty: Expr,
               height: Option<u16>,
               builtin: Option<bool>)
               -> Self {
        Declaration {
            name,
            univ_params,
            ty,
            height : height.unwrap_or(0u16),
            builtin : builtin.unwrap_or(false)
        }
    }

    pub fn to_axiom(&self) -> Modification {
        assert!(self.univ_params.iter().all(|x| x.is_param()));
        Modification::AxiomMod(Axiom::new(self.name.clone(), self.univ_params.clone(), self.ty.clone()))
    }

    pub fn indep_declaration_check(&self, env : Arc<RwLock<Env>>) {
        let mut tc = TypeChecker::new(None, env);
        self.declaration_check(&mut tc);
    }

    pub fn declaration_check(&self, tc : &mut TypeChecker) {
        assert!(univ_params_subset(&self.ty, &self.univ_params
                                                  .iter()
                                                  .collect::<HashSet<&Level>>()));
        assert!(!self.ty.has_vars());
        assert!(!self.ty.has_locals());
        tc.infer_universe_of_type(&self.ty);
    }


}



/** This is the thing we actually add to the environment and type check.
 They have the following strucutres :
 Axiom : Has one `Declaration` to add to the environment.
 CompiledDefinition : Has one `Declaration`, one `ReductionRule`, as well
                      as a type (a pi expr) and a value (a lambda expr).
                      The latter two are only type checked, not added to
                      the environment.
 Quot : Has four Declarations rules (quot, quot.mk, quot.lift, quot.ind)
        and one reduction rule.
 Inductive : Has its base type as a `Declaration`, a sequence of `Declaration`
             items representing its introduction rules, a `Declaration`            
             representing its elimination rule, and a sequence of 
             `ReductionRule`s. */
#[derive(Debug, Clone)]
pub enum CompiledModification {
    CompiledAxiomMod     (Declaration),
    CompiledDefinition   (Declaration, ReductionRule, Expr, Expr),
    //                                              Type, and Value
    CompiledQuotMod      (Vec<Declaration>, ReductionRule),
    CompiledInductive    (Declaration, Vec<Declaration>, Declaration, Vec<ReductionRule>),
    // (base_type_axiom, intro_declarations, elim_declaration(rec), reduction_rules)
}


/// As with the other types, we have to wrap these in a way that feels
/// a little bit excessive to get the behavior we want, which is that
/// functions and collections can sometimes accept any `Modification`,
/// and other times discriminate between IE a `DefMod` and an `Inductive`.
/// We can't use a trait to tie everything together since we need
/// to have collections of Modifications, and heterogeneous collections
/// built over traits of different types would mean a large performance
/// hit from dynamic dispatch.
#[derive(Clone)]
pub enum Modification {
    AxiomMod (Axiom),
    DefMod   (Definition),
    QuotMod  (Quot),
    IndMod   (crate::inductive::ProtoInd),
}


impl Env {
    pub fn new(num_mods : usize) -> Self {
        Env {
            declarations : HashMap::with_capacity(num_mods),
            reduction_map : ReductionMap::new(num_mods),
            notations : HashMap::with_capacity(500)
        }
    }

    pub fn get_declaration_height(&self, name : &Name) -> Option<u16> {
        self.declarations.get(name).map(|dec| dec.height)
    }

    pub fn insert_declaration(&mut self, d : Declaration) {
        self.declarations.insert(d.name.clone(), d);
    }

    pub fn insert_reduction_rule(&mut self, r : ReductionRule) {
        self.reduction_map.add_rule(r);
    }

    pub fn get_value(&self, n : &Name) -> Option<&Expr> {
        self.reduction_map.get_value(n)
    }

    pub fn add_notation(&mut self, n : &Name, notation: Notation) {
        match self.notations.get(n) {
            Some(_) => (),
            None => { self.notations.insert(n.clone(), notation); }
        }
    }

    pub fn num_declars(&self) -> usize {
        self.declarations.len()
    }



}

impl Modification {
    pub fn compile(self, env : &Arc<RwLock<Env>>) -> CompiledModification {
        match self {
            AxiomMod(axiom) => {
                let derived_declaration = Declaration::mk(axiom.name,
                                                          axiom.univ_params,
                                                          axiom.ty,
                                                          None,
                                                          None);
                CompiledAxiomMod(derived_declaration)
            },
            DefMod(def) => {
                let height = 
                    match unique_const_names(&def.val)
                          .iter()
                          .filter_map(|name| env.read().get_declaration_height(&name))
                          .max() {
                              Some(h) => h + 1,
                              None => 1
                          };
                let derived_declaration = 
                    Declaration::mk(def.name.clone(),
                                    def.univ_params.clone(),
                                    def.ty.clone(),
                                    Some(height),
                                    None);
                let derived_reduction_rule = 
                    ReductionRule::new_rr(mk_const(def.name, def.univ_params),  
                                          def.val.clone(),
                                          Vec::new());
                CompiledDefinition(derived_declaration, 
                                   derived_reduction_rule, 
                                   def.ty,
                                   def.val)
            },
            QuotMod(quot) => quot.compile_self(),
            IndMod(ind) => {
                let ind = Inductive::new(
                    ind.name,
                    ind.params,
                    ind.ty,
                    ind.num_params,
                    ind.intros,
                    env.clone()
                );
                ind.compile(&env.clone())
            }

        }
    } 
}




impl CompiledModification {
   // All this does is add the (as of yet unchecked) item to 
   // the environment. We then have to come back and check it later. 
   pub fn add_only(&self, env : &Arc<RwLock<Env>>) {
       let mut write_guard = env.write();
        match self {
            CompiledAxiomMod(declaration) => {
                write_guard.insert_declaration(declaration.clone());
            },
            CompiledDefinition(declaration, rule, ..) => {
                write_guard.insert_declaration(declaration.clone());
                write_guard.insert_reduction_rule(rule.clone());
            },
            CompiledQuotMod(declarations, rule) => {
                for d in declarations {
                    write_guard.insert_declaration(d.clone());
                }
                write_guard.insert_reduction_rule(rule.clone());
            },
            CompiledInductive(_base_type, intros, elim_declaration, reductions) => {
                for d in intros {
                    write_guard.insert_declaration(d.clone());
                }

                write_guard.insert_declaration(elim_declaration.clone());

                for r in reductions {
                    write_guard.insert_reduction_rule(r.clone())
                }

            }
        }
    }

    // Checks a given item.
    pub fn check_only(&self, env : &Arc<RwLock<Env>>) {
        match self {
            CompiledAxiomMod(declaration) => {
                let mut tc = TypeChecker::new(None, env.clone());
                declaration.declaration_check(&mut tc);
            },
            CompiledDefinition(declaration, _, ty, val) => {
                let mut tc = TypeChecker::new(None, env.clone());
                declaration.declaration_check(&mut tc);
                tc.check_type(val, ty);
            },
            CompiledQuotMod(declarations, _) => {
                for d in declarations {
                    d.indep_declaration_check(env.clone());
                }
            },
            CompiledInductive(base_type, intros, ..) => {
                for d in Some(base_type).into_iter().chain(intros.into_iter()) {
                    d.indep_declaration_check(env.clone());
                }
            }
        }
    }
}

