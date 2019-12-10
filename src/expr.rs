use std::sync::Arc;
use std::cmp::max;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::Relaxed;
use std::hash::{ Hash, Hasher };

use fxhash::hash64;
use hashbrown::{ HashMap, HashSet };

use crate::name::{ Name, mk_anon };
use crate::level::{ Level, unique_univ_params, mk_zero };
use crate::utils::{ safe_minus_one, max3 };
use crate::errors;

use InnerExpr::*;

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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

    pub fn as_local(self) -> Expr {
        let serial = LOCAL_SERIAL.fetch_add(1, Relaxed);
        let digest = hash64(&(serial, &self));
        Local(ExprCache::mk(digest, 0, true), serial, self).into()
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
    Sort(PROP_CACHE, mk_zero()).into() // into Level from InnerLevel
}


/// Makes a variable expression which contains a 
/// [De Brujin index](https://en.wikipedia.org/wiki/De_Bruijn_index).
pub fn mk_var(idx : u64) -> Expr {
    let digest = hash64(&(idx));
    Var(ExprCache::mk(digest, idx as u16 + 1, false), idx).into() // InnerLevel -> Level
}

/// Makes a node in the tree, joining two expressions as application.
pub fn mk_app(lhs : Expr, rhs : Expr) -> Expr {
    let digest = hash64(&(lhs.get_digest(), rhs.get_digest()));
    let var_bound = lhs.var_bound().max(rhs.var_bound());
    let has_locals = lhs.has_locals() || rhs.has_locals();
    App(ExprCache::mk(digest, var_bound, has_locals), lhs, rhs).into() // InnerLevel -> Level 
}

/// Represents a Sort/Level/Universe. You can read more about these in 
/// sources like Theorem Proving in Lean.
pub fn mk_sort(level : Level) -> Expr {
    let digest = hash64(&level);
    Sort(ExprCache::mk(digest, 0, false), level).into() // InnerLevel -> Level 
}

/// A constant; represents a reference to a declaration that has already
/// been added to the environment.
pub fn mk_const(name : impl Into<Name>, levels : impl Into<Arc<Vec<Level>>>) -> Expr {
    let name = name.into();
    let levels = levels.into();
    let digest = hash64(&(&name, &levels));
    Const(ExprCache::mk(digest, 0, false), name, levels).into()
}

/// A lambda function.
pub fn mk_lambda(domain : Binding, body: Expr) -> Expr {
    let digest = hash64(&(LAMBDA_HASH, &domain, body.get_digest()));
    let var_bound = max(domain.ty.var_bound(), 
                        safe_minus_one(body.var_bound()));
    let has_locals = domain.ty.has_locals() || body.has_locals();
    Lambda(ExprCache::mk(digest, var_bound, has_locals), domain, body).into() // InnerLevel -> Level
}

/// A Pi (dependent function) type.
pub fn mk_pi(domain : Binding, body: Expr) -> Expr {
    let digest = hash64(&(PI_HASH, &domain, body.get_digest()));
    let var_bound = max(domain.ty.var_bound(),
                        safe_minus_one(body.var_bound()));
    let has_locals = domain.ty.has_locals() || body.has_locals();
    Pi(ExprCache::mk(digest, var_bound, has_locals), domain, body).into() // InnerLevel -> Level
}

/// A let binding, IE `let (x : nat) := 5  in 2 * x`
pub fn mk_let(domain : Binding, val : Expr, body : Expr) -> Expr {
    let digest = hash64(&(&domain, val.get_digest(), body.get_digest()));
    let var_bound = max3(domain.ty.var_bound(),
                         val.var_bound(),
                         safe_minus_one(body.var_bound()));
    let has_locals = domain.ty.has_locals() || body.has_locals() || val.has_locals();
    Let(ExprCache::mk(digest, var_bound, has_locals), domain, val, body).into() // InnerLevel -> Level
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

    Local(ExprCache::mk(digest, 0, true),
          serial,
          binding).into()  // InnerLevel -> Level
}


impl Expr {

