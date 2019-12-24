use std::sync::Arc;
use std::cmp::max;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::Relaxed;
use std::hash::{ Hash, Hasher };

use fxhash::hash64;
use hashbrown::{ HashMap, HashSet };

use crate::name::{ Name, mk_anon };
use crate::level::{ Level, 
                    unique_univ_params, 
                    mk_zero,
                    is_def_eq_lvls };
use crate::utils::{ safe_minus_one, max3 };
use crate::errors;

use InnerExpr::*;

use crate::errors::{ NanodaResult, NanodaErr::* };



/// Because we calculate hashes based on structure, we need
/// something to distinguish between Pi and Lambda expressions, which 
/// apart from their enum discriminant, have the same structure internally.
/// We just use a prime number in a (probably futile) attempt to reduce
/// the likelihood of hash collisions since they'll often be kept in hash maps.
/// Prop itself is treated as a constant, so we also need to know its hash
/// beforehand.
pub const LAMBDA_HASH   : u64 = 402653189;
pub const PI_HASH       : u64 = 1610612741;
pub const PROP_HASH     : u64 = 786433;
pub const PROP_CACHE    : ExprCache = ExprCache { digest : PROP_HASH, 
                                                  var_bound : 0, 
                                                  has_locals : false };

/// Globally visible incrementing counter for fresh Local names. 
/// Lazy man's way of creating fresh names across threads.
/// `Local` items need to have the property that two locals will
/// have the same serial iff `B` was created by executing `clone()`
/// on `A`. 
pub static LOCAL_SERIAL : AtomicU64 = AtomicU64::new(0);


pub fn easy_fresh_name() -> Name {
    let num = LOCAL_SERIAL.fetch_add(1, Relaxed);
    mk_anon().extend_num(num)
}



/// Denote different flavors of binders.
/// Each variant corresponds to the following Lean binder notation : (click for info)
///``` pseudo
///Default         |->   ( .. )
///Implicit        |->   { .. }
///StrictImplicit  |->  {{ .. }}
///InstImplicit    |->   [ .. ]
///```
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BinderStyle {
    Default,
    Implicit,
    StrictImplicit,
    InstImplicit,
}


/// Binding is used to represent the information associated with a Pi, Lambda, or Let
/// expression's binding. pp_name and ty would be like the `x` and `T` respectively
/// in `(λ x : T, E)`. See the doc comments for BinderStyle for information on that.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Binding {
    pub pp_name : Name,
    pub ty : Expr,
    pub style : BinderStyle
}



impl Binding {
    pub fn mk(name : Name, ty : Expr, style : BinderStyle) -> Self {
        Binding {
            pp_name : name,
            ty : ty,
            style
        }
    }

    pub fn is_explicit(&self) -> bool {
        match self.style {
            BinderStyle::Default => true,
            BinderStyle::Implicit => false,
            BinderStyle::StrictImplicit => false,
            BinderStyle::InstImplicit => false
        }
    }


    pub fn as_local(self) -> Expr {
        let serial = LOCAL_SERIAL.fetch_add(1, Relaxed);
        let digest = hash64(&(serial, &self));
        Local { cache : ExprCache::mk(digest, 0, true), 
                binder : self, 
                serial }.into()
    }

    pub fn swap_ty(&self, other : Expr) -> Self {
        Binding::mk(self.pp_name.clone(), other, self.style)
    }

    pub fn swap_name(&self, other : Name) -> Self {
        Binding::mk(other, self.ty.clone(), self.style)
    }

    pub fn swap_name_and_ty(&self, other_n : Name, other_t : Expr) -> Self {
        Binding::mk(other_n, other_t, self.style)
    }
}

/// Arc wrapper around `InnerExpr`. See  InnerExpr's docs.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Expr(Arc<InnerExpr>);

impl std::fmt::Debug for Expr {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.as_ref())
    }
}

/// special constructor for an Expr::Sort that corresponds to `Prop`
pub fn mk_prop() -> Expr {
    Sort { cache : PROP_CACHE, level : mk_zero() }.into() // into Level from InnerLevel
}

/// Makes a variable expression which contains a 
/// [De Brujin index](https://en.wikipedia.org/wiki/De_Bruijn_index).
pub fn mk_var(dbj : usize) -> Expr {
    let digest = hash64(&(dbj));
    Var { cache : ExprCache::mk(digest, dbj as u16 + 1, false), 
          dbj }.into() // InnerLevel -> Level
}

/// Makes a node in the tree, joining two expressions as application.
pub fn mk_app(fun : Expr, arg : Expr) -> Expr {
    let digest = hash64(&(fun.get_digest(), arg.get_digest()));
    let var_bound = fun.var_bound().max(arg.var_bound());
    let has_locals = fun.has_locals() || arg.has_locals();
    App { cache : ExprCache::mk(digest, var_bound, has_locals), 
          fun, 
          arg }.into() // InnerLevel -> Level 
}

/// Represents a Sort/Level/Universe. You can read more about these in 
/// sources like Theorem Proving in Lean.
pub fn mk_sort(level : Level) -> Expr {
    let digest = hash64(&level);
    Sort { cache : ExprCache::mk(digest, 0, false), level }.into() // InnerLevel -> Level 
}

/// A constant; represents a reference to a declaration that has already
/// been added to the environment.
pub fn mk_const(name : impl Into<Name>, levels : impl Into<Vec<Level>>) -> Expr {
    let name = name.into();
    let levels = levels.into();
    let digest = hash64(&(&name, &levels));
    Const { cache : ExprCache::mk(digest, 0, false), 
            name, 
            levels }.into()
}

/// A lambda function.
pub fn mk_lambda(binder : Binding, body: Expr) -> Expr {
    let digest = hash64(&(LAMBDA_HASH, &binder, &body));
    let var_bound = max(binder.ty.var_bound(), 
                        safe_minus_one(body.var_bound()));
    let has_locals = binder.ty.has_locals() || body.has_locals();
    Lambda { cache : ExprCache::mk(digest, var_bound, has_locals), binder, body }.into() // InnerLevel -> Level
}

