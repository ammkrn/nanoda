
use std::sync::Arc;
use hashbrown::HashMap;
use parking_lot::RwLock;
use stacker::maybe_grow;

use crate::utils::{ ShortCircuit, ShortCircuit::*, EqCache };
use crate::name::Name;
use crate::level::{ Level, mk_imax, mk_succ };
use crate::expr::{ Expr, Binding, InnerExpr::*, mk_app, mk_lambda, mk_var, mk_sort, mk_prop, mk_pi };
use crate::reduction::ReductionCache;
use crate::env::Env;
use crate::errors::*;
use Flag::*;


/// "A Typechecker" is just a collection of caches and a handle to the current
/// environment (we only ever need to read from it in this case). 
/// unsafe_unchecked should be true iff the TypeChecker will only ever
/// be used by the pretty printer.
#[derive(Clone)]
pub struct TypeChecker {
    unsafe_unchecked: bool,
    pub infer_cache : HashMap<Expr, Expr>,
    pub eq_cache : EqCache,
    pub whnf_cache : HashMap<Expr, Expr>,
    pub reduction_cache : ReductionCache,
    pub env : Arc<RwLock<Env>>,
}

impl std::fmt::Debug for TypeChecker {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "<typechecker>")
    }
}

impl TypeChecker {
    pub fn new(unsafe_unchecked : Option<bool>, env : Arc<RwLock<Env>>) -> Self {
        TypeChecker {
            unsafe_unchecked : unsafe_unchecked.unwrap_or(false),
            infer_cache : HashMap::with_capacity(1000),
            eq_cache : EqCache::with_capacity(500),
            whnf_cache : HashMap::with_capacity(100),
            reduction_cache : ReductionCache::with_capacity(100),
            env
        }
    }

    pub fn fork_env(&self) -> Arc<RwLock<Env>> {
        self.env.clone()
    }

    pub fn should_check(&self) -> bool {
        !self.unsafe_unchecked
    }

    /// The "heights" of two terms `E1` and `E2` are used to determine whether one 
    /// is defined in terms of or uses terms derived from the other. If at some point 
    /// we need to unify `E1 == E2`, we want to unfold the HIGHER one FIRST, since 
    /// it will eventually unfold into (something resembling) the lower term, 
    /// whereas continued unfolding of the lower term will just get us more and 
    /// more primitive terms that get further away from the goal.
    /// Thanks to @Gebner for explaining this to me.
    fn def_height(&self, _fn : &Expr) -> u16 {
        if let Const(_, name, _) = _fn.as_ref() {
            self.env.read()
                    .declarations
                    .get(name)
                    .map(|h| h.height + 1)
                    .unwrap_or(0u16)
        } else {
            0u16
        }
    }

    /// e is a prop iff it destructures as Sort(Level(Zero))
    pub fn is_prop(&mut self, e : &Expr) -> bool {
        match self.whnf(e).as_ref() {
            Sort(_, lvl) => lvl.is_zero(),
            _ => false
        }
    }

    /// tries is_prop after inferring e
    pub fn is_proposition(&mut self, e : &Expr) -> bool {
        let inferred = self.infer(e);
        self.is_prop(&inferred)
    }

    pub fn is_proof(&mut self, p: &Expr) -> bool {
        let inferred = self.infer(p);
        self.is_proposition(&inferred)
    }

    fn is_proof_irrel_eq(&mut self, e1: &Expr, e2: &Expr) -> bool {
        self.is_proof(e1) && self.is_proof(e2)
    }


    /// More aggressive version of `unfold_pis`. Given some term `E`, repeats  
    /// `{ apply whnf(e), then unfold_pis(e) }` until that combination 
    /// fails to strip any more binders out.
    pub fn normalize_pis(&mut self, e : &Expr) -> (Expr, Vec<Expr>) {
        let mut collected_binders = Vec::new();
        let mut acc = e.clone();

        loop {
            let len_before = collected_binders.len();
            acc = self.whnf(&acc);
            acc.unfold_pis(&mut collected_binders);
            if len_before == collected_binders.len() {
                break
            }
        }

        (acc, collected_binders)
    }

