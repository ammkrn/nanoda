
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
use crate::inductive::ProtoInd;

use Modification::*;
use CompiledModification::*;


/// 汎用的な「環境へ加えるはず」の物を表す包です。
/// 公理、帰納型の成分、Quotの成分もこのように扱われます。
/// tc::def_height で height について読めます。
#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub name: Name,
    pub univ_params: Arc<Vec<Level>>,
    pub ty: Expr,
    pub height: u16,
    pub builtin: bool,
}

/// `Declaration`（宣言）, `ReductionRule`(縮小規則), `Notation`
/// (記号) を保持する環境です。型検査装置の蓄積となる物です。複数スレッドから
/// 同時にアクセス出来るために、Arc<RwLock<Env>> と対応しますが、Arc<T> が
/// T として deref して、parkinng_lot の RwLock<T> が直接に T を返すこと
/// によって、これらの包を直接に触る必要が殆どないんです。
#[derive(Clone)]
pub struct Env {
    pub declarations: HashMap<Name, Declaration>,
    pub reduction_map: ReductionMap,
    pub notations : HashMap<Name, Notation>,
}

/// 文字通りの物です。公理を表すやつ。
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


/// Lean の `def`/`definition`キーワードで導入される定義を表す物です。
/// 名前、ユニバース引数、型、値を持っています。Lean での `lemma` もこの型で
/// 表されます。　
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



/// この型は環境へ追加して検査する構造です。種類は以下の構成をしています：
///
/// ```pseudo
/// Axiom : 一つだけの環境に追加する `Declaration` を持っています。
/// CompiledDefinition : `Declaration`, `ReductionRule` をそれぞれ一個持ってて、
///                      定義の型・値を表す (Pi, Lambda) のペアも持っています。
///                      型と値が検査された後、捨てられたんです。
/// Quot : 紹介規則を`Declaration`として４つ持ってて (quot, quot.mk, 
///        quot.lift, quot.ind)も一つの縮小規則を持っています。
/// Inductive : ベース型、いくつかの紹介規則、削除規則を `Declaration, Vec<Declaration>,
///             Decalaration というように持っています。縮小規則は Vec<ReductionRule>
///             としても持っています。
/// ```
#[derive(Debug, Clone)]
pub enum CompiledModification {
    CompiledAxiomMod     (Declaration),
    CompiledDefinition   (Declaration, ReductionRule, Expr, Expr),
    //                                                 型    値　
    CompiledQuotMod      (Vec<Declaration>, ReductionRule),
    CompiledInductive    (Declaration, Vec<Declaration>, Declaration, Vec<ReductionRule>),
    //                      (ベース型, 紹介原理　, recursor/削除原理, 縮小規則)
}

#[derive(Clone)]
pub enum Modification {
    AxiomMod (Axiom),
    DefMod   (Definition),
    QuotMod  (Quot),
    IndMod   (ProtoInd),
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