/// A Pi (dependent function) type.
pub fn mk_pi(binder : Binding, body: Expr) -> Expr {
    let digest = hash64(&(PI_HASH, &binder, &body));
    let var_bound = max(binder.ty.var_bound(),
                        safe_minus_one(body.var_bound()));
    let has_locals = binder.ty.has_locals() || body.has_locals();
    Pi { cache : ExprCache::mk(digest, var_bound, has_locals), 
         binder,
         body  }.into()
}

pub fn mk_local_declar_for(e : &Expr) -> Expr {
    match e.as_ref() {
        Pi { binder, .. } => {
            mk_local(binder.pp_name.clone(), binder.ty.clone(), binder.style)
        },
        owise => panic!("mk_local_declar for wants a pi, got : {:?}\n", owise)
    }

}

pub fn mk_local_declar(n : Name, t : Expr, bi : BinderStyle) -> Expr {
    mk_local(n, t, bi)
}


/// A let binding, IE `let (x : nat) := 5  in 2 * x`
pub fn mk_let(binder : Binding, val : Expr, body : Expr) -> Expr {
    let digest = hash64(&(&binder, val.get_digest(), body.get_digest()));
    let var_bound = max3(binder.ty.var_bound(),
                         val.var_bound(),
                         safe_minus_one(body.var_bound()));
    let has_locals = binder.ty.has_locals() || body.has_locals() || val.has_locals();
    Let { cache : ExprCache::mk(digest, var_bound, has_locals), 
          binder, 
          val, 
          body }.into() // InnerLevel -> Level
}

/// A `Local` represents a free variable. All `Local` terms have a unique
/// identifier (here we just use a monotonically increasing counter, with each
/// local's identifier being called a `serial`), and carries its type around.
/// As discussed above, locals must have the property that a clone/deep copy
/// is the only way to produce two local items with the same serial. All other
/// methods of constructing a local must produce a term with a unique identifier.
pub fn mk_local(name : impl Into<Name>, ty : Expr, style : BinderStyle) -> Expr {
    let binding = Binding::mk(name.into(), ty, style);
    let serial = LOCAL_SERIAL.fetch_add(1, Relaxed);
    let digest = hash64(&(serial, &binding));

    Local { cache : ExprCache::mk(digest, 0, true),
            serial,
            binder : binding }.into()  // InnerLevel -> Level
}

pub fn mk_local_w_serial(serial : u64, binding : &Binding, new_ty : Expr) -> Expr {
    let new_binding = binding.swap_ty(new_ty);
    let digest = hash64(&(serial, &new_binding));

    Local { cache : ExprCache::mk(digest, 0, true),
            serial,
            binder : new_binding }.into()  // InnerLevel -> Level
}


impl Expr {
    pub fn eq_mod_locals(&self, other : &Expr) -> bool {
        match (self.as_ref(), other.as_ref()) {
            (Var { dbj : dbj1, .. }, Var { dbj : dbj2, .. }) => dbj1 == dbj2,
            (Sort { level : lvl1, .. }, Sort { level : lvl2, .. }) => lvl1.eq_by_antisymm(lvl2),
            (Const { name : n1, levels : lvls1, .. }, Const { name : n2, levels : lvls2, ..  }) => (n1 == n2) && (is_def_eq_lvls(lvls1, lvls2)),
            (Lambda { binder : bind1, body : body1, .. }, Lambda { binder : bind2, body : body2, .. }) => {
                (&bind1.pp_name == &bind2.pp_name)
                && (bind1.ty.eq_mod_locals(&bind2.ty))
                && (bind1.style == bind2.style)
                && (body1.eq_mod_locals(&body2))
            },
            (App { fun : lhs1, arg : rhs1, .. }, App { fun : lhs2, arg : rhs2, .. }) => {
                (lhs1.eq_mod_locals(lhs2))
                && (rhs1.eq_mod_locals(rhs2))
            }
            (Pi { binder : bind1, body : body1, .. }, Pi { binder : bind2, body : body2, .. }) => {
                (&bind1.pp_name == &bind2.pp_name)
                && (bind1.ty.eq_mod_locals(&bind2.ty))
                && (bind1.style == bind2.style)
                && (body1.eq_mod_locals(&body2))

            },
            (Let { binder : bind1, val  : val1, body : body1, .. }, Let { binder : bind2, val : val2, body : body2, .. }) => {
                (&bind1.pp_name == &bind2.pp_name)
                && (bind1.ty.eq_mod_locals(&bind2.ty))
                && (bind1.style == bind2.style)
                && (body1.eq_mod_locals(&body2))
                && (val1.eq_mod_locals(&val2))

            },
            (Local { binder : bind1, .. }, Local { binder : bind2, .. }) => {
                (&bind1.pp_name == &bind2.pp_name)
                && (bind1.ty.eq_mod_locals(&bind2.ty))
                && (bind1.style == bind2.style)
            },
            _ => false
        }
    }

    /*
    pub fn cheap_beta_reduce(&self) -> Expr {
        match self.as_ref() {
            App { fun, .. } => {
                match fun.as_ref() {
                    Lambda { binder : bind, body, .. } => {
                        let (mut fn_clone, mut args) = self.get_app_args();
                        let mut i = 0;
                        while fn_clone.is_lambda() && i < args.len() {
                            i += 1;
                            fn_clone = fn_clone.get_binding_body().clone();
                        }

                        if (!fn_clone.has_vars()) {
                            fn_clone.mk_app_ptr(args.len() - i, args.iter().skip(i).collect::<Vec<&Expr>>())
                            fn_clone.mk_app_ptr(args.len() - i, args.iter().skip(i).collect::<Vec<&Expr>>())
                        } else if let Var { dbj : dbj_, .. } = fn_clone.as_ref() {
                            let dbj = *dbj_ as usize;
                            assert!(dbj < i);
                            let indexed = (&args[i - dbj - 1]).clone();
                            indexed.mk_app_ptr(args.len() - i, args.iter().skip(i).collect::<Vec<&Expr>>())

                        } else {
                            self.clone()
                        }
                    },
                    _ => self.clone()
                }
            },
            _ => self.clone()
        }
    }
    */