    // This only gets used once in inductive. Will use &[Expr]
    // that comes as `toplevel_params` used during formation of intro rules.
    // I'm not really sure how the length of the subst sequence corresponds
    // to the number of times whnf is supposed to be executed to be honest.
    pub fn instantiate_pis(&mut self, intro_type : &Expr, toplevel_intro_params : &[Expr]) -> Expr {
        let mut iterations_left = toplevel_intro_params.len();
        let mut acc = intro_type.clone();

        while iterations_left > 0 {
            match acc.as_ref() {
                Pi(.., body) => {
                    iterations_left -= 1;
                    acc = body.clone();
                },
                _ => { 
                    acc = self.whnf(&acc);
                    // assert that the result is a Pi
                    assert!(match acc.as_ref() {
                        Pi(..) => true,
                        _ => false
                    });
                }
            }
        }

        acc.instantiate(toplevel_intro_params.into_iter().rev())
    }

    /// Outward facing function/entry point for reduction to weak head normal form. 
    /// Checks cache for a previous result, calling whnf_core on a cache miss.
    pub fn whnf(&mut self, e : &Expr) -> Expr {
        if let Some(cached) = self.whnf_cache.get(e) {
            return cached.clone()
        } else {
            let cache_key = e.clone();
            let result = self.whnf_core(e, Some(FlagT));
            self.whnf_cache.insert(cache_key, result.clone());
            result
        }
    }

    pub fn whnf_core(&mut self, e : &Expr, _flag : Option<Flag>) -> Expr {
        let flag = _flag.unwrap_or(FlagT);
        let (_fn, apps) = e.unfold_apps_refs();

        match _fn.as_ref() {
            Sort(_, lvl) => {
                let simpd = lvl.simplify();
                mk_sort(simpd)
            },
            Lambda(..) if !apps.is_empty() => {
                let intermed = self.whnf_lambda(_fn, apps);
                self.whnf_core(&intermed, Some(flag))
            },
            Let(.., val, body) => {
                let instd = body.instantiate(Some(val).into_iter());
                let applied = instd.fold_apps(apps.into_iter().rev());
                self.whnf_core(&applied, Some(flag))
            },
            _ => {
                let reduced = self.reduce_hdtl(_fn, apps.as_slice(), Some(flag));
                match reduced {
                    Some(eprime) => self.whnf_core(&eprime, Some(flag)),
                    None => e.clone()
                }
            }
        }
    }

    pub fn whnf_lambda(&mut self, 
                   mut f : &Expr, 
                   mut apps : Vec<&Expr>) -> Expr {
        let mut ctx = Vec::with_capacity(apps.len());

        while let Lambda(_, _, fn_) = f.as_ref() {
            if let Some(hd) = apps.pop() {
                ctx.push(hd);
                f = fn_;
                continue
            } else {
                break
            }
        }

        f.instantiate(ctx.into_iter().rev())
         .fold_apps(apps.into_iter().rev())
    }

    /// The entry point for executing a single reduction step on two
    /// expressions.
    pub fn reduce_exps(&mut self, e1 : Expr, e2 : Expr, flag : Option<Flag>) -> Option<(Expr, Expr)> {
        assert!(flag == Some(FlagT));
        let (fn1, apps1) = e1.unfold_apps_refs();
        let (fn2, apps2) = e2.unfold_apps_refs();

        // we want to evaluate these lazily.
        let red1 = |tc : &mut TypeChecker| tc.reduce_hdtl(fn1, apps1.as_slice(), flag).map(|r| (r, e2.clone()));
        let red2 = |tc : &mut TypeChecker| tc.reduce_hdtl(fn2, apps2.as_slice(), flag).map(|r| (e1.clone(), r));

        if self.def_height(fn1) > self.def_height(fn2) {
            red1(self).or(red2(self))
        } else {
            red2(self).or(red1(self))
        }
    }


