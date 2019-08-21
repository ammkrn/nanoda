use std::hash::{ Hash, Hasher };
use std::sync::Arc;

use fxhash::hash64;
use hashbrown::HashMap;

use crate::name::Name;
use crate::level::Level;
use crate::expr::{ Expr, InnerExpr::* };
use crate::errors;


/// (ReductionRule, [(Level, Level)]) の鍵を Expr の値までマップするものです。
/// タスクは、「この RecutionRule はこれらのユニバース置換を適用したことがありますか?
///  やったことあったら、カッシュされた結果の Expr を返すだけでいい」ってことだ。
#[derive(Clone)]
pub struct ReductionCache {
    pub inner : HashMap<(ReductionRule, Vec<(Level, Level)>), Expr>
}

impl ReductionCache {
    pub fn with_capacity(n : usize) -> Self {
        ReductionCache {
            inner : HashMap::with_capacity(n)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReductionRule {
    pub lhs_const_name: Name,
    pub lhs: Expr,
    pub rhs: Expr,
    pub def_eq_constraints: Arc<Vec<(Expr, Expr)>>,
    pub lhs_var_bound: u16,
    pub lhs_args_size: usize,
    pub majors : Vec<usize>,
    pub digest : u64,
}

impl Hash for ReductionRule {
    fn hash<H : Hasher>(&self, state : &mut H) {
        self.digest.hash(state);
    }
}

/// ReductionRule って２つのコンストラクターがあります。`new_nondef_rr` って
/// Inductive と Quotient に用いられて new_rr を呼ぶ前の準備をするものだけです。
/// `Definition` アイテムは直接に `new_rr` を呼べます。
impl ReductionRule {
    pub fn new_rr(lhs : Expr, rhs : Expr, def_eq_constraints : Vec<(Expr, Expr)>) -> Self {
        let lhs_var_bound = lhs.var_bound();
        assert!(!lhs.has_locals());
        assert!(!rhs.has_locals());
        assert!(rhs.var_bound() <= lhs_var_bound);

        let (app_fn, lhs_args) = lhs.unfold_apps_refs();
        let lhs_args_size = lhs_args.len();
        let lhs_const_name = match app_fn.as_ref() {
            Const(_, name, _) => name.clone(),
            owise => errors::err_rr_const(line!(), owise),
        };

        // IE : Vec[Var(9), Const(..), Var(19), Var(11), Sort(..)]
        // って Vec[0, 2, 3] になる。　
        let majors = lhs_args.iter().rev().enumerate().filter_map(|(idx, arg)| {
            match arg.as_ref() {
                Var(..) => None,
                _ => Some(idx)
            }
        }).collect::<Vec<usize>>();

        let digest = hash64(&(&lhs.get_digest(), &rhs.get_digest()));

        ReductionRule {
            lhs_const_name,
            lhs,
            rhs,
            def_eq_constraints : Arc::new(def_eq_constraints),
            lhs_var_bound,
            lhs_args_size : lhs_args_size,
            majors : majors,
            digest
        }
    }

    pub fn new_nondef_rr<'r, R>(locals : &[Expr], 
                                lhs : Expr, 
                                rhs : Expr, 
                                def_eq_constraints : R) -> Self
      where R : Iterator<Item = (&'r Expr, &'r Expr)> {
        let lhs_abstd = lhs.abstract_(locals.into_iter());
        let rhs_abstd = rhs.abstract_(locals.into_iter());

        let def_eq_constraints_abstd = def_eq_constraints.map(|(a, b)| {
            let c1_a = a.abstract_(locals.into_iter());
            let c2_a =  b.abstract_(locals.into_iter());
            (c1_a, c2_a)
        }).collect::<Vec<(Expr, Expr)>>();

        ReductionRule::new_rr(lhs_abstd, rhs_abstd, def_eq_constraints_abstd)
    }

    pub fn collect_substs<'l, 's>(&self, 
                                  e1 : &'l Expr, 
                                  e2 : &'l Expr, 
                                  var_subs  : &'s mut Vec<&'l Expr>, 
                                  univ_subs : &mut Vec<(Level, Level)>) -> bool {
        match (e1.as_ref(), e2.as_ref()) {
            (App(_, lhs1, rhs1), App(_, lhs2, rhs2)) => {
                self.collect_substs(lhs1, lhs2, var_subs, univ_subs)
                && self.collect_substs(rhs1, rhs2, var_subs, univ_subs)
            },
            (Const(.., n1, lvls1), Const(.., n2, lvls2)) if n1 == n2 => {
                for (lhs, rhs) in lvls1.as_ref().clone().into_iter()
                                       .zip(lvls2.as_ref().clone()) {
                    univ_subs.push((lhs, rhs));
                }
                true
            },
            (Var(_, idx), _) => {
                match var_subs.get_mut(*idx as usize) {
                    Some(already) => { std::mem::replace(already, e2); },
                    None => {
                        // この配列の位置が全部満たされますが、順序が保証されてないから、
                        // (2) (5) っていうような連続がきたら、(1), (3) (4) をパッドしなきゃ。
                        while var_subs.len() < *idx as usize {
                            var_subs.push(e2);
                        }
                        var_subs.push(e2);
                        assert!(var_subs.len() == (*idx as usize) + 1);
                    }
                }
                true
            },
            _ => false
        }        
    }

    pub fn apply_reduction<'l>(&self, 
                                   e : Expr,
                                   cache : &mut ReductionCache) 
                                   -> Option<(Expr, Vec<(Expr, Expr)>)> {
        let mut var_subs = Vec::<&'l Expr>::with_capacity(100);
        let mut univ_subs = Vec::with_capacity(100);

        if !self.collect_substs(&self.lhs, &e, &mut var_subs, &mut univ_subs) {
            return None
        }

        let cached_or_new = match cache.inner.get(&(self.clone(), univ_subs.clone())) {
            Some(cached) => cached.clone(),
            None => {
                let new_cache_val = self.rhs.instantiate_ps(&univ_subs);
                cache.inner.insert((self.clone(), univ_subs.clone()), new_cache_val.clone());
                new_cache_val
            }
        };

        if self.lhs_var_bound == 0 {
            Some((cached_or_new, self.def_eq_constraints.as_ref().clone()))
        } else {
            let instd_base = cached_or_new.instantiate(var_subs.iter().cloned());
            let instd_constraints = 
                    self.def_eq_constraints.iter()
                                           .map(|(i, j)| {
                let i_ = i.instantiate(var_subs.iter().cloned());
                let j_ = j.instantiate(var_subs.iter().cloned());
                (i_, j_)
            }).collect::<Vec<(Expr, Expr)>>();

            Some((instd_base, instd_constraints))
        }
    }

    pub fn apply_hd_tl(&self, hd : &Expr, apps : &[Expr], cache : &mut ReductionCache) -> Option<(Expr, Vec<(Expr, Expr)>)> {
        if apps.len() < self.lhs_args_size {
            return None
        } 

        let (apps_l, apps_r) = apps.split_at(self.lhs_args_size);
        let applied = hd.fold_apps(apps_l);
        self.apply_reduction(applied, cache)
            .map(|(reduc, cs)| {
                let applied = reduc.fold_apps(apps_r);
                (applied, cs)
            })
    }

}

#[derive(Clone)]
pub struct ReductionMap {
    pub reduction_rules : HashMap<Name, Vec<ReductionRule>>,
    major_premises : HashMap<Name, Vec<usize>>
}


impl ReductionMap {