    // pointer
    pub fn check_ptr_eq(&self, other : &Expr) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    pub fn mk_local_declar_binders_only(&self) -> NanodaResult<Expr> {
        match self.as_ref() {
            Pi { binder, .. } | Lambda { binder, .. } => {
                Ok(mk_local(binder.pp_name.clone(), binder.ty.clone(), binder.style))
            },
            _ => Err(NotBinderErr(file!(), line!()))
        }
    }

/*
    pub fn compare_core(&self, other : &Expr, v : &mut Vec<(Expr, Expr)>) {
        let (mut c1, mut c2) = (self.clone(), other.clone());
        match (c1.as_ref(), c2.as_ref()) {
            (Var {..), Var {..)) | (Const {..), Const {..)) | (Sort {..), Sort {..)) => {
                v.push_back((c1, c2))
            },
            (Lambda {_, d1, b1), Lambda {_, d2, b2)) => {
                v.push((d1.ty.clone(), d2.ty.clone()));
                v.push((b1.clone(), b2.clone()));
            },
            (Pi {_, d1, b1), Pi {_, d2, b2)) => {
                v.push((d1.ty.clone(), d2.ty.clone()));
                v.push((b1.clone(), b2.clone()));
            },
            (Let {_, d1, v1, b1), Let {_, d2, v2, b2)) => {
                v.push((d1.ty.clone(), d2.ty.clone()));
                v.push((v1.clone(), v2.clone()));
                v.push((b1.clone(), b2.clone()));
            },
            (Local {_, _, b1), Local {_, _, b2)) => {
                v.push((b1.ty.clone(), b2.ty.clone()));

            },
            _ => ()
        }
    }
    */

    pub fn contains_subterm(&self, other : &Expr) -> bool {
        let mut todos = vec![self];
        while let Some(elem) = todos.pop() {
            if elem.eq_mod_locals(other) {
                return true
            } else {
                match elem.as_ref() {
                    Var {..} | Sort {..} | Const {..} => (),
                    App { fun, arg, .. } => {
                        todos.push(fun);
                        todos.push(arg);
                    },
                    Lambda { binder, body, .. } | Pi { binder, body, .. } => {
                        todos.push(&binder.ty);
                        todos.push(body);
                    },
                    Let { binder, val, body, .. } => {
                        todos.push(&binder.ty);
                        todos.push(val);
                        todos.push(body);

                    },
                    Local { binder, .. } => {
                        todos.push(&binder.ty);
                    }
                }
            }
        }

        false

    }


    pub fn find_matching(&self, f : impl Fn(&Expr) -> bool) -> Option<Expr> {
        let mut v = vec![self];

        while !v.is_empty() {
            let elem = v.pop().expect("impossible `None` in find_matching");

            if f(elem) {
                return Some(elem.clone())
            }

            match elem.as_ref() {
                Var {..} | Const {..} | Sort {..} => {
                    continue
                },
                App { fun, arg , .. } => {
                    v.push(fun);
                    v.push(arg);
                },
                Pi { binder, body, .. } | Lambda { binder, body, .. } => {
                    v.push(&binder.ty);
                    v.push(body);
                },
                Let { binder, val, body, .. } => {
                    v.push(&binder.ty);
                    v.push(val);
                    v.push(body);
                },
                Local { binder, .. } => {
                    v.push(&binder.ty);
                }

            }
        }

        assert!(v.len() == 0);
        return None
    }

    /*
    pub fn infer_implicit(&self, strict : bool) -> Expr {
        self.infer_implicit_core(u16::max_value(), strict)
    }

    pub fn infer_implicit_core(&self, num_params : u16, strict : bool) -> Expr {
        if num_params == 0 {
            return self.clone()
        } else if let Pi { binder, body, .. } = self.as_ref() {
            let new_body = body.infer_implicit_core(num_params - 1, strict);
            if (!binder.is_explicit()) {
                self.update_binding3(&binder.ty, &body)
            } else if (binder.ty.has_vars()) {
                self.update_binding4(&binder.ty, &new_body, BinderStyle::Implicit)
            } else {
                self.update_binding3(&binder.ty, &body)
            }
        } else {
            return self.clone()
        }
    }



    pub fn update_binding3(&self, new_dom : &Expr, new_body : &Expr) -> Expr {
        match self.as_ref() {
            Lambda { binder : dom, .. } => {
                let new_name = dom.pp_name.clone();
                let binding_info = dom.style;
                let binding = Binding::mk(new_name, new_dom.clone(), binding_info);
                mk_lambda(binding, new_body.clone())

            },
            Pi { binder : dom, .. } => {
                let new_name = dom.pp_name.clone();
                let binding_info = dom.style;
                let binding = Binding::mk(new_name, new_dom.clone(), binding_info);
                mk_pi(binding, new_body.clone())
            },
            owise => {
                eprintln!("update_binding3 exptected a pi or lambda, got : {:?}\n", owise);
                std::process::exit(-1)
            }
        }
    }

    pub fn update_binding4(&self, new_dom : &Expr, new_body : &Expr, style : BinderStyle) -> Expr {
        match self.as_ref() {
            Lambda { binder : dom, .. } => {
                let new_name = dom.pp_name.clone();
                let binding = Binding::mk(new_name, new_dom.clone(), style);
                mk_lambda(binding, new_body.clone())

            },
            Pi { binder : dom, .. } => {
                let new_name = dom.pp_name.clone();
                let binding = Binding::mk(new_name, new_dom.clone(), style);
                mk_pi(binding, new_body.clone())

            }
            owise => {
                eprintln!("update_binding4 exptected a pi or lambda, got : {:?}\n", owise);
                std::process::exit(-1)
            }
        }
    }
    */