    pub fn reduce_hdtl(&mut self, _fn : &Expr, apps : &[&Expr], flag : Option<Flag>) -> Option<Expr> {

        if let Some(FlagF) = flag {
            return None
        }

        let name : &Name = match _fn.as_ref() {
            Const(_, name, _) => (name),
            _ => return None
        };

        let major_prems = self.env
                              .read()
                              .reduction_map
                              .get_major_premises(&name)
                              .cloned();

        let mut collected = Vec::with_capacity(apps.len());
        
        for (idx, elem) in apps.into_iter().rev().enumerate() {
            if major_prems
                   .as_ref()
                   .map(|set| set.contains(&idx))
                   .unwrap_or(false) {
                       collected.push(self.whnf(&elem));
                   } else {
                       collected.push(elem.clone().clone());
                   }
        }

        let applied = _fn.fold_apps(collected.iter()); 
        let (result, constraints) = self.env
                                        .read()
                                        .reduction_map
                                        .apply_to_map(applied, &mut self.reduction_cache)?;

        match constraints.iter()
                         .all(|(a, b)| self.def_eq(a, b)) {
                             true => Some(result),
                             false => None
                         }
    }


    fn def_eq(&mut self, a : &Expr, b : &Expr) -> bool {
        if self.check_def_eq(a, b) == EqShort {
            return true
        } else {
            return false
        }
    }


    /// only used in `check_def_eq_patterns`. Broken out
    /// to prevent `patterns` from getting too big/hard to read.
    pub fn apps_eq(&mut self, 
                   apps1 : Vec<&Expr>, 
                   apps2 : Vec<&Expr>) -> ShortCircuit {
        if apps1.len() != apps2.len() {
            return NeqShort
        } else {
            for (a, b) in apps1.iter().zip(apps2).rev() {
                let closure = maybe_grow(64 * 1024, 1024 * 1024, || self.check_def_eq(a, b));
                if closure == EqShort {
                    continue
                } else {
                    return NeqShort
                }
            }
            EqShort
        }
    }

    /// Main entry point for checking definitional equality of two terms, which 
    /// dispatches out into a number of different functions. 
    /// 1. `check_def_eq_core` does some destructuring and reduction to weak head
    ///     normal form.
    /// 2. `check_def_eq_patterns` just consults a big list of cases/patterns 
    ///     to determine which decision procedure it needs to use move forward.
    /// 3. `patterns` may call `check_def_eq_pi/lambda` to determine whether
    ///     a pair of Pi or Lambda expressions are definitionally equal.
    pub fn check_def_eq(&mut self, e1 : &Expr, e2 : &Expr) -> ShortCircuit {
        // checks for both pointer and structural equality
        if e1 == e2 {
            return EqShort
        } 
        
        // check whether this equality has been seen before.
        if let Some(cached) = self.eq_cache.get(&e1, &e2) {
            return cached
        }

        // otherwise, compute a result, then cache it in case we see these terms again.
        let result = if self.is_proof_irrel_eq(e1, e2) {
            EqShort
        } else {
           self.check_def_eq_core(e1, e2)
        };

        self.eq_cache.insert(e1.clone(), e2.clone(), result);
        result
    }