    pub fn new(num_mods : usize) -> Self {
        ReductionMap {
            reduction_rules : HashMap::with_capacity(num_mods),
            major_premises : HashMap::with_capacity(num_mods),
        }
    }

    pub fn get_value(&self, n : &Name) -> Option<&Expr> {
        for elem in self.reduction_rules.get(n)? {
            match elem.lhs.as_ref() {
                Const(..) => return Some(&elem.rhs),
                _ => continue
            }
        };
        // ループが早く返せる値なんて見つけられなかったら。。。　
        return None
    }
    
    pub fn apply_to_map(&self, 
                        e : Expr, 
                        cache : &mut ReductionCache) -> Option<(Expr, Vec<(Expr, Expr)>)> {
        let (hd, apps) = e.unfold_apps_special(); 

        if let Const(_, name, _) = hd.as_ref() {
            let source = self.reduction_rules.get(&name).cloned()?; 
            for elem in source {
                match elem.apply_hd_tl(&hd, apps.as_slice(), cache) {
                    found @ Some(_) => return found,
                    None => continue
                }
            }
            return None
        } else {
            return None
        }
    }

    pub fn add_rule(&mut self, new_rule : ReductionRule) {
        let name_key = new_rule.lhs_const_name.clone();
        let major_prem_vec = new_rule.majors.clone();
        match self.reduction_rules.get_mut(&name_key) {
            Some(already_rules) => {
                already_rules.push(new_rule);
                self.major_premises.get_mut(&name_key)
                           .unwrap_or_else(|| errors::err_add_rule(line!(), &name_key))
                           .extend(major_prem_vec.into_iter());
            },
            None => {
                let res1 = self.reduction_rules.insert(name_key.clone(), vec![new_rule]);
                let res2 = self.major_premises.insert(name_key.clone(), major_prem_vec);
                // 既にマップに入ってなかったことを主張する
                assert!(res1.is_none() && res2.is_none());
            }
        }
    }

    pub fn get_major_premises(&self, key : &Name) -> Option<&Vec<usize>> {
        self.major_premises.get(key)
    }


}