    pub fn eq_mod_serial(&self, other : &Expr) -> bool {
        match (self.as_ref(), other.as_ref()) {
            (Local { binder : b1, .. }, Local { binder : b2, .. }) => b1 == b2,
            _ => false
        }
    }


    pub fn is_local(&self) -> bool {
        match self.as_ref() {
            Local {..} => true,
            _ => false
        }
    }

// from cpp
    pub fn get_sort_level(&self) -> NanodaResult<&Level> {
        match self.as_ref() {
            Sort { level, .. } => Ok(level),
            _ => Err(NotSortErr(file!(), line!())),
        }
    }

    pub fn get_local_type(&self) -> NanodaResult<&Expr> {
        match self.as_ref() {
            Local { binder, .. } => Ok(&binder.ty),
            _ => Err(NotLocalErr(file!(), line!()))
        }
    }


    pub fn get_digest(&self) -> u64 {
        self.as_ref().get_cache().digest
    }

    pub fn has_locals(&self) -> bool {
        self.as_ref().get_cache().has_locals
    }

    pub fn has_vars(&self) -> bool {
        self.as_ref().get_cache().var_bound > 0
    }

    pub fn var_bound(&self) -> u16 {
        self.as_ref().get_cache().var_bound
    }



    // !! Partial function !!
    pub fn lc_binding(&self) -> &Binding {
        match self.as_ref() {
            Local { binder, .. } => binder,
            owise => errors::err_lc_binding(line!(), owise)
        }
    }

    // !! Partial function !!
    // only used once in the pretty printer.
    pub fn binder_is_pi(&self) -> bool {
        match self.as_ref() {
            Pi {..} => true,
            Lambda {..} => false,
            owise => errors::partial_is_pi(line!(), owise)
        }
    }

    // only used in new_inductive
    pub fn is_pi(&self) -> bool {
        match self.as_ref() {
            Pi {..} => true,
            _ => false
        }
    }

    pub fn is_lambda(&self) -> bool {
        match self.as_ref() {
            Lambda {..} => true,
            _ => false
        }
    }

    /// Only used in the pretty printer.
    pub fn swap_local_binding_name(&self, new_name : &Name) -> Expr {
        match self.as_ref() {
            Local { serial, binder, .. } => {
                let new_binding = Binding::mk(new_name.clone(), binder.ty.clone(), binder.style);
                let digest = hash64(&(serial, &binder));

                Local { cache : ExprCache::mk(digest, 0, true),
                      binder : new_binding,
                      serial : *serial }.into()  // InnerLevel -> Level
            },
            owise => errors::err_swap_local_binding_name(line!(), owise),
        }
    }

    /// !! Partial function !!  
    /// If the expression is a Local, returns its unique identifier/serial number.
    /// Else kills the program with a fatal error.
    pub fn get_serial(&self) -> u64 {
        match self.as_ref() {
            Local { serial, .. } => *serial,
            owise => errors::err_get_serial(line!(), owise)
        }
    }

    /// This is the primitive joining of applying two expressions with the arrow
    /// constructor. Given some `e1` and `e2`, constructs `e1 → e2` by turning
    /// it into `Π (e1), e2` 
    pub fn mk_arrow(&self, other : &Expr) -> Expr {
        let binding = Binding::mk(mk_anon(), self.clone(), BinderStyle::Default);
        mk_pi(binding, other.clone())
    }