    /// Dispatch point for different decision procedures used to determine
    /// whether two expressions are definitionally equal in a certain context.
    pub fn check_def_eq_patterns(&mut self, whnfd_1 : &Expr, whnfd_2 : &Expr) -> ShortCircuit {
        let (fn1, apps1) = whnfd_1.unfold_apps_refs();
        let (fn2, apps2) = whnfd_2.unfold_apps_refs();

        match (fn1.as_ref(), fn2.as_ref()) {
            (Sort(_, l1), Sort(_, l2)) => 
            match apps1.is_empty() && apps2.is_empty() {
                    true => match Level::eq_by_antisymm(l1, l2) {
                        true => EqShort,
                        false => NeqShort,
                    }
                    _ => NeqShort
            },
            (Const(_, n1, lvls1), Const(_, n2, lvls2)) => {
                if n1 == n2 && lvls1.iter().zip(lvls2.as_ref()).all(|(a, b)| Level::eq_by_antisymm(a, b)) {
                    self.apps_eq(apps1, apps2)
                } else {
                    NeqShort
                }
            },
            (Local(.., of1), Local(.., of2)) => {
                if of1 == of2 {
                    self.apps_eq(apps1, apps2)
                } else {
                    NeqShort
                }
            },
            (Lambda(..), Lambda(..)) => self.check_def_eq_lambdas(fn1, fn2),
            (Lambda(_, dom,  _), _) => {
                assert!(apps1.is_empty());
                let app = mk_app(whnfd_2.clone(), mk_var(0));
                let new_lam = mk_lambda(dom.clone(), app);
                self.check_def_eq_core(fn1, &new_lam)
            },
            (_, Lambda(_, dom, _)) => {
                let app = mk_app(whnfd_1.clone(), mk_var(0));
                let new_lam = mk_lambda(dom.clone(), app);
                self.check_def_eq_core(&new_lam, fn2)
            },
            (Pi(..), Pi(..)) => self.check_def_eq_pis(fn1, fn2),
            _ => NeqShort
        }
    }



    pub fn check_def_eq_core(&mut self, e1_0 : &Expr, e2_0 : &Expr) -> ShortCircuit {

        let whnfd_1 = self.whnf_core(e1_0, Some(FlagF));
        let whnfd_2 = self.whnf_core(e2_0, Some(FlagF));

        // consult different patterns laid out in 
        // check_def_eq_patterns to see how to proceed
        match self.check_def_eq_patterns(&whnfd_1, &whnfd_2) {
            EqShort => return EqShort,
            NeqShort => {
                match self.reduce_exps(whnfd_1, whnfd_2, Some(FlagT)) {
                    Some((red1, red2)) => self.check_def_eq_core(&red1, &red2),
                    _ => return NeqShort
                }
            },
            _ => unreachable!()
        }
    }


    // Literally the same function as its Lambda counterpart, but checks for a different
    // enum discriminant (Pis instead of Lambdas).
    pub fn check_def_eq_pis(&mut self, mut e1 : &Expr, mut e2 : &Expr) -> ShortCircuit {

        let mut substs = Vec::new();

        // weird rust syntax; just means 'for as long as e1 and e2 
        // are both Pi terms, keep executing the code in this block"
        while let (Pi(_, dom1, body1), Pi(_, dom2, body2)) = (e1.as_ref(), e2.as_ref()) {
            let mut lhs_type = None;

            if dom1 != dom2 {
                let instd_d2_ty = dom2.ty.instantiate(substs.iter().rev());
                let instd_d1_ty = dom1.ty.instantiate(substs.iter().rev());

                lhs_type = Some(dom2.clone().swap_ty(instd_d2_ty.clone()));
                // If the domains are found not to be equal, return early
                // with NeqShort since the whole thing is therefore not equal
                if !self.def_eq(&instd_d1_ty, &instd_d2_ty) {
                    return NeqShort
                }
            }

            if (body1.has_vars() || body2.has_vars()) {
                let new_local = match lhs_type {
                    Some(elem) => elem.as_local(),
                    None => {
                        let mut _x = dom2.clone();
                        let new_ty = _x.ty.instantiate(substs.iter().rev());
                        _x.swap_ty(new_ty).as_local()
                    }
                };
                substs.push(new_local);
            }  else { 
                substs.push(mk_prop()) 
            }

            e1 = body1;
            e2 = body2;
        }

        match self.def_eq(&e1.instantiate(substs.iter().rev()), 
                          &e2.instantiate(substs.iter().rev())) {
            true => EqShort,
            false => NeqShort
        }
    }