    pub fn is_local(&self) -> bool {
        match self.as_ref() {
            Local(..) => true,
            _ => false
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
            Local(.., binding) => binding,
            owise => errors::err_lc_binding(line!(), owise)
        }
    }

    // !! Partial function !!
    // only used once in the pretty printer.
    pub fn binder_is_pi(&self) -> bool {
        match self.as_ref() {
            Pi(..) => true,
            Lambda(..) => false,
            owise => errors::partial_is_pi(line!(), owise)
        }
    }

    /// Only used in the pretty printer.
    pub fn swap_local_binding_name(&self, new_name : &Name) -> Expr {
        match self.as_ref() {
            Local(.., serial, binding) => {
                let new_binding = Binding::mk(new_name.clone(), binding.ty.clone(), binding.style);
                let digest = hash64(&(serial, &binding));

                Local(ExprCache::mk(digest, 0, true),
                      *serial,
                      new_binding).into()  // InnerLevel -> Level
            },
            owise => errors::err_swap_local_binding_name(line!(), owise),
        }
    }

    /// !! Partial function !!  
    /// If the expression is a Local, returns its unique identifier/serial number.
    /// Else kills the program with a fatal error.
    pub fn get_serial(&self) -> u64 {
        match self.as_ref() {
            Local(_, serial, _) => *serial,
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
    pub fn abstract_<'e>(&self, lcs : impl Iterator<Item = &'e Expr> + Clone) -> Expr {
        if !self.has_locals() {
            self.clone() 
        } else {
            let mut cache = OffsetCache::new();
            self.abstract_core(0usize, lcs.clone(), &mut cache)
        }
    }

    fn abstract_core<'e>(&self, offset : usize, locals : impl Iterator<Item = &'e Expr> + Clone, cache : &mut OffsetCache) -> Expr {
        if !self.has_locals() {
            self.clone()
        } else if let Local(_, serial, _) = self.as_ref() {
            locals.clone()
                  .position(|lc| lc.get_serial() == *serial)
                  .map_or_else(|| self.clone(), |position| {
                      mk_var((position + offset) as u64)
                   })
        } else {
            cache.get(self, offset).cloned().unwrap_or_else(|| {
                let result = match self.as_ref() {
                    App(_, lhs, rhs) => {
                        let new_lhs = lhs.abstract_core(offset, locals.clone(), cache);
                        let new_rhs = rhs.abstract_core(offset, locals, cache);
                        mk_app(new_lhs, new_rhs)
                    },
                    Lambda(_, dom, body) => {
                        let new_domty = dom.ty.abstract_core(offset, locals.clone(), cache);
                        let new_body = body.abstract_core(offset + 1, locals, cache);
                        mk_lambda(dom.swap_ty(new_domty), new_body)
                    }
                    Pi(_, dom, body) => {
                        let new_domty = dom.ty.abstract_core(offset, locals.clone(), cache);
                        let new_body = body.abstract_core(offset + 1, locals, cache);
                        mk_pi(dom.swap_ty(new_domty), new_body)
                    },
                    Let(_, dom, val, body) => {
                        let new_domty = dom.ty.abstract_core(offset, locals.clone(), cache);
                        let new_val = val.abstract_core(offset, locals.clone(), cache);
                        let new_body = body.abstract_core(offset + 1, locals, cache);
                        mk_let(dom.swap_ty(new_domty), new_val, new_body)
                    },
                    owise => unreachable!("Illegal match item in Expr::abstract_core {:?}\n", owise)
                };

                cache.insert(self.clone(), result.clone(), offset);
                result
            })
        }
    }

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
    pub fn instantiate<'e>(&self, es : impl Iterator<Item = &'e Expr> + Clone) -> Expr {
        if self.var_bound() as usize == 0 {
            self.clone()
        } else {
            let mut cache = OffsetCache::new();
            self.instantiate_core(0usize, es.clone(), &mut cache)
        }
    } 

    // The way 'offset' works is that it pushes the index further left
    // in the vec it's indexing. Or you can think of it as pushing `None` values
    // onto the left of the collection, so an offset of 3 would become :
    // [None, None, None, e1, e2, e3, e4, e5]
    //   0     1     2    3   4   5   6   7
    fn instantiate_core<'e>(&self, offset : usize, es : impl Iterator<Item = &'e Expr> + Clone, cache : &mut OffsetCache) -> Self {
        if self.var_bound() as usize <= offset {
            return self.clone()
        } else if let Var(_, idx_) = self.as_ref() {
            es.clone()
              .nth((*idx_ as usize) - offset)
              .cloned()
              .unwrap_or_else(|| self.clone())
        } else {
            cache.get(&self, offset).cloned().unwrap_or_else(|| {
                let calcd = match self.as_ref() {
                    App(_, lhs, rhs) => {
                        let new_lhs = lhs.instantiate_core(offset, es.clone(), cache);
                        let new_rhs = rhs.instantiate_core(offset, es, cache);
                        mk_app(new_lhs, new_rhs)
                    },
                    | Lambda(_, dom, body) => {
                        let new_dom_ty = dom.ty.instantiate_core(offset, es.clone(), cache);
                        let new_body = body.instantiate_core(offset + 1, es, cache);
                        mk_lambda(dom.swap_ty(new_dom_ty), new_body)
                    }
                    | Pi(_, dom, body) => {
                        let new_dom_ty = dom.ty.instantiate_core(offset, es.clone(), cache);
                        let new_body = body.instantiate_core(offset + 1, es, cache);
                        mk_pi(dom.swap_ty(new_dom_ty), new_body)
                    },
                    Let(_, dom, val, body) => {
                        let new_dom_ty = dom.ty.instantiate_core(offset, es.clone(), cache);
                        let new_val = val.instantiate_core(offset, es.clone(), cache);
                        let new_body = body.instantiate_core(offset + 1, es, cache);
                        mk_let(dom.swap_ty(new_dom_ty), new_val, new_body)
                    },
                    owise => unreachable!("Illegal match result in Expr::instantiate_core {:?}\n", owise)
                };
                cache.insert(self.clone(), calcd.clone(), offset);
                calcd
            })
        }
    }
    /// This just performs variable substitution by going through
    /// the `Level` items contained in `Sort` and `Const` expressions.
    /// For all levels therein, attempts to replace `Level::Param`
    /// items with something in the `substs` mapping, which maps
    /// (Level::Param |-> Level)
    pub fn instantiate_ps(&self, substs : &Vec<(Level, Level)>) -> Expr {
        if substs.iter().any(|(l, r)| l != r) {
            match self.as_ref() {
                App(_, lhs, rhs) => {
                    let new_lhs = lhs.instantiate_ps(substs);
                    let new_rhs = rhs.instantiate_ps(substs);
                    mk_app(new_lhs, new_rhs)
                },
                Lambda(_, dom, body) => {
                    let new_domty = dom.ty.instantiate_ps(substs);
                    let new_body = body.instantiate_ps(substs);
                    mk_lambda(dom.swap_ty(new_domty), new_body)

                }
                Pi(_, dom, body) => {
                    let new_domty = dom.ty.instantiate_ps(substs);
                    let new_body = body.instantiate_ps(substs);
                    mk_pi(dom.swap_ty(new_domty), new_body)
                },

                Let(_, dom, val, body) => {
                    let new_domty = dom.ty.instantiate_ps(substs);
                    let new_val = val.instantiate_ps(substs);
                    let new_body = body.instantiate_ps(substs);
                    mk_let(dom.swap_ty(new_domty), new_val, new_body)

                },
                Local(.., of) => {
                    let new_of_ty = of.ty.instantiate_ps(substs);
                    of.swap_ty(new_of_ty).as_local()
                },
                Var(..) => self.clone(),
                Sort(_, lvl) => {
                    let instd_level = lvl.instantiate_lvl(substs);
                    mk_sort(instd_level)
                },
                Const(_, name, lvls) => {
                    let new_levels = lvls.iter()
                                         .map(|x| (x.instantiate_lvl(substs)))
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
    /// App( ... App(App(F, X_1), X_2)...  X_n)
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
    pub fn fold_apps<'q, Q>(&self, apps : Q) -> Expr 
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
    pub fn unfold_apps_refs(&self) -> (&Expr,  Vec<&Expr>) {
        let (mut _fn, mut acc) = (self, Vec::with_capacity(40));
        while let App(_, f, app) = _fn.as_ref() {
            acc.push(app);
            _fn = f;
        }
        (_fn, acc)
    }



    /// Same as unfold_apps_refs, but returns owned values instead 
    /// of references and returns the vector backwards. used a 
    /// couple of times in inductive, and once in reduction.
    pub fn unfold_apps_special(&self) -> (Expr, Vec<Expr>) {
        let (mut _fn, mut acc) = (self, Vec::with_capacity(10));
        while let App(_, f, app) = _fn.as_ref() {
            acc.push((app).clone());
            _fn = f;
        }
        acc.reverse();
        (_fn.clone(), acc)
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
        while let Pi(_, dom, body) = self.as_ref() {
            let local = dom.clone().as_local();
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
    Var    (ExprCache, u64),
    Sort   (ExprCache, Level),
    Const  (ExprCache, Name, Arc<Vec<Level>>),
    Local  (ExprCache, u64, Binding),
    App    (ExprCache, Expr,  Expr),
    Lambda (ExprCache, Binding, Expr),
    Pi     (ExprCache, Binding, Expr),
    Let    (ExprCache, Binding, Expr, Expr)
}

impl InnerExpr {
    pub fn get_digest(&self) -> u64 {
        self.get_cache().digest
    }

    pub fn get_cache(&self) -> ExprCache {
        match self {
            | Var    (info, ..) 
            | Sort   (info, ..) 
            | Const  (info, ..) 
            | Local  (info, ..) 
            | App    (info, ..) 
            | Lambda (info, ..) 
            | Pi     (info, ..) 
            | Let    (info, ..)  => *info
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
            Local(.., binding) => binding.clone(),
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
            App(_, lhs, rhs) => {
                unique_const_names_core(lhs, s, cache);
                unique_const_names_core(rhs, s, cache);
            },
            | Lambda(_, dom, body)
            | Pi(_, dom, body) => {
                unique_const_names_core(&dom.ty, s, cache);
                unique_const_names_core(&body, s, cache);

            },
            Let(_, dom, val, body) => {
                unique_const_names_core(&dom.ty, s, cache);
                unique_const_names_core(&val, s, cache);
                unique_const_names_core(&body, s, cache);
            },
            Const(_, name, _) => {
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
        App(_, lhs, rhs) => {
            univ_params_subset_core(lhs, s);
            univ_params_subset_core(rhs, s);
        },
        | Lambda(_, dom, body)
        | Pi(_, dom, body) => {
            univ_params_subset_core(&dom.ty, s);
            univ_params_subset_core(body, s);
        },
        Let(_, dom, val, body) => {
            univ_params_subset_core(&dom.ty, s);
            univ_params_subset_core(val, s);
            univ_params_subset_core(body, s);
        },
        Sort(_, lvl) => { s.extend(unique_univ_params(lvl)); },
        Const(.., lvls) => for lvl in lvls.as_ref() {
            s.extend(unique_univ_params(lvl));
        },
        _ => ()
    }
}



impl std::fmt::Debug for InnerExpr {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Var(_, idx) => {
                write!(f, "Var({})", idx)
            },
            Sort(_, lvl) => {
                write!(f, "Sort({:?})", lvl)
            },
            Const(_, name, lvls) => {
                write!(f, "Const({:?}, {:?})", name, lvls)
            },
            App(_, e1, e2) => {
                write!(f, "App({:?}, {:?})", e1, e2)
            },
            Lambda(_, dom, body) => {
                write!(f, "(λ ({:?}), {:?})", dom, body)
            },
            Pi(_, dom, body) => {
                write!(f, "(Π ({:?}), {:?})", dom, body)
            },
            Let(_, dom, val, body) => {
                write!(f, "let {:?} := {:?} in {:?}", dom, val, body)
            },
            Local(_, serial, of) => {
                let truncated = serial.to_string().chars().take(6).collect::<String>();
                write!(f, "Local(serial : {:?}, of : {:?}", truncated, of)
            }
        }
    }
}