    /// The goal here is to traverse an expression, replacing `Local` terms with `Var`
    /// terms where possible, while caching terms we've already performed 
    /// substitution on. 
    /// It's a relatively generic traversal where we cache expressions to that we 
    /// don't have to fully evaluate subtrees if we already know how they evaluate.
    /// The 'interesting' case is when we run across a Local `L` in our tree; we look 
    /// in the collection `lcs` for a term `L'` such that `L' = L`. If there isn't one,
    /// just return `L`. If there IS one, we note the position/index of `L'` in `lcs`,
    /// create a variable whose inner index is pos(L'), and return the newly created
    /// variable.
    /// `offset` is used to mark the transition from one binder's scope into another;
    /// you can see that it only increments as we recurse into the body of a binder
    /// (Lambda, Pi, or Let term).
    pub fn abstract_<'e, I>(&self, locals : I) -> Expr 
    where I : Iterator<Item = &'e Expr> + Clone {
        if !self.has_locals() {
            return self.clone() 
        }
        let mut cache = OffsetCache::new();
        self.abstract_core(0usize, &mut cache, locals)
    }

    fn abstract_core<'e, I>(&self, offset : usize, cache : &mut OffsetCache, locals : I) -> Expr 
    where I : Iterator<Item = &'e Expr> + Clone {
        if !self.has_locals() {
            self.clone()
        } else if let Some(cached) = cache.get(&self, offset) {
            cached.clone()
        } else if let Local { serial, .. } = self.as_ref() {
            locals.clone()
            .position(|lc| lc.get_serial() == *serial)
            .map_or_else(|| self.clone(), |position| {
                mk_var(position + offset)
            })
        } else if let Some(cached) = cache.get(&self, offset) {
            cached.clone()
        } else {
            let result = match self.as_ref() {
                App { fun, arg, .. } => {
                    let new_fun = fun.abstract_core(offset, cache, locals.clone());
                    let new_arg = arg.abstract_core(offset, cache, locals);
                    mk_app(new_fun, new_arg)
                },
                Lambda { binder, body, .. } => {
                    let new_binder_ty = binder.ty.abstract_core(offset, cache, locals.clone());
                    let new_body = body.abstract_core(offset + 1, cache, locals);
                    mk_lambda(binder.swap_ty(new_binder_ty), new_body)
                }
                Pi { binder, body, .. } => {
                    let new_binder_ty = binder.ty.abstract_core(offset, cache, locals.clone());
                    let new_body = body.abstract_core(offset + 1, cache, locals);
                    mk_pi(binder.swap_ty(new_binder_ty), new_body)
                },
                Let { binder, val, body, .. } => {
                    let new_binder_ty = binder.ty.abstract_core(offset, cache, locals.clone());
                    let new_val = val.abstract_core(offset, cache, locals.clone());
                    let new_body = body.abstract_core(offset + 1, cache, locals);
                    mk_let(binder.swap_ty(new_binder_ty), new_val, new_body)
                },
                owise => unreachable!("Illegal match item in Expr::abstract_core {:?}\n", owise)
            };

            cache.insert(self.clone(), result.clone(), offset);
            result
        }
    }

    //pub fn instantiate_rev<'e>(&self, es : impl Iterator<Item = &'e Expr>) -> Expr {
    //    unimplemented!()
    //}


    /// Similar shape to abstract; we traverse an expression, but this time we want
    /// to substitute variables for other expressions, stil carrying a cache and
    /// using an offset to track the transition into the body of successive binders.
    /// The interesting case this time is when we run across a Variable; we
    /// make sure the index is in bounds, then use it to index into the sequence
    /// `es`, replacing our Variable with `es`[idx].
    /// `instantiate_core` is the single most time consuming part of running
    /// the type checker, with some of the expression trees it has to traverse
    /// spanning millions of nodes, so if you're going to implement a 
    /// type checker yourself and you want it to be fast, figure out a way
    /// to make these functions efficient.
    pub fn instantiate_w_offset<'e, I>(&self, offset : usize, es : I) -> Expr 
    where I : Iterator<Item = &'e Expr> + Clone {
        let mut cache = OffsetCache::new();
        self.instantiate_core(offset, &mut cache, es)
    }


    pub fn instantiate<'e, I>(&self, es : I) -> Expr 
    where I : Iterator<Item = &'e Expr> + Clone {
       let mut cache = OffsetCache::new();
        self.instantiate_core(0usize, &mut cache, es)
    } 


    fn instantiate_core<'e, I>(&self, offset : usize, cache : &mut OffsetCache, es : I) -> Self 
    where I : Iterator<Item = &'e Expr> + Clone {
        if self.var_bound() as usize <= offset {
            self.clone()
        } else if let Var { dbj, .. } = self.as_ref() {
            es.clone()
              .nth(*dbj as usize - offset)
              .cloned()
              .unwrap_or_else(|| self.clone())
        } else if let Some(cached) = cache.get(self, offset) {
            cached.clone()
        } else {
            let calcd = match self.as_ref() {
                App { fun, arg, .. } => {
                    let new_fun = fun.instantiate_core(offset, cache, es.clone());
                    let new_arg = arg.instantiate_core(offset, cache, es);
                    mk_app(new_fun, new_arg)
                },
                | Lambda { binder, body, .. } => {
                    let new_binder_ty = binder.ty.instantiate_core(offset, cache, es.clone());
                    let new_body = body.instantiate_core(offset + 1, cache, es);
                    mk_lambda(binder.swap_ty(new_binder_ty), new_body)
                }
                | Pi { binder, body, .. } => {
                    let new_binder_ty = binder.ty.instantiate_core(offset, cache, es.clone());
                    let new_body = body.instantiate_core(offset + 1, cache, es);
                    mk_pi(binder.swap_ty(new_binder_ty), new_body)
                },
                Let { binder, val, body, .. } => {
                    let new_binder_ty = binder.ty.instantiate_core(offset, cache, es.clone());
                    let new_val = val.instantiate_core(offset, cache, es.clone());
                    let new_body = body.instantiate_core(offset + 1, cache, es);
                    mk_let(binder.swap_ty(new_binder_ty), new_val, new_body)
                },
                owise => unreachable!("Illegal match result in Expr::instantiate_core {:?}\n", owise)
            };

            cache.insert(self.clone(), calcd.clone(), offset);
            calcd
        }

    }