    // Literally the same function as its Pi counterpart, but checks for a different
    // enum discriminant (Lambdas instead of Pis).
    pub fn check_def_eq_lambdas(&mut self, mut e1 : &Expr, mut e2 : &Expr) -> ShortCircuit {
        let mut substs = Vec::new();

        // weird rust syntax; just means "for as long as e1 and e2 
        // are both Lambda terms, keep executing the code in this block"
        while let (Lambda(_, dom1, body1), Lambda(_, dom2, body2)) = (e1.as_ref(), e2.as_ref()) {
            let mut lhs_type = None;

            if dom1 != dom2 {
                let instd_d2_ty = dom2.ty.instantiate(substs.iter().rev());
                let instd_d1_ty = dom1.ty.instantiate(substs.iter().rev());

                lhs_type = Some(dom2.clone().swap_ty(instd_d2_ty.clone()));
                // If the lambda domains are found not to be equal, return early
                // with NeqShort since the whole thing is therefore not equal
                if !self.def_eq(&instd_d1_ty, &instd_d2_ty) {
                    return NeqShort
                }
            }

            if (body1.has_vars() || body2.has_vars()) {
                let new_local = match lhs_type {
                    Some(elem) => elem.as_local(),
                    None => {
                        let mut _x = dom2.clone();
                        let new_ty = _x.ty.instantiate(substs.iter().rev());
                        _x.swap_ty(new_ty).as_local()
                    }
                };
                substs.push(new_local);
            }  else { 
                substs.push(mk_prop()) 
            }

            e1 = body1;
            e2 = body2;

           }

        match self.def_eq(&e1.instantiate(substs.iter().rev()), 
                          &e2.instantiate(substs.iter().rev())) {
            true => EqShort,
            false => NeqShort
        }
    }



    /// Main dispatch point for type inference. Attempts to return early
    /// by checking a cache of previously inferred terms. 
    /// Some of the methods are fairly long so they've been broken out 
    /// into separate functions, trusting in the compiler to inline 
    /// where appropriate.
    pub fn infer(&mut self, term : &Expr) -> Expr {
        if let Some(cached) = self.infer_cache.get(&term) {
            return cached.clone()
        }

        let cache_key = term.clone();

        let result = match term.as_ref() {
            Sort(_, lvl)           => mk_sort(mk_succ(lvl.clone())),
            Const(_, name, lvls)   => self.infer_const(name, lvls),
            Local(.., bind)        => (bind.ty).clone(),
            App(..)                => self.infer_apps(term),
            Lambda(..)             => self.infer_lambda(term),
            Pi(..)                 => mk_sort(self.infer_pi(term)),
            Let(_, dom, val, body) => self.infer_let(dom, val, body),
            owise                  => err_infer_var(line!(), owise),
        };

        self.infer_cache.insert(cache_key, result.clone());

        result
    }



    pub fn infer_const(&mut self, name : &Name, levels : &Arc<Vec<Level>>) -> Expr {
        match self.env.read().declarations.get(name) {
            Some(dec) => {
                let univ_params = dec.univ_params.as_ref();
                assert!(univ_params.len() == levels.len());
                let subst_map = univ_params.clone().into_iter().zip(levels.as_ref().clone()).collect::<Vec<(Level, Level)>>();
                dec.ty.instantiate_ps(&subst_map)
            },
            None => err_infer_const(line!(), name)
        }
    }

