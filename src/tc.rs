use std::cmp::Ordering::*;
use hashbrown::HashMap;
use once_cell::sync::Lazy;

use Cheap::*;
use crate::utils::{ Either, 
                    FailureCache, 
                    Either::*, 
                    DeltaResult, 
                    DeltaResult::*, 
                    ShortCircuit, 
                    ShortCircuit::*, 
                    SSOption, 
                    EqCache };
use crate::name::Name;
use crate::level::{ Level, 
                    mk_imax, 
                    mk_succ, 
                    is_def_eq_lvls };
use crate::env::{ ArcEnv, ConstantInfo };
use crate::errors::*;
use crate::recursor::RecursorVal;
use crate::expr::{ Expr, 
                   mk_var,
                   mk_sort,
                   mk_const, 
                   mk_app,
                   mk_pi,
                   mk_lambda,
                   mk_prop,
                   Binding, InnerExpr::*, };

pub static QLIFT    : Lazy<Name> = Lazy::new(|| Name::from("quot").extend_str("lift"));
pub static QMK      : Lazy<Name> = Lazy::new(|| Name::from("quot").extend_str("mk"));
pub static QIND     : Lazy<Name> = Lazy::new(|| Name::from("quot").extend_str("ind"));
pub static ID_DELTA : Lazy<Name> = Lazy::new(|| Name::from("id_delta"));

#[derive(Clone)]
pub struct TypeChecker {
    pub m_safe_only : bool,
    pub infer_cache : HashMap<Expr, Expr>,
    pub eq_cache : EqCache,
    pub whnf_cache : HashMap<Expr, Expr>,
    pub whnf_core_cache : HashMap<Expr, Expr>,
    pub env : ArcEnv,
    pub m_lparams : Option<Vec<Level>>,
    pub lc_cache : LcCache,
    pub failure_cache : FailureCache,
}

impl std::fmt::Debug for TypeChecker {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "<typechecker>")
    }
}

impl std::cmp::PartialEq for TypeChecker {
    fn eq(&self, _ : &TypeChecker) -> bool {
        true
    }
}

impl std::cmp::Eq for TypeChecker {}

impl TypeChecker {
    pub fn new(safe_only : Option<bool>, env : ArcEnv) -> Self {
        TypeChecker {
            m_safe_only : safe_only.unwrap_or(false),
            infer_cache : HashMap::with_capacity(1000),
            eq_cache : EqCache::with_capacity(1000),
            whnf_cache : HashMap::with_capacity(1000),
            whnf_core_cache : HashMap::with_capacity(100),
            env,
            m_lparams : None,
            lc_cache : LcCache::new(),
            failure_cache : FailureCache::with_capacity(500),
        }
    }

    // FVars are not yet implemented.
    pub fn whnf_fvar(&mut self, _e : &Expr) -> Expr {
        unimplemented!()
    }


    pub fn infer_only(&mut self, _e : &Expr) -> Expr {
        self.infer_type(_e)
    }

    pub fn infer_type(&mut self, _e : &Expr) -> Expr {
        self.infer_type_core(_e, true)
    }


    pub fn infer_universe_of_type(&mut self, term : &Expr) -> Level {
        let inferred = self.infer_type_core(term, false);

        match self.whnf(&inferred).as_ref() {
            Sort { level : lvl, .. } => {
                lvl.clone()
            }
            owise => err_infer_universe(line!(), owise),
        }
    }


    pub fn check_type___(&mut self, e : &Expr, ty : &Expr) {
        let inferred = self.infer_type_core(e, false);
        match self.is_def_eq(ty, &inferred) {
            EqShort => (),
            _ => {
                err_check_type(line!(), e, ty)
            }
        }
    }

    fn infer_apps(&mut self, term : &Expr, infer_only : bool) -> Expr {

        let (fn_, mut apps) = term.unfold_apps();
        let mut acc = self.infer_type_core(fn_, infer_only);

        let mut context = Vec::<&Expr>::with_capacity(apps.len());

        while let Some(elem) = apps.pop() {
            if let Pi { binder : ref old_dom, body : ref old_body, .. } = acc.as_ref() {
                if !infer_only {
                    let new_dom_ty = old_dom.ty
                                     .instantiate(context.iter().copied().rev());
                    self.check_type___(elem, &new_dom_ty);
                }
                context.push(elem);
                acc = (old_body).clone();
            } else {
                let instd = acc.instantiate(context.iter().copied().rev());
                let whnfd = self.whnf(&instd);

                match whnfd.as_ref() {
                    Pi {..} => {
                        apps.push(elem);
                        context = Vec::new();
                        acc = whnfd;
                    },
                    owise => err_infer_apps(line!(), owise),
                }
            }
        }

        acc.instantiate(context.iter().copied().rev())
   }