// If it returns `Some`, you've replaced the whole sub-tree, so you don't need
// to continue iterating over the children.
    pub fn replace_expr(&self, f : impl Fn(&Expr) -> Option<Expr> + Copy) -> Expr {

        let mut cache = OffsetCache::new();
        self.replace_expr_core(0usize, &mut cache, f)
    } 

    fn replace_expr_core(&self, offset : usize, cache : &mut OffsetCache, f : impl Fn(&Expr) -> Option<Expr> + Copy) -> Self {
        if let Some(cached) = cache.get(&self, offset) {
            return cached.clone()
        } else if let Some(e) = f(self) {
            cache.insert(self.clone(), e.clone(), offset);
            e
        } else {
            let result = match self.as_ref()  {
                App { fun, arg, .. } => {
                    let new_fun = fun.replace_expr_core(offset, cache, f);
                    let new_arg = arg.replace_expr_core(offset, cache, f);
                    mk_app(new_fun, new_arg)
                },
                | Lambda { binder, body, .. } => {
                    let new_binder_ty = binder.ty.replace_expr_core(offset, cache, f);
                    let new_body = body.replace_expr_core(offset + 1, cache, f);
                    mk_lambda(binder.swap_ty(new_binder_ty), new_body)
                }
                | Pi { binder, body, .. } => {
                    let new_binder_ty = binder.ty.replace_expr_core(offset, cache, f);
                    let new_body = body.replace_expr_core(offset + 1, cache, f);
                    mk_pi(binder.swap_ty(new_binder_ty), new_body)
                },
                Let { binder, val, body, .. } => {
                    let new_binder_ty = binder.ty.replace_expr_core(offset, cache, f);
                    let new_val = val.replace_expr_core(offset, cache, f);
                    let new_body = body.replace_expr_core(offset + 1, cache, f);
                    mk_let(binder.swap_ty(new_binder_ty), new_val, new_body)
                },
                Local { binder, .. } => {
                    let new_binder_ty = binder.ty.replace_expr_core(offset, cache, f);
                    mk_local(binder.pp_name.clone(), new_binder_ty, binder.style)
                },
                Var {..} | Sort {..} | Const {..} => self.clone()
            };
            cache.insert(self.clone(), result.clone(), offset);
            result
        }
    }


    /// This just performs variable substitution by going through
    /// the `Level` items contained in `Sort` and `Const` expressions.
    /// For all levels therein, attempts to replace `Level::Param`
    /// items with something in the `substs` mapping, which maps
    /// (Level::Param |-> Level)
    pub fn instantiate_lparams<'l, I>(&self, substs : I) -> Expr 
    where I : Iterator<Item = (&'l Level, &'l Level)> + Clone {
        if substs.clone().any(|(l, r)| l != r) {
            match self.as_ref() {
                App { fun : lhs, arg : rhs, .. } => {
                    let new_lhs = lhs.instantiate_lparams(substs.clone());
                    let new_rhs = rhs.instantiate_lparams(substs);
                    mk_app(new_lhs, new_rhs)
                },
                Lambda { binder, body, .. } => {
                    let new_binder_ty = binder.ty.instantiate_lparams(substs.clone());
                    let new_body = body.instantiate_lparams(substs);
                    mk_lambda(binder.swap_ty(new_binder_ty), new_body)

                }
                Pi { binder, body, .. } => {
                    let new_binder_ty = binder.ty.instantiate_lparams(substs.clone());
                    let new_body = body.instantiate_lparams(substs);
                    mk_pi(binder.swap_ty(new_binder_ty), new_body)
                },

                Let { binder, val, body, .. } => {
                    let new_binder_ty = binder.ty.instantiate_lparams(substs.clone());
                    let new_val = val.instantiate_lparams(substs.clone());
                    let new_body = body.instantiate_lparams(substs);
                    mk_let(binder.swap_ty(new_binder_ty), new_val, new_body)
                },
                Local { binder, .. } => {
                    let new_binder_ty = binder.ty.instantiate_lparams(substs);
                    binder.swap_ty(new_binder_ty).as_local()
                },
                Var {..} => self.clone(),
                Sort { level : lvl, .. } => {
                    let instd_level = lvl.instantiate_lparams(substs);
                    mk_sort(instd_level)
                },
                Const { name, levels : lvls, .. } => {
                    let new_levels = lvls.iter()
                                         .map(|x| (x.instantiate_lparams(substs.clone())))
                                         .collect::<Vec<Level>>();
                    mk_const(name.clone(), new_levels)
                }
            }
        } else {
            self.clone()
        }
    }




    /// Note for non-rust users, IntoIterator is idempotent over Iterators; if
    /// we pass this something that's already an interator, nothing happens. 
    /// But if we pass it something that isnt YET an iterator, it will turn 
    /// it into one for us. Given a list of expressions [X_1, X_2, ... X_n] and 
    /// some expression F, iteratively apply the `App` constructor to get :
    ///```pseudo
    /// App { ... App {App {F, X_1), X_2)...  X_n)
    /// 
    ///              App
    ///            /    \
    ///          App    X_n...
    ///         ...
    ///       /    \
    ///      App    X_2...
    ///    /    \
    ///   F     X_1...
    ///```
    pub fn foldl_apps<'q, Q>(&self, apps : Q) -> Expr 
    where Q : IntoIterator<Item = &'q Expr> {
        let mut acc = self.clone();
        for next in apps {
            acc = mk_app(acc, next.clone())
        }
        acc
    }

    

    /// From an already constructed tree, unfold all consecutive 
    /// `App` constructors along their spine from right to left.
    ///```pseudo
    /// `App` nodes, and the bottom left expression `F`.
    /// 
    ///              App      =>   (F, [X_n subtree, X_2 subtree, X_1 subtree])
    ///            /    \
    ///          App    X_n...
    ///         ...      
    ///       /    \
    ///      App    X_2...
    ///    /    \    
    ///   F     X_1...
    ///```
    pub fn unfold_apps(&self) -> (&Expr,  Vec<&Expr>) {
        let (mut _fn, mut acc) = (self, Vec::with_capacity(40));
        while let App { fun, arg, .. } = _fn.as_ref() {
            acc.push(arg);
            _fn = fun;
        }
        (_fn, acc)
    }

    // FIXME inefficient
    pub fn unfold_apps_rev(&self) -> (&Expr, Vec<&Expr>) {
        let (fun, mut args) = self.unfold_apps();
        args.reverse();
        (fun, args)
    }


    pub fn unfold_apps_fn(&self) -> &Expr {
        let mut it = self;
        while let App { fun, .. } = it.as_ref() {
            it = fun;
        }
        it
    }

    /// Given two expressions `E` and `L`, where `L` is known to be a Local :
    ///```pseudo
    /// let E = E.abstract(L)
    /// return (Π (L) (E'))
    ///```
    pub fn apply_pi(&self, domain : &Expr) -> Expr {
        assert!(domain.is_local());
        let abstracted = self.clone().abstract_(Some(domain).into_iter());
        mk_pi(Binding::from(domain), abstracted)
    }


    /// Given a list of Local expressions [L_1, L_2, ... L_n] and a 
    /// body `E : Expr`, use your favorite method (fold_right is 
    /// nice) and the Pi constructor to make :
    ///
    ///```pseudo
    /// (Π L_1, (Π L_2, ... (Π L_n, E)))
    ///```
    ///
    ///```pseudo
    ///              Π 
    ///           /    \
    ///          L_1    Π 
    ///               /   \
    ///             L_2   ...
    ///                    Π 
    ///                  /  \
    ///                L_n   E
    ///```
    /// same as fold_pis, but generic over iterators.
    pub fn fold_pis<'q, Q>(&self, doms : Q) -> Expr 
    where Q : Iterator<Item = &'q Expr> + DoubleEndedIterator {
        let mut acc = self.clone();
        for next in doms.rev() {
            acc = acc.apply_pi(next)
        }
    
        acc
    }

    /// This unfolds consecutive applications of `Pi` into the "core" term,
    /// and a list of binders pulled from the Pi applications. This is one of the few
    /// places where we use an in-place mutation since it's used iteratively and we don't
    /// want a bunch of vector allocations for no reason.
    /// An example of its application might look like :
    ///```pseudo
    ///  let t = Π α, (Π β, (Π γ, E))
    ///  let binder_acc = []
    ///  ...unfold_pis(t)
    /// assert (t = E) && (binder_acc = [α, β, γ])
    ///
    pub fn unfold_pis(&mut self, binder_acc : &mut Vec<Expr>) {
        while let Pi { binder, body, .. } = self.as_ref() {
            let local = binder.clone().as_local();
            let instd = body.instantiate(Some(&local).into_iter());
            binder_acc.push(local);
            std::mem::replace(self, instd);
        }
    }

    /// Given two expressions `E` and `L`, where `L` is known to be a Local,
    ///```pseudo
    ///  let E' = E.abstract(L)
    ///  return (λ L, E')
    ///```
    pub fn apply_lambda(&self, domain : &Expr) -> Expr {
        assert!(domain.is_local());
        let abstracted = self.clone().abstract_(Some(domain).into_iter());
        mk_lambda(Binding::from(domain), abstracted)
    }

    pub fn get_const_name(&self) -> Option<&Name> {
        match self.as_ref() {
            Const { name, .. } => Some(name),
            _ => None
        }
    }

    pub fn get_const_levels(&self) -> Option<&Vec<Level>> {
        match self.as_ref() {
            Const { levels, .. } => Some(levels),
            _owise => None
        }
    }

    pub fn try_const_fields(&self) -> Option<(&Name, &Vec<Level>)> {
        match self.as_ref() {
            Const { name, levels, .. } => Some((name, levels)),
            _ => None
        }
    }

    pub fn get_const_levels_inf(&self) -> &Vec<Level> {
        match self.as_ref() {
            Const { levels, .. } => levels,
            _owise => panic!("const_level_inf")
        }
    }


    pub fn get_const_name_opt(&self) -> Option<&Name> {
        match self.as_ref() {
            Const { name, .. } => Some(name),
            _owise => None
        }
    }

    pub fn is_const(&self) -> bool {
        match self.as_ref() {
            Const {..} => true,
            _ => false
        }
    }

    pub fn is_app(&self) -> bool {
        match self.as_ref() {
            App {..} => true,
            _ => false
        }
    }


    /// Given a list of Local expressions [L_1, L_2, ... L_n] and a body `E : Expr`, 
    /// use your favorite method (here we use a right fold) and the Lambda constructor to make :
    ///```pseudo
    /// (λ  L_1, (λ  L_2, ... (λ  L_n, E)))
    ///
    ///              λ 
    ///           /    \
    ///          L_1     λ 
    ///                /   \
    ///              L_2    ... 
    ///                      λ 
    ///                    /  \
    ///                  L_n   E
    ///```
    /// same as fold_lambdas, but generic over iterators.
    pub fn fold_lambdas<'q, Q>(&self, doms : Q) -> Expr 
    where Q : Iterator<Item = &'q Expr> + DoubleEndedIterator {
        let mut acc = self.clone();
        for next in doms.rev() {
            acc = acc.apply_lambda(next)
        }
        acc
    }

}