    pub fn infer_lambda(&mut self, mut term : &Expr) -> Expr {
        let mut domains = Vec::with_capacity(50);
        let mut locals  = Vec::with_capacity(50);

        while let Lambda(_, ref old_dom, ref old_body) = term.as_ref() {
            domains.push(old_dom.clone());
            let new_dom_ty = old_dom.ty.instantiate(locals.iter().rev());
            let new_dom = old_dom.clone().swap_ty(new_dom_ty.clone());

            if self.should_check() {
                self.infer_universe_of_type(&new_dom_ty);
            }

            let new_local = new_dom.as_local();
            locals.push(new_local);
            term = old_body;
        }

        let instd = term.instantiate(locals.iter().rev());
        let inferred = self.infer(&instd);
        let mut abstrd = inferred.abstract_(locals.iter().rev());

        while let Some(d) = domains.pop() {
            abstrd = mk_pi(d, abstrd);
        }

        abstrd
    }



  
    pub fn infer_universe_of_type(&mut self, term : &Expr) -> Level {
        let inferred = self.infer(term);
        match self.whnf(&inferred).as_ref() {
            Sort(_, lvl) => lvl.clone(),
            owise => err_infer_universe(line!(), owise),
        }
    }


    fn infer_apps(&mut self, term : &Expr) -> Expr {
        let (fn_, mut apps) = term.unfold_apps_refs();

        let mut acc = self.infer(fn_);
        let mut context = Vec::<&Expr>::with_capacity(apps.len());

        while let Some(elem) = apps.pop() {
            if let Pi(_, ref old_dom, ref old_body) = acc.as_ref() {
                if self.should_check() {
                    let new_dom_ty = old_dom.ty
                                     .instantiate(context.iter().map(|x| *x).rev());
                    self.check_type(elem, &new_dom_ty);
                }
                context.push(elem);
                acc = (old_body).clone();
            } else {
                let instd = acc.instantiate(context.iter().map(|x| *x).rev());
                let whnfd = self.whnf(&instd);
                match whnfd.as_ref() {
                    Pi(..) => {
                        apps.push(elem);
                        context = Vec::new();
                        acc = whnfd;
                    },
                    owise => err_infer_apps(line!(), owise),
                }
            }
        }

        acc.instantiate(context.iter().map(|x| *x).rev())
   }


    pub fn infer_pi(&mut self, mut term : &Expr) -> Level {
        let mut locals = Vec::new();
        let mut universes = Vec::new();

        while let Pi(_, ref old_dom, ref old_body) = term.as_ref() {
            let new_dom_ty = old_dom.ty.instantiate(locals.iter().rev());
            let new_dom = old_dom.clone().swap_ty(new_dom_ty.clone());
            let dom_univ = self.infer_universe_of_type(&new_dom_ty);
            universes.push(dom_univ);
            let new_local = new_dom.as_local();
            locals.push(new_local);
            term = old_body;
        }

        let instd = term.clone().instantiate(locals.iter().rev());
        let mut inferred = self.infer_universe_of_type(&instd);
        //let inferred = self.infer_universe_of_type(&instd);
        //foldr(|acc, next| mk_imax(acc, next), universes, inferred)

        while let Some(u) = universes.pop() {
            inferred = mk_imax(u, inferred);
        };

        inferred
    }
    
    pub fn infer_let(&mut self, dom : &Binding, val : &Expr, body : &Expr) -> Expr {
        if self.should_check() {
            self.infer_universe_of_type(&dom.ty);
        }
        if self.should_check() {
            self.check_type(val, &dom.ty);
        }

        let instd_body = body.instantiate(Some(val).into_iter());
        self.infer(&instd_body)
    }


    pub fn check_type(&mut self, e : &Expr, ty : &Expr) {
        let inferred = self.infer(e);
        match self.check_def_eq(ty, &inferred) {
            EqShort => (),
            _ => err_check_type(line!(), e, ty),
        }
    }

    pub fn require_def_eq(&mut self, e1 : &Expr, e2 : &Expr) {
        match self.check_def_eq(e1, e2) {
            EqShort => (),
            _ => err_req_def_eq(line!(), e1, e2)
        }
    }

}



/// Exercises some control over the degree of reduction. In particular, 
/// affects whether `reduce_hdtl()` proceeds in attempting to reduce 
/// a constant term.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Flag {
    FlagT,
    FlagF
}