    pub fn infer_pi(&mut self, mut term : &Expr) -> Level {
        let mut locals = Vec::new();
        let mut universes = Vec::new();

        while let Pi { binder : ref old_dom, body : ref old_body, .. } = term.as_ref() {
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


        while let Some(u) = universes.pop() {
            inferred = mk_imax(u, inferred);
        };

        inferred
    }

    pub fn infer_lambda(&mut self, mut term : &Expr, infer_only : bool) -> Expr {
        let mut domains = Vec::with_capacity(50);
        let mut locals  = Vec::with_capacity(50);

        while let Lambda { binder : ref old_dom, body : ref old_body, .. } = term.as_ref() {
            domains.push(old_dom.clone());
            let new_dom_ty = old_dom.ty.instantiate(locals.iter().rev());
            let new_dom = old_dom.clone().swap_ty(new_dom_ty.clone());

            if !infer_only {
                self.infer_universe_of_type(&new_dom_ty);
            }

            let new_local = new_dom.as_local();
            locals.push(new_local);
            term = old_body;
        }

        let instd = term.instantiate(locals.iter().rev());
        let inferred = self.infer_type_core(&instd, infer_only);
        let mut abstrd = inferred.abstract_(locals.iter().rev());
        while let Some(d) = domains.pop() {
            abstrd = mk_pi(d, abstrd);
        }
        abstrd
    }

    pub fn infer_let(&mut self, dom : &Binding, val : &Expr, body : &Expr, infer_only : bool) -> Expr {
        if !infer_only {
            self.infer_universe_of_type(&dom.ty);
        }
        if !infer_only {
            let infd = self.infer_type_core(val, infer_only);
            assert!(self.is_def_eq(&infd, &dom.ty) == EqShort)
        }

        let instd_body = body.instantiate(Some(val).into_iter());
        self.infer_type_core(&instd_body, infer_only)
    }

    pub fn infer_type_core(&mut self, _e : &Expr, infer_only : bool) -> Expr {
        self.infer_cache.get(_e).cloned().unwrap_or_else(|| {
            let result = match _e.as_ref() {
                Sort { level : lvl, .. } => {
                    if infer_only {
                        check_level(self.m_lparams.as_ref(), lvl)
                    } 
                    mk_sort(mk_succ(lvl.clone()))
                },
                Local { binder : bind, .. }         => (bind.ty).clone(),
                Const { name, levels : lvls, .. }   => self.infer_const(name, lvls, infer_only),
                App    {..}                => self.infer_apps(_e, infer_only),
                Lambda {..}                => self.infer_lambda(_e, infer_only),
                Pi     {..}                => mk_sort(self.infer_pi(_e)),
                Let { binder : dom, val, body, .. } => self.infer_let(dom, val, body, infer_only),
                owise                  => err_infer_var(line!(), owise),
            };

            self.infer_cache.insert(_e.clone(), result.clone());
            result
        })
    }

    pub fn infer_const(&mut self, n : &Name, ls : &Vec<Level>, infer_only : bool) -> Expr {
        if let Some(const_info) = self.env.read().get_constant_info(n) {
            assert_eq!(ls.len(), const_info.get_constant_val().lparams.len());
            if !infer_only {
                if ((self.m_safe_only) && const_info.is_unsafe()) {
                    panic!("Cannot check an unsafe definition in this manner")
                }

                ls.iter().for_each(|l| check_level(self.m_lparams.as_ref(), l))
            }

            const_info.get_constant_val()
            .type_
            .instantiate_lparams(const_info.get_constant_val().lparams.iter().zip(ls))
        } else {
            err_infer_const(line!(), n)
        }
    }

    // Revisit; 1. this corresponds to infer_universe_of_type in the old one;
    // seems to omit a final step of asserting that whnfd' result is a Sort.
    // 2. This has two versions; one that ignores undefined univ params,
    // the other sets the self.m_lparams thing to be the names list it gets.
    // I'm not sure how you're supposed to drop them afterward.
    pub fn check(&mut self, _e : &Expr, lparams : Vec<Level>) -> Expr {
        self.m_lparams = Some(lparams);
        self.infer_type_core(_e, false)
    }

    pub fn ensure_type(&mut self, _e : &Expr) -> Expr {
        let infd = self.infer_only(_e);
        self.ensure_sort(&infd)
    }

    pub fn ensure_pi(&mut self, _e : &Expr) -> Expr {
        self.ensure_pi_core(_e)
    }

    pub fn ensure_sort(&mut self, _e : &Expr) -> Expr {
        self.ensure_sort_core(_e)
    }

    pub fn ensure_sort_core(&mut self, _e : &Expr) -> Expr {
        if let Sort {..} = _e.as_ref() {
            return _e.clone()
        }
        let new_e = self.whnf(_e);
        if let Sort {..} = new_e.as_ref() {
            return new_e
        } else {
            panic!("ensure_sort expected to whnf() to a Sort, but didn't. Got \n{:#?}\n", new_e)
        }
    }

    pub fn ensure_pi_core(&mut self, _e : &Expr) -> Expr {
        if let Pi {..} = _e.as_ref() {
            return _e.clone()
        }

        let new_e = self.whnf(_e);
        if let Pi {..} = new_e.as_ref() {
            return new_e
        } else {
            panic!("ensure_pis expected to whnf() to a pi, but didn't. Got \n{:#?}\n", new_e)
        }
    }

    pub fn is_def_eq(&mut self, _t : &Expr, _s : &Expr) -> ShortCircuit {
        self.eq_cache.get(_t, _s).unwrap_or_else(|| {
            let res = if _t == _s {
                EqShort
            } else {
                self.is_def_eq_core(_t, _s)
            };
            self.eq_cache.insert(_t.clone(), _s.clone(), res);
            res
        })
    }

    pub fn is_def_eq_core(&mut self, t : &Expr, s : &Expr) -> ShortCircuit {
        if let Some(short) = self.quick_is_def_eq(t, s) {
            return short
        }

        let t_n = self.whnf_core(t, None);
        let s_n = self.whnf_core(s, None);
    
        // only re-check `quick_is_def_eq` if whnf_core actually reduced
        if ((!t_n.check_ptr_eq(t)) || (!s_n.check_ptr_eq(s))) {
            if let Some(short) = self.quick_is_def_eq(&t_n, &s_n) {
                return short
            }
        }

        if self.is_def_eq_proof_irrel(&t_n, &s_n) {
            return EqShort
        }
        
        let (t_reduced, s_reduced) = match self.lazy_delta_reduction(&t_n, &s_n) {
            Left(Some(short)) => return short,
            Left(None) => unreachable!(),
            Right((e1, e2)) => (e1, e2),
        };

        if let (Const { name : n1, levels : lvls1, .. }, Const { name : n2, levels : lvls2, .. }) = (t_reduced.as_ref(), s_reduced.as_ref()) {
            if (n1 == n2) && (is_def_eq_lvls(lvls1, lvls2)) {
                return EqShort
            }
        }
    
        // if two Locals have the same serial, they must
        // be clones, and are therefore definitionally equal.
        if let (Local { serial : serial1, .. }, Local { serial : serial2, .. }) = (t_reduced.as_ref(), s_reduced.as_ref()) {
            if serial1 == serial2 {
                return EqShort
            }
        }

        // Projections are not yet implemented
        //if let (Proj(.., pidx1, proj_expr1), Proj(.., pidx2, proj_expr2)) = (t_reduced.as_ref(), s_reduced.as_ref()) {
        //    if proj_expr1 == proj_expr2 {
        //        return true
        //    }
        //}

        if self.is_def_eq_app(&t_reduced, &s_reduced) {
            return EqShort
        }
    
        if self.try_eta_expansion(&t_reduced, &s_reduced) {
            return EqShort
        }
    
    
        NeqShort
    }

    /// e is a prop iff it destructures as Sort(Level(Zero))
    pub fn is_prop(&mut self, e : &Expr) -> bool {
        match self.whnf(e).as_ref() {
            Sort { level, .. } => level.is_zero(),
            _ => false
        }
    }

    /// tries is_prop after inferring e
    pub fn is_proposition(&mut self, e : &Expr) -> bool {
        let inferred = self.infer_only(e);
        self.is_prop(&inferred)
    }

    pub fn is_proof(&mut self, p: &Expr) -> bool {
        let inferred = self.infer_only(p);
        self.is_proposition(&inferred)
    }

    fn is_def_eq_proof_irrel(&mut self, e1: &Expr, e2: &Expr) -> bool {
        ((self.is_proof(e1)) && (self.is_proof(e2)))
    }


    pub fn is_delta(&self, _e : &Expr) -> Option<ConstantInfo> {
        _e.unfold_apps_fn()
          .get_const_name()
          .and_then(|name| self.env.read().get_constant_info(name).cloned())
          .and_then(|const_info| 
            match const_info.has_value(None) {
              true => Some(const_info),
              false => None
          })
   }


    pub fn unfold_definition_core(&self, _e : &Expr) -> Option<Expr> {
        if let (Const { levels, .. }, Some(ref const_info)) = (_e.as_ref(), self.is_delta(_e)) {
            if (levels.len() == const_info.get_constant_val().lparams.len()) {
                return Some(instantiate_value_lparams(const_info, levels))
            }
        }
        None
    }

    pub fn unfold_definition_infallible(&self, _e : &Expr) -> Expr {
        match self.unfold_definition(_e) {
            Some(r) => r,
            None => crate::errors::unfold_definition_infallible_failed(line!(), _e)
        }
    }
    
    pub fn unfold_definition(&self, _e : &Expr) -> Option<Expr> {
        if let App {..} = _e.as_ref() {
            let f0 = _e.unfold_apps_fn();
            self.unfold_definition_core(&f0)
            .map(|unfolded| {
                let (_, args) = _e.unfold_apps();
                unfolded.foldl_apps(args.iter().rev().copied())
            })
        } else {
            self.unfold_definition_core(_e)
        }
    }

    // FIXME figure out how to do this without allocating
    // vectors
    // The reversal here is super important; if you don't reverse this,
    // you'll hit a wall trying to check things like pi314
    pub fn eq_args(&mut self, _t : &Expr, _s : &Expr) -> bool {
        let (_, t_args) = _t.unfold_apps();
        let (_, s_args) = _s.unfold_apps();

        if t_args.len() != s_args.len() {
            return false
        }

        for (a, b) in t_args.into_iter().zip(s_args.into_iter()).rev() {
            if self.is_def_eq(a, b) == NeqShort {
                return false
            }
        }
        true

    }

    pub fn eq_args_(&mut self, mut t : &Expr, mut s : &Expr) -> bool {
        while let (App { fun : f1, arg : r1, .. }, App { fun : f2, arg : r2, .. }) = (t.as_ref(), s.as_ref()) {
            match self.is_def_eq(r1, r2) {
                EqShort => {
                    t = f1;
                    s = f2;
                },
                _ => return false
            }
        }
        true
    }

    pub fn lazy_delta_reduction(&mut self, t : &Expr, s : &Expr) -> Either<SSOption, (Expr, Expr)> {
        let mut t_cursor = t.clone();
        let mut s_cursor = s.clone();
        loop {
            match self.lazy_delta_reduction_step(&t_cursor, &s_cursor) {
                Continue(t_, s_) => {
                    t_cursor = t_;
                    s_cursor = s_;
                },
                StopEq  => return Left(Some(EqShort)),
                StopNeq => return Left(Some(NeqShort)),
                Unknown => return Right((t_cursor, s_cursor))
            }
        }
    }

    pub fn lazy_delta_reduction_step(&mut self, t_n0 : &Expr, s_n0 : &Expr) -> DeltaResult {
        let delta_t = self.is_delta(t_n0);
        let delta_s = self.is_delta(s_n0);

        let (reduced_t, reduced_s) = match (delta_t, delta_s) {
            // v returns early
            (None, None) => return Unknown,
            // v returns early
            (Some(dt_info), _) if &dt_info.get_constant_val().name == (&*ID_DELTA) => {
                let unfolded = self.unfold_definition_infallible(t_n0);
                let whnfd = self.whnf_core(&unfolded, None);

                if &whnfd == s_n0 {
                    return StopEq
                }

                if let Some(u) = self.unfold_definition(&whnfd) {
                    return Continue(self.whnf_core(&u, None), s_n0.clone())
                } else {
                    return Continue(whnfd, s_n0.clone())
                }
            },
            // v returns early
            (_, Some(ds_info)) if &ds_info.get_constant_val().name == (&*ID_DELTA) => {
                let unfolded = self.unfold_definition_infallible(s_n0);
                let whnfd = self.whnf_core(&unfolded, None);

                if (t_n0 == &whnfd) {
                    return StopEq
                }

                if let Some(u) = self.unfold_definition(&whnfd) {
                    return Continue(t_n0.clone(), self.whnf_core(&u, None))
                } else {
                    return Continue(t_n0.clone(), whnfd)
                }
            },
            (Some(_), None) => {
                let unfolded = self.unfold_definition_infallible(t_n0);
                (self.whnf_core(&unfolded, None), s_n0.clone())
            },
            (None, Some(_)) => {
                let unfolded = self.unfold_definition_infallible(s_n0);
                (t_n0.clone(), self.whnf_core(&unfolded, None))
            },
            (Some(dt_info), Some(ds_info)) => {
                match dt_info.get_hint().compare(ds_info.get_hint()) {
                    Greater => {
                        let unfolded = self.unfold_definition_infallible(t_n0);
                        (self.whnf_core(&unfolded, None), s_n0.clone())
                    },
                    Less => {
                        let unfolded = self.unfold_definition_infallible(s_n0);
                        (t_n0.clone(), self.whnf_core(&unfolded, None))
                    }
                    Equal => {
                        if ((t_n0.is_app()) && (s_n0.is_app()) && (dt_info == ds_info)) {
                            // FIXME ideally would return errors instead of using
                            // partial get_const_levels
                            if ((self.eq_args(t_n0, s_n0)) && (is_def_eq_lvls(t_n0.unfold_apps_fn().get_const_levels_inf(), &s_n0.unfold_apps_fn().get_const_levels_inf()))) {
                                return StopEq
                            } else {
                                self.failure_cache.insert(t_n0.clone(), s_n0.clone())
                            }
                        }

                        let unfolded_t = self.unfold_definition_infallible(t_n0);
                        let unfolded_s = self.unfold_definition_infallible(s_n0);
                        (self.whnf_core(&unfolded_t, None), self.whnf_core(&unfolded_s, None))
                    }
                }
            }
        };

        match self.quick_is_def_eq(&reduced_t, &reduced_s) {
            Some(EqShort) => StopEq,
            Some(NeqShort) => StopNeq,
            None => Continue(reduced_t, reduced_s)
        }
    }

    pub fn is_def_eq_app(&mut self, _t : &Expr, _s : &Expr) -> bool {
        if !(_t.is_app() && _s.is_app()) {
            return false
        }
        let (t_fn, t_args) = _t.unfold_apps_rev();
        let (s_fn, s_args) = _s.unfold_apps_rev();

        if ((t_args.len() == s_args.len()) && (self.is_def_eq(t_fn, s_fn) == EqShort)) {
            t_args.into_iter()
            .zip(s_args.into_iter())
            .all(|(l, r)| self.is_def_eq(l, r) == EqShort)
        } else {
            false
        }
    }

    pub fn try_eta_expansion(&mut self, t : &Expr, s : &Expr) -> bool {
        self.try_eta_expansion_core(t, s) || self.try_eta_expansion_core(s, t)
    }

    pub fn try_eta_expansion_core(&mut self, t : &Expr, s : &Expr) -> bool {
        if ((t.is_lambda()) && (!s.is_lambda())) {
            let s_infd = self.infer_type(s);
            let s_type = self.whnf(&s_infd);
            if let Pi { binder : bind, .. } = s_type.as_ref() {
                let s_1 = mk_app(s.clone(), mk_var(0));
                let binding = Binding::mk(bind.pp_name.clone(), bind.ty.clone(), bind.style);
                let new_s = mk_lambda(binding, s_1);
                if (self.is_def_eq(t, &new_s) == NeqShort) {
                    false
                } else {
                    true
                }
            } else {
                false
            }
        } else {
            if !t.is_lambda() {
                false
            } else {
                assert!(s.is_lambda());
                false
            }
        }
    }

    pub fn self_check_with_lc(&mut self, binding : &Binding, body1 : &Expr, body2 : &Expr) -> ShortCircuit {
        let lc = self.lc_cache.get_lc(binding);
        let inst1 = body1.instantiate(Some(&lc).into_iter());
        let inst2 = body2.instantiate(Some(&lc).into_iter());
        let result = self.is_def_eq(&inst1, &inst2);
        self.lc_cache.replace_lc(binding.clone(), lc);
        result
    }

    pub fn quick_is_def_eq(&mut self, t : &Expr, s : &Expr) -> SSOption {
        let result = if let Some(cached) = self.eq_cache.get(t, s) {
            Some(cached)
        } else {
            match (t.as_ref(), s.as_ref()) {
                (Sort { level : lvl1, .. }, Sort { level : lvl2, .. }) => {
                    match lvl1.eq_by_antisymm(lvl2) {
                        true => Some(EqShort),
                        false => Some(NeqShort)
                    }
                },
                // Experimental.
                (Lambda { binder : bind1, body : body1, .. }, Lambda { binder : bind2, body : body2, .. })  => {
                    if (self.is_def_eq(&bind1.ty, &bind2.ty) == NeqShort) {
                        Some(NeqShort)
                    } else {
                        Some(self.self_check_with_lc(bind1, body1, body2))
                    }
                }
                (Pi { binder : bind1, body : body1, .. }, Pi { binder : bind2, body : body2, .. }) => {
                    if (self.is_def_eq(&bind1.ty, &bind2.ty) == NeqShort) {
                        Some(NeqShort)
                    } else {
                        Some(self.self_check_with_lc(bind1, body1, body2))
                    }
                }
                //(Lambda {..}, Lambda {..}) => Some(self.check_def_eq_lambdas(t, s)),
                //(Pi {..}, Pi {..}) => Some(self.check_def_eq_pis(t, s)),
                _ => None
            }
        };
        result


    }

    // Literally the same function as its Lambda counterpart, but checks for a different
    // enum discriminant (Pis instead of Lambdas).
    pub fn check_def_eq_pis(&mut self, mut e1 : &Expr, mut e2 : &Expr) -> ShortCircuit {
        let mut substs = Vec::new();
        while let (Pi { binder : dom1, body : body1, .. }, Pi { binder : dom2, body : body2, .. }) = (e1.as_ref(), e2.as_ref()) {
            let mut lhs_type = None;

            if dom1 != dom2 {
                let instd_d2_ty = dom2.ty.instantiate(substs.iter().rev());
                let instd_d1_ty = dom1.ty.instantiate(substs.iter().rev());
                lhs_type = Some(dom2.clone().swap_ty(instd_d2_ty.clone()));
                if (self.is_def_eq(&instd_d1_ty, &instd_d2_ty) == NeqShort) {
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

        self.is_def_eq(&e1.instantiate(substs.iter().rev()), 
                       &e2.instantiate(substs.iter().rev()))
    }


    pub fn check_def_eq_lambdas(&mut self, mut e1 : &Expr, mut e2 : &Expr) -> ShortCircuit {
        let mut substs = Vec::new();
        while let (Lambda { binder : dom1, body : body1, .. }, Lambda { binder : dom2, body : body2, .. }) = (e1.as_ref(), e2.as_ref()) {
            let mut lhs_type = None;

            if dom1 != dom2 {
                let instd_d2_ty = dom2.ty.instantiate(substs.iter().rev());
                let instd_d1_ty = dom1.ty.instantiate(substs.iter().rev());
                lhs_type = Some(dom2.clone().swap_ty(instd_d2_ty.clone()));
                if (self.is_def_eq(&instd_d1_ty, &instd_d2_ty) == NeqShort) {
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

         self.is_def_eq(&e1.instantiate(substs.iter().rev()), 
                        &e2.instantiate(substs.iter().rev()))
    }

    pub fn reduce_quot_rec(&mut self, _e : &Expr) -> Option<Expr> {
        if !self.env.read().quot_is_init {
            return None
        };

        let (fun, args) = _e.unfold_apps_rev();
        if let Const { name, .. } = fun.as_ref() {
            let (mk_pos, arg_pos) = match name {
                n if n == &*QLIFT => (5, 3),
                n if n == &*QIND => (4, 3),
                _ => return None
            };

            if args.len() <= mk_pos {
                return None
            }

            let mk = self.whnf(args.get(mk_pos)?);
            let mk_fn = mk.unfold_apps_fn();

            if !(mk_fn.get_const_name()? == &*QMK) {
                return None
            }

            let f = args.get(arg_pos)?;

            let r_rhs = match mk.as_ref() {
                App { arg, .. } => arg,
                owise => crate::errors::quot_rec_bad_app(line!(), owise)
            };

            let r = mk_app((*f).clone(), r_rhs.clone());
            let elim_arity = mk_pos + 1;

            if args.len() > elim_arity {
                let num_items = args.len() - elim_arity;
                Some(r.foldl_apps(args.into_iter().skip(elim_arity).take(num_items)))
            } else {
                Some(r)
            }
        } else {
            None
        }
    }

    pub fn inductive_reduce_rec(&mut self, _e : &Expr, cheap : Cheap) -> Option<Expr> {
        let (fun, args) = _e.unfold_apps_rev();
        let (name, levels) = fun.try_const_fields()?;

        let whnf_closure = |tc : &mut TypeChecker, x : &Expr| {
            match cheap {
                CheapTrue => tc.whnf_core(x, Some(cheap)),
                _ => tc.whnf(x)
            }
        };

        // Have to clone since it's behind the RwLock and we need to do 
        // other stuff that requires mutable access to Tc/Env
        let recursor_val = match self.env.read().constant_infos.get(name)? {
            ConstantInfo::RecursorInfo(r) => r.clone(),
            _ => return None
        };

        let major_idx = recursor_val.get_major_idx();
        if major_idx >= args.len() {
            return None
        }

        let mut major = (*&args[major_idx]).clone();
        if recursor_val.is_k {
            if let Some(k_cnstr) = self.to_cnstr_when_K(&recursor_val, &major) {
                major = k_cnstr;
            }
        }

        //major = self.whnf(&major);
        major = whnf_closure(self, &major);

        let rule = recursor_val.get_rec_rule_for(&major)?;

        let (_, major_args) = major.unfold_apps_rev();
        if rule.nfields > major_args.len() {
            return None
        }

        if (levels.len() != (recursor_val.constant_val.lparams.len())) {
            return None
        }

        let rhs_zip = recursor_val.constant_val.lparams.iter().zip(levels.iter());
        let rhs = rule.rhs.instantiate_lparams(rhs_zip);
        let rhs = rhs.foldl_apps(args.iter().take(recursor_val.nparams 
                                                + recursor_val.nmotives 
                                                + recursor_val.nminors).copied());
        let nparams = major_args.len() - rule.get_nfields();

        let rhs = rhs.foldl_apps(major_args.iter().skip(nparams).take(rule.get_nfields()).copied());

        if (args.len() > major_idx + 1) {
            let nextra = args.len() - major_idx - 1;
            let rhs = rhs.foldl_apps(args.iter().skip(major_idx + 1).take(nextra).copied());
            Some(rhs)
        } else {
            Some(rhs)
        }
    }

    pub fn mk_nullary_cnstr(&self, _e : &Expr, num_params : usize) -> Option<Expr> {
        let (fun, args) = _e.unfold_apps_rev();
        if let Const { name, levels, .. } = fun.as_ref() {
            let constructor_name = self.env.read().get_first_constructor_name(name).cloned()?;
            Some(mk_const(constructor_name, levels.clone()).foldl_apps(args.iter().take(num_params).copied()))
        } else {
            None
        }
    }


    fn to_cnstr_when_K(&mut self, rval : &RecursorVal, _e : &Expr) -> Option<Expr> {
        let infd = self.infer_type(_e);
        let app_type = self.whnf(&infd);
        if (app_type.unfold_apps_fn().get_const_name()? != rval.get_induct()) {
            return None
        }

        let new_cnstr_app = self.mk_nullary_cnstr(&app_type, rval.nparams)?;
        let new_type = self.infer_type(&new_cnstr_app);
        if (self.is_def_eq(&app_type, &new_type) == NeqShort) {
            return None
        }
        Some(new_cnstr_app)
    }

    pub fn whnf(&mut self, _e : &Expr) -> Expr {
        self.whnf_cache.get(_e).cloned().unwrap_or_else(|| {
            let mut cursor = _e.clone();
            loop {
                let whnfd = self.whnf_core(&cursor, None);
                if let Some(next_term) = self.unfold_definition(&whnfd) {
                    cursor = next_term;
                } else {
                    self.whnf_cache.insert(_e.clone(), whnfd.clone());
                    return whnfd
                }
            }
        })
    }

    pub fn whnf_core(&mut self, _e : &Expr, _cheap : Option<Cheap>) -> Expr {
        let cheap = _cheap.unwrap_or(CheapFalse);

        let (_fn, args) = _e.unfold_apps();
        match _fn.as_ref() {
            Sort { level, .. } => mk_sort(level.simplify()),
            Lambda {..} if !args.is_empty() => {
                let whnfd = self.whnf_lambda(_fn, args);
                self.whnf_core(&whnfd, Some(cheap))
            },
            Let { val, body, .. } => {
                let applied = body.instantiate(Some(val).into_iter())
                              .foldl_apps(args.into_iter().rev());
                self.whnf_core(&applied, Some(cheap))
            },
            _ => self.reduce_quot_rec(_e)
                 .or(self.inductive_reduce_rec(_e, cheap))
                 .map(|reduced| self.whnf_core(&reduced, Some(cheap)))
                 .unwrap_or_else(|| _e.clone())
        }
    }

    pub fn whnf_lambda(&mut self, 
                   mut fun : &Expr, 
                   mut args : Vec<&Expr>) -> Expr {
        let mut ctx = Vec::with_capacity(args.len());

        while let Lambda { body, .. } = fun.as_ref() {
            if let Some(hd) = args.pop() {
                ctx.push(hd);
                fun = body;
                continue
            } else {
                break
            }
        }

        fun.instantiate(ctx.into_iter().rev())
           .foldl_apps(args.into_iter().rev())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cheap {
    CheapTrue,
    CheapFalse,
}

// Basically just instantiate_lparams for a ConstantInfo it it's one 
// that has a Value with some optimizations.
pub fn instantiate_value_lparams(const_info : &ConstantInfo, ls : &Vec<Level>) -> Expr {
    if (const_info.get_constant_val().lparams.len() != ls.len()) {
        panic!("Universe mismatch at instantiate_value_lparams")
    } else if (!const_info.has_value(None)) {
        panic!("definition/theorem expected at instantiate_value_level_params; got : {:#?}\n", const_info) 
    // REVISIT
    // I think this one is just an optimization, but I'm not 100% sure 
    // if it does what I think it does or not, so I'm going to skip it
    // for now
    //} else if ((ls.is_empty()) || (!const_info.get_value().has_param())) {
    //    const_info.get_value()
    //}
    // REVISIT also you're supposed to do caching here apparently.
    } else {
        //let zipvec = const_info.get_constant_val().lparams.clone().into_iter().zip(ls.into_iter()).collect::<Vec<(Level, Level)>>();
        let zip = const_info.get_constant_val().lparams.iter().zip(ls.iter());
        let value = const_info.get_value();
        value.instantiate_lparams(zip)
    }
}

pub fn instantiate_type_lparams(const_info : &ConstantInfo, ls : Vec<Level>) -> Expr {
    if ((const_info.get_constant_val().lparams.len()) != (ls.len())) {
        panic!("Universe mismatch at instantiate_type_lparams")
    } 
    // REVISIT similar ambiguity to the above function
    //else if (() || ())
    else {
        let zip = const_info.get_constant_val().lparams.iter().zip(ls.iter());
        let const_val_type = const_info.get_constant_val().type_.clone();
        const_val_type.instantiate_lparams(zip)
    }
}
    
pub fn check_level(m_lparams : Option<&Vec<Level>>, l : &Level) {
    if let Some(set) = m_lparams {
        match l.get_undef_param(set) {
            Some(bad) => {
                panic!("check_level found an undefined param : {:?}\n", bad);
            },
            _ => ()
        }
    }
}

#[derive(Clone)]
pub struct LcCache {
    inner : HashMap<Expr, Vec<Expr>>
}

impl LcCache {
    pub fn new() -> Self {
        LcCache {
            inner : HashMap::new()
        }
    }

    pub fn get_lc(&mut self, b : &Binding) -> Expr {
        match self.inner.get_mut(&b.ty) {
            Some(v) => {
                match v.pop() {
                    Some(l) => {
                        l
                    }
                    None => b.clone().as_local()
                }
            },
            None => {
                b.clone().as_local()
            }
        }
    }

    pub fn replace_lc(&mut self, b : Binding, lc : Expr) {
        match self.inner.get_mut(&b.ty) {
            Some(v) => {
                v.push(lc);
            }
            None => {
                self.inner.insert(b.ty, vec![lc]);
            }
        }
    }

    pub fn replace_lcs(&mut self, mut bs : Vec<Binding>, mut lcs : Vec<Expr>) {
        assert_eq!(bs.len(), lcs.len());
        while !bs.is_empty() {
            let b = bs.pop().unwrap();
            let l = lcs.pop().unwrap();
            self.replace_lc(b, l);
        }

    }
}

