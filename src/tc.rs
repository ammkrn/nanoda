
use std::sync::Arc;
use hashbrown::HashMap;
use parking_lot::RwLock;

use crate::utils::{ ShortCircuit, ShortCircuit::*, EqCache };
use crate::name::Name;
use crate::level::{ Level, mk_imax, mk_succ };
use crate::expr::{ Expr, Binding, InnerExpr::*, mk_app, mk_lambda, mk_var, mk_sort, mk_prop, mk_pi };
use crate::reduction::ReductionCache;
use crate::env::Env;
use crate::errors::*;


/// TypeChecker は型として見れば、カッシュのまとめと現在使用されている環境
/// へのハンドルだけです。TypeChecker は環境から読むだけです、書く必要がありません。
/// unsafe_unchecked はプリティープリンターが使用する値をマークするための物だけです。
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
            eq_cache : EqCache::new(),
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

    /// `E1` と `E2` という２つの項の「height・高さ」は, `E1`が`E2`を成分として使って
    /// 定義されているかどうか（あるいは逆の関係がある）ってことを確かめるための数。
    /// もし、`E1 と E2` をユニファイする必要が合ったら、height が高いやつを最初に
    /// unfold したいんだ。なぜならば、その unfold に露出される項はいずれか height が低い
    /// やつにりますが、逆に height が低いやつから始めていけば、もっともっと
    /// 原始的な定義しか出てきません。
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


    /// expr モジュールにある `unfold_pis` と似たようなものですが、それよりアグレッシブ。
    /// `{ apply whnf(e), then unfold_pis(e) } という循環が束縛子を取れなくなるまで
    /// 繰り返す物です。
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

    /// この関数は inductive で一回だけ使用されます。紹介規則を作る内のものです。
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

    /// ある項を whnf へ縮小するためのルートメソッドです。
    pub fn whnf(&mut self, e : &Expr) -> Expr {
        if let Some(cached) = self.whnf_cache.get(e) {
            return cached.clone()
        } else {
            let cache_key = e.clone();
            let result = self.whnf_core(e, Flag::rho_true());
            self.whnf_cache.insert(cache_key, result.clone());
            result
        }
    }

    pub fn whnf_core(&mut self, e : &Expr, flag : Flag) -> Expr {
        let (_fn, apps) = e.unfold_apps_refs();

        match _fn.as_ref() {
            Sort(_, lvl) => {
                let simpd = lvl.simplify();
                mk_sort(simpd)
            },
            Lambda(..) if !apps.is_empty() => {
                let intermed = self.whnf_lambda(_fn, apps);
                self.whnf_core(&intermed, flag)
            },
            Let(.., val, body) => {
                let instd = body.instantiate(Some(val).into_iter());
                let applied = instd.fold_apps(apps.into_iter().rev());
                self.whnf_core(&applied, flag)
            },
            _ => {
                let reduced = self.reduce_hdtl(_fn, apps.as_slice(), flag);
                match reduced {
                    Some(eprime) => self.whnf_core(&eprime, flag),
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

    /// ２つの `Expr` を一ステップで縮小する
    pub fn reduce_exps(&mut self, e1 : Expr, e2 : Expr, flag : Flag) -> Option<(Expr, Expr)> {
        let (fn1, apps1) = e1.unfold_apps_refs();
        let (fn2, apps2) = e2.unfold_apps_refs();

        // 遅延評価を使いたいんです。
        let red1 = |tc : &mut TypeChecker| tc.reduce_hdtl(fn1, apps1.as_slice(), flag).map(|r| (r, e2.clone()));
        let red2 = |tc : &mut TypeChecker| tc.reduce_hdtl(fn2, apps2.as_slice(), flag).map(|r| (r, e1.clone()));

        if self.def_height(fn1) > self.def_height(fn2) {
            red1(self).or(red2(self))
        } else {
            red2(self).or(red1(self))
        }
    }


    pub fn reduce_hdtl(&mut self, _fn : &Expr, apps : &[&Expr], flag : Flag) -> Option<Expr> {
        if !flag.rho {
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


    pub fn apps_eq(&mut self, 
                   apps1 : Vec<&Expr>, 
                   apps2 : Vec<&Expr>) -> ShortCircuit {
        if apps1.len() != apps2.len() {
            return NeqShort
        } else {
            for (a, b) in apps1.iter().zip(apps2).rev() {
                if self.check_def_eq(a, b) == EqShort {
                    continue
                } else {
                    return NeqShort
                }
            }
            EqShort
        }
    }


    /// ２つの `Expr` が定義的に等しいかどうかを確かめる手続きに用いられる
    /// 長たらしいケース分析。
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
                    _ => return NeqShort
            },
            (Const(_, n1, lvls1), Const(_, n2, lvls2)) => {
                if n1 == n2 && lvls1.iter().zip(lvls2.as_ref()).all(|(a, b)| Level::eq_by_antisymm(a, b)) {
                    return self.apps_eq(apps1, apps2)
                } else {
                    return NeqShort
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
                return self.check_def_eq_core(fn1, &new_lam)
            },
            (_, Lambda(_, dom, _)) => {
                let app = mk_app(whnfd_1.clone(), mk_var(0));
                let new_lam = mk_lambda(dom.clone(), app);
                return self.check_def_eq_core(&new_lam, fn2)
            },
            (Pi(..), Pi(..)) => self.check_def_eq_pis(fn1, fn2),
            _ => return NeqShort
        }
    }

    pub fn check_def_eq(&mut self, e1 : &Expr, e2 : &Expr) -> ShortCircuit {
        // ポインター等値も構成等値もチェックします。
        if e1 == e2 {
            return EqShort
        } 
        
        // この２つのようそって比較したことがあるかどうかをチェックするステップ
        if let Some(cached) = self.eq_cache.get(&e1, &e2) {
            return cached
        }
        // 比較したことがなければ、結果を計算して、カッシュする後返す。
        let result = if self.is_proof_irrel_eq(e1, e2) {
            EqShort
        } else {
           self.check_def_eq_core(e1, e2)
        };

        self.eq_cache.insert(e1.clone(), e2.clone(), result);
        result
    }


    pub fn check_def_eq_core(&mut self, e1_0 : &Expr, e2_0 : &Expr) -> ShortCircuit {
        let flag = Flag { rho : false };

        let whnfd_1 = self.whnf_core(e1_0, flag);
        let whnfd_2 = self.whnf_core(e2_0, flag);

        match self.check_def_eq_patterns(&whnfd_1, &whnfd_2) {
            EqShort => return EqShort,
            NeqShort => {
                match self.reduce_exps(whnfd_1, whnfd_2, Flag::rho_true()) {
                    Some((red1, red2)) => self.check_def_eq_core(&red1, &red2),
                    _ => return NeqShort
                }
            },
            _ => unreachable!()
        }
    }


    /// Lambda 版と全く同じですが、Lambda の代わりに Pi を処理する。
    pub fn check_def_eq_pis(&mut self, mut e1 : &Expr, mut e2 : &Expr) -> ShortCircuit {

        let mut substs = Vec::new();

        // 分かりにくいけどかなり便利な rust 構文。意味は 「e1 と e2 が
        // 両方 Pi である限り、ブロック内のコードを繰り返して」
        while let (Pi(_, dom1, body1), Pi(_, dom2, body2)) = (e1.as_ref(), e2.as_ref()) {
            let mut lhs_type = None;

            if dom1 != dom2 {
                let instd_d2_ty = dom2.ty.instantiate(substs.iter().rev());
                let instd_d1_ty = dom1.ty.instantiate(substs.iter().rev());

                lhs_type = Some(dom2.clone().swap_ty(instd_d2_ty.clone()));
                // dom が等しくなければ、項全体が等しくあるわけがないから, 残りを計算
                // せずに NeqShortを返す
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


    /// Pi 版と全く同じですが、Lambda の代わりに Pi を処理する。
    pub fn check_def_eq_lambdas(&mut self, mut e1 : &Expr, mut e2 : &Expr) -> ShortCircuit {
        let mut substs = Vec::new();

        // 分かりにくいけどかなり便利な rust 構文。意味は 「e1 と e2 が
        // 両方 Lambda である限り、ブロック内のコードを繰り返して」
        while let (Lambda(_, dom1, body1), Lambda(_, dom2, body2)) = (e1.as_ref(), e2.as_ref()) {
            let mut lhs_type = None;

            if dom1 != dom2 {
                let instd_d2_ty = dom2.ty.instantiate(substs.iter().rev());
                let instd_d1_ty = dom1.ty.instantiate(substs.iter().rev());

                lhs_type = Some(dom2.clone().swap_ty(instd_d2_ty.clone()));
                // dom が等しくなければ、項全体が等しくあるわけがないから, 残りを計算
                // せずに NeqShortを返す
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


    /// 型推論のメイン関数だ。カッシュを検索して早く返して見るものです。
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



/// 縮小ステップの深さを盛業するためのものです。特に、`reduce_hdtl(..)` というメソッド
/// が Const 項を縮小してみるかどうかを制御します。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Flag {
    pub rho: bool,
}

impl Flag {
    pub fn can_reduce_consts(rho: bool) -> Self {
        Flag { rho: rho }
    }

    pub fn rho_true() -> Self {
        Flag { rho: true }
    }
}