impl Hash for InnerExpr {
    fn hash<H : Hasher>(&self, state : &mut H) {
        self.get_digest().hash(state);
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum InnerExpr {
    Var    { cache : ExprCache, dbj : usize },
    Sort   { cache : ExprCache, level : Level },
    Const  { cache : ExprCache, name : Name, levels : Vec<Level> },
    App    { cache : ExprCache, fun : Expr,  arg : Expr },
    Lambda { cache : ExprCache, binder : Binding, body : Expr } ,
    Pi     { cache : ExprCache, binder : Binding, body : Expr },
    Let    { cache : ExprCache, binder : Binding, val : Expr, body : Expr },
    Local  { cache : ExprCache, binder : Binding, serial : u64 },
}

impl InnerExpr {
    pub fn get_digest(&self) -> u64 {
        self.get_cache().digest
    }

    pub fn get_cache(&self) -> ExprCache {
        match self {
            | Var    { cache , .. } 
            | Sort   { cache , .. } 
            | Const  { cache , .. } 
            | Local  { cache , .. } 
            | App    { cache , .. } 
            | Lambda { cache , .. } 
            | Pi     { cache , .. } 
            | Let    { cache , .. }  => *cache
        }
    }
}

/// Caches an expression's hash digest, number of bound variables, and whether
/// or not it contains locals. The important part of this is it's calculated
/// as an expression tree is constructed, where each node's cache captures
/// the information for itself and for its entire subtree, since IE the hash digest
/// is the digest of its component nodes, which are in turn the comopsition of THEIR
/// component nodes, etc. This is extremely important for performance reasons
/// since we want to do things like use hash tables as caches. If we implemented things
/// naively, we would be rehashing the entire tree every time we wanted to look a term
/// up in a cache, and expression trees can get very, very large. Instead we tell
/// all hash-keyed data structures to (more or less) pull the cache.digest value off
/// and use that instead.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct ExprCache {
    digest : u64,
    var_bound : u16,
    has_locals : bool,
}

impl std::fmt::Debug for ExprCache {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "")
    }
}


impl ExprCache {
    pub fn mk(digest : u64, var_bound : u16, has_locals : bool) -> Self {
        ExprCache {
            digest,
            var_bound,
            has_locals,
        }
    }

}


impl std::convert::AsRef<InnerExpr> for Expr {
    fn as_ref(&self) -> &InnerExpr {
        match self {
            Expr(arc) => arc.as_ref()
        }
    }
}

impl From<InnerExpr> for Expr {
    fn from(x : InnerExpr) -> Expr {
        Expr(Arc::new(x))
    }
}


// !! Partial function !!
impl From<&Expr> for Binding {
    fn from(e : &Expr) -> Binding {
        match e.as_ref() {
            Local { binder, .. } => binder.clone(),
            owise => errors::err_binding_lc(line!(), owise),
        }
    }
}


/// Mapping of ((Expr × int) |-> Expr) that says "(expression A at offset B) 
/// maps to (expression C)". There are multiple ways to do this, 
/// but this way of doing itturned out to be (much to my surprise, 
/// shout-outs to @GEbner) faster than (Expr x Int) -> Expr, probably 
/// due in large part because of how tuples work in Rust. 

pub struct OffsetCache(Vec<HashMap<Expr, Expr>>);

impl OffsetCache {
    pub fn new() -> Self {
        OffsetCache(Vec::with_capacity(200))
    }


    pub fn get(&self, e : &Expr, offset : usize) -> Option<&Expr> {
        match self {
            OffsetCache(inner) => inner.get(offset)?.get(e)
        }
    }

    pub fn insert(&mut self, e1 : Expr, e2 : Expr, offset : usize) {
        let map_vec = match self {
            OffsetCache(x) => x
        };

        while map_vec.len() <= offset {
            map_vec.push(HashMap::with_capacity(50));
        }

        match map_vec.get_mut(offset) {
            Some(v) => v.insert(e1, e2),
            None => errors::err_offset_cache(line!(), offset, map_vec.len()),
        };
    }

}

/// For some expression `E`, traverse `E`, putting the `Name` field 
/// of any constant into a set `S`. This is only used once, when compiling 
/// a `Definition`; we get all of the names out of an expression's constant terms,
/// and use them to look up the height of those definitions in the environment. 
/// There's more information about definition height under tc::def_height().
/// This isn't defined as an associated method because it wanted more 
/// detailed lifetime information than could be provided by `self`.   
pub fn unique_const_names<'l, 's>(n : &'l Expr) -> HashSet<&'l Name> {
    let mut acc = HashSet::with_capacity(80);
    let mut cache = HashSet::with_capacity(200);
    unique_const_names_core(n, &mut acc, &mut cache);
    acc
}

pub fn unique_const_names_core<'l, 's>(n : &'l Expr, 
                                       s : &'s mut HashSet<&'l Name>, 
                                       cache : &'s mut HashSet<&'l Expr>) {
    if cache.contains(n) {
        return
    } else {
        match n.as_ref() {
            App { fun, arg, .. } => {
                unique_const_names_core(fun, s, cache);
                unique_const_names_core(arg, s, cache);
            },
            | Lambda { binder, body, .. }
            | Pi { binder, body, .. } => {
                unique_const_names_core(&binder.ty, s, cache);
                unique_const_names_core(&body, s, cache);

            },
            Let { binder, val, body, .. } => {
                unique_const_names_core(&binder.ty, s, cache);
                unique_const_names_core(&val, s, cache);
                unique_const_names_core(&body, s, cache);
            },
            Const { name, .. } => {
                s.insert(name);
            },
            _ => (),
        };
        cache.insert(n);
    }
}

/// Given some expression `E` and a set of levels `S_X`, collect all 
/// Level::Param elements in `E` into a set `S_E`, and determine whether 
/// or not `S_E` is a subset of `S_X`. This only gets used once, in 
/// the process of checking the type field of a `Declaration`, in order 
/// to ensure that all of the universe parameters being used in some
/// declaration's type are properly declared in it's separate 
/// `univ_params` field.
pub fn univ_params_subset<'l, 's>(e : &'l Expr, other : &'s HashSet<&'l Level>) -> bool {
    let mut const_names_in_e = HashSet::with_capacity(40);
    univ_params_subset_core(e, &mut const_names_in_e);

    const_names_in_e.is_subset(&other)
}

fn univ_params_subset_core<'l, 's>(e : &'l Expr, s : &'s mut HashSet<&'l Level>) {
    match e.as_ref() {
        App { fun, arg, .. } => {
            univ_params_subset_core(fun, s);
            univ_params_subset_core(arg, s);
        },
        | Lambda { binder, body, .. }
        | Pi { binder, body, .. } => {
            univ_params_subset_core(&binder.ty, s);
            univ_params_subset_core(body, s);
        },
        Let { binder, val, body, .. } => {
            univ_params_subset_core(&binder.ty, s);
            univ_params_subset_core(val, s);
            univ_params_subset_core(body, s);
        },
        Sort { level, .. } => { s.extend(unique_univ_params(level)); },
        Const { levels, .. } => for l in levels {
            s.extend(unique_univ_params(l));
        },
        _ => ()
    }
}



impl std::fmt::Debug for InnerExpr {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Var { dbj : idx, .. } => {
                write!(f, "Var{}", idx)
            },
            Sort { level, .. } => {
                write!(f, "Sort {:?}", level)
            },
            Const { name, levels, .. } => {
                write!(f, "Const ({:?}, {:?})", name, levels)
            },
            App { fun, arg, .. } => {
                write!(f, "App ({:?}, {:?})", fun, arg)
                //write!(f, "{:?} {:?}", fun, arg)
            },
            Lambda { binder, body, .. } => {
                write!(f, "λ {:?}, ({:?})", binder, body)
            },
            Pi { binder, body, .. } => {
                write!(f, "Π {:?}, ({:?})", binder, body)
            },
            Let { binder, val, body, .. } => {
                write!(f, "let {:?} := {:?} in {:?}", binder, val, body)
            },
            Local { binder, .. } => {
                //lt truncated = serial.to_string().chars().take(6).collect::<String>();
                write!(f, "(serial of : {:?}", binder)
            }
        }
    }
}

impl std::fmt::Debug for Binding {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self.style {
            BinderStyle::Default => format!("({} : {:?})", self.pp_name, self.ty),
            BinderStyle::Implicit => format!("{{{} : {:?}}}", self.pp_name, self.ty),
            BinderStyle::InstImplicit => format!("[{} : {:?}]", self.pp_name, self.ty),
            BinderStyle::StrictImplicit => format!("{{{{{} : {:?}}}}}", self.pp_name, self.ty),
        };
        write!(f, "{}", s)
    }
}




