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

/// ハッシュを計算する際に用いられる識別子。
const LAMBDA_HASH   : u64 = 402653189;
/// ハッシュを計算する際に用いられる識別子。
const PI_HASH       : u64 = 1610612741;
/// これと PROP_CACHE って Prop を定数として使用出来るための物です。
const PROP_HASH     : u64 = 786433;
const PROP_CACHE    : ExprCache = ExprCache { digest : PROP_HASH, 
                                                  var_bound : 0, 
                                                  has_locals : false };

/// 未使用の Local 名前を作るために、一で増やすカウンターを使ってるんだけです。
/// Atomic であるので、複数のスレッドから呼べます。Local という Expression
/// の類は `clone()` で作られたら、元のやつと同じ名前を持つ必要がありますが、
/// コンストラクターから作られたやつはどの事情でもユニーク名前を持つことが必要です。
pub static LOCAL_SERIAL : AtomicU64 = AtomicU64::new(0);




/// 束縛記号 (binder) の様々な種類を表す型です。マッピングは以下：
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


/// この型は Pi・Lambda・Let （束縛を表す表現）の束縛を指定するための型です。
/// 例えば、`(λ x : T, E)` を見れば、pp_name って `x`, ty って `T と
/// なります。
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

/// これは木構造を効率的に使えるための `InnerExpr` を包むものです。
/// `Expr` の木を対処する関数はこの型と対応します。
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Expr(Arc<InnerExpr>);

impl std::fmt::Debug for Expr {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.as_ref())
    }
}



/// Prop と対応する特別な Expr::Sort のコンストラクターです。
pub fn mk_prop() -> Expr {
    Sort(PROP_CACHE, mk_zero()).into() // InnerLevel -> Level
}


/// ド・ブラウン・インデックスで表される変数です。
pub fn mk_var(idx : u64) -> Expr {
    let digest = hash64(&(idx));
    Var(ExprCache::mk(digest, idx as u16 + 1, false), idx).into() // InnerLevel -> Level
}

/// 木構造の二項ノードを適用で作るコンストラクターです。
pub fn mk_app(lhs : Expr, rhs : Expr) -> Expr {
    let digest = hash64(&(lhs.get_digest(), rhs.get_digest()));
    let var_bound = lhs.var_bound().max(rhs.var_bound());
    let has_locals = lhs.has_locals() || rhs.has_locals();
    App(ExprCache::mk(digest, var_bound, has_locals), lhs, rhs).into() // InnerLevel -> Level 
}

/// これはユニバース・ソート・LevelをExprとして表すコンストラクターだけですね。
pub fn mk_sort(level : Level) -> Expr {
    let digest = hash64(&level);
    Sort(ExprCache::mk(digest, 0, false), level).into() // InnerLevel -> Level 
}

/// 型検査装置の文脈で、Const（定数)ってことは既に認められているEnv・環境にある物への参照です。
pub fn mk_const(name : impl Into<Name>, levels : impl Into<Arc<Vec<Level>>>) -> Expr {
    let name = name.into();
    let levels = levels.into();
    let digest = hash64(&(&name, &levels));
    Const(ExprCache::mk(digest, 0, false), name, levels).into()
}

/// ラムダ抽象化のコンストラクター
pub fn mk_lambda(domain : Binding, body: Expr) -> Expr {
    let digest = hash64(&(LAMBDA_HASH, &domain, body.get_digest()));
    let var_bound = max(domain.ty.var_bound(), 
                        safe_minus_one(body.var_bound()));
    let has_locals = domain.ty.has_locals() || body.has_locals();
    Lambda(ExprCache::mk(digest, var_bound, has_locals), domain, body).into() // InnerLevel -> Level
}

/// Pi(依存関数)を作るコンストラクター
pub fn mk_pi(domain : Binding, body: Expr) -> Expr {
    let digest = hash64(&(PI_HASH, &domain, body.get_digest()));
    let var_bound = max(domain.ty.var_bound(),
                        safe_minus_one(body.var_bound()));
    let has_locals = domain.ty.has_locals() || body.has_locals();
    Pi(ExprCache::mk(digest, var_bound, has_locals), domain, body).into() // InnerLevel -> Level
}

/// これは let 抽象物を作るコンストラクターです。例えば `let (x : nat) := 5 in 2 * x`
pub fn mk_let(domain : Binding, val : Expr, body : Expr) -> Expr {
    let digest = hash64(&(&domain, val.get_digest(), body.get_digest()));
    let var_bound = max3(domain.ty.var_bound(),
                         val.var_bound(),
                         safe_minus_one(body.var_bound()));
    let has_locals = domain.ty.has_locals() || body.has_locals() || val.has_locals();
    Let(ExprCache::mk(digest, var_bound, has_locals), domain, val, body).into() // InnerLevel -> Level
}

/// Local っていうのは自由変数を表す物です。型検査装置で、全てのLocalは自分の型を
/// 持っているもので、判明するために唯一の通し番号を `serial` として持っています。
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

    // !! 部分的関数 !!
    pub fn lc_binding(&self) -> &Binding {
        match self.as_ref() {
            Local(.., binding) => binding,
            owise => errors::err_lc_binding(line!(), owise)
        }
    }

    // !! 部分的関数 !!
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

    /// !! 部分的関数 !!  
    /// 与えられた変数が Local なら、その Local の一貫番号が返されます。
    /// Local ではなければ、エラーを起こしてしまう。
    pub fn get_serial(&self) -> u64 {
        match self.as_ref() {
            Local(_, serial, _) => *serial,
            owise => errors::err_get_serial(line!(), owise)
        }
    }

    /// これは２つの表現を矢印で組んでくれる原始的なものです。`e1` と `e2` という表現
    /// があったら、`Π (e1), e2` というように `e1 → e2` が作られて返されたんです。
    pub fn mk_arrow(&self, other : &Expr) -> Expr {
        let binding = Binding::mk(mk_anon(), self.clone(), BinderStyle::Default);
        mk_pi(binding, other.clone())
    }


    /// ある Expr を巡回して、出来る限りに Local を Var で交換してみながら、既に見た項・subtree
    /// を二回評価しないために結果をカッシュする仕事です。交換が行われるケースは `Local` のケースだ。
    /// その時に、`lcs` という連続を `Local` と等しくある項を探して、見つけられた場合、その項の
    /// インデックス `n` をとって、Local を新たに作った Var(n) と交換する。`offset` 値は、
    /// ある束縛子（Pi, Lambda, Let バインダー）の範囲から、他のやつの範囲へ移動する時に
    /// 増やされた数字です。
    pub fn abstract_<'e>(&self, lcs : impl Iterator<Item = &'e Expr>) -> Expr {
        if !self.has_locals() {
            return self.clone() 
        }

        let mut cache = OffsetCache::new();
        // あいにくここにもcollectを使ってます。
        let lcs = lcs.collect::<Vec<&Expr>>();

        self.abstract_core(0usize, &lcs, &mut cache)
    }

    fn abstract_core(&self, offset : usize, locals : &Vec<&Expr>, cache : &mut OffsetCache) -> Expr {
        if !self.has_locals() {
            return self.clone()
        } else if let Some(cached) = cache.get(&self, offset) {
            return cached.clone()
        } else if let Local(_, serial, _) = self.as_ref() {
            locals.iter()
            .position(|lc| lc.get_serial() == *serial)
            .map_or_else(|| self.clone(), |position| {
                mk_var((position + offset) as u64)
            })
        } else {

            let cache_key = self.clone();

            let result = match self.as_ref() {
                App(_, lhs, rhs) => {
                    let new_lhs = lhs.abstract_core(offset, locals, cache);
                    let new_rhs = rhs.abstract_core(offset, locals, cache);
                    mk_app(new_lhs, new_rhs)
                },
                Lambda(_, dom, body) => {
                    let new_domty = dom.ty.abstract_core(offset, locals, cache);
                    let new_body = body.abstract_core(offset + 1, locals, cache);
                    mk_lambda(dom.swap_ty(new_domty), new_body)
                }
                Pi(_, dom, body) => {
                    let new_domty = dom.ty.abstract_core(offset, locals, cache);
                    let new_body = body.abstract_core(offset + 1, locals, cache);
                    mk_pi(dom.swap_ty(new_domty), new_body)
                },
                Let(_, dom, val, body) => {
                    let new_domty = dom.ty.abstract_core(offset, locals, cache);
                    let new_val = val.abstract_core(offset, locals, cache);
                    let new_body = body.abstract_core(offset + 1, locals, cache);
                    mk_let(dom.swap_ty(new_domty), new_val, new_body)
                },
                owise => unreachable!("Illegal match item in Expr::abstract_core {:?}\n", owise)
            };

            cache.insert(cache_key, result.clone(), offset);
            result
        }
    }


    /// この関数は abstract と似たような形をしている巡回です。しかし、今回は Var を他の Expr
    /// と交換してみたいんです。ここにもカッシュ・offset が用いられます。今回の特別ケースは
    /// Var 対の行動です。Var(n) の `n` がインバウンドってことを確認した後、`es[n]`
    /// を抜き出さずにコピーして、コピーした物を元の Var(n) と交換するように進んでいきます。
    pub fn instantiate<'e>(&self, es : impl Iterator<Item = &'e Expr>) -> Expr {
        // ここの `collect()` は確かに必要ではないんですが、自分でテストした時に core 
        // 中イテレータを何回もクローンしていくことのパフォーマンスはだいたい同じだったんです。
        let es = es.collect::<Vec<&Expr>>();

        let mut cache = OffsetCache::new();
        self.instantiate_core(0usize, &es, &mut cache)
    } 

    fn instantiate_core(&self, offset : usize, es : &Vec<&Expr>, cache : &mut OffsetCache) -> Self {
        if self.var_bound() as usize <= offset {
            return self.clone()
        } else if let Some(cached) = cache.get(&self, offset) {
            return cached.clone()
        } else if let Var(_, idx_) = self.as_ref() {
            let idx = *idx_ as usize;

            if offset <= idx && idx < (offset + es.len()) {
                es[idx - offset].clone()
            } else {
                return self.clone()
            }
        } else {

            let cache_key = self.clone();

            let result = match self.as_ref()  {
                App(_, lhs, rhs) => {
                    let new_lhs = lhs.instantiate_core(offset, es, cache);
                    let new_rhs = rhs.instantiate_core(offset, es, cache);
                    mk_app(new_lhs, new_rhs)
                },
                | Lambda(_, dom, body) => {
                    let new_dom_ty = dom.ty.instantiate_core(offset, es, cache);
                    let new_body = body.instantiate_core(offset + 1, es, cache);
                    mk_lambda(dom.swap_ty(new_dom_ty), new_body)
                }
                | Pi(_, dom, body) => {
                    let new_dom_ty = dom.ty.instantiate_core(offset, es, cache);
                    let new_body = body.instantiate_core(offset + 1, es, cache);
                    mk_pi(dom.swap_ty(new_dom_ty), new_body)
                },
                Let(_, dom, val, body) => {
                    let new_dom_ty = dom.ty.instantiate_core(offset, es, cache);
                    let new_val = val.instantiate_core(offset, es, cache);
                    let new_body = body.instantiate_core(offset + 1, es, cache);
                    mk_let(dom.swap_ty(new_dom_ty), new_val, new_body)
                },
                owise => unreachable!("Illegal match result in Expr::instantiate_core {:?}\n", owise)
            };

            cache.insert(cache_key, result.clone(), offset);

            result

        }
    }

    /// ある `Expr` の `Sort` と `Const` にあるLevel::paramを 
    /// 交換してみる関数です。`substs` という鍵値マッピングを使って交換してみる
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


    /// 与えられた`F : Expr` と [X_1, X_2, ... X_n] : List<Expr> から、 App
    /// コンストラクターを以下の形を作るために繰り返し適用する関数。
    ///
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
    

    /// 既に痛苦られた木構造から、背骨の右から左へ unfold していく関数です。
    /// 以下の図のように働きます。
    ///
    ///```pseudo
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



    /// unfold_aps_refs とにたような物ですが、返された subtree を持っている
    /// ベクターは逆で、返された物は所有されている値です。
    pub fn unfold_apps_special(&self) -> (Expr, Vec<Expr>) {
        let (mut _fn, mut acc) = (self, Vec::with_capacity(10));
        while let App(_, f, app) = _fn.as_ref() {
            acc.push((app).clone());
            _fn = f;
        }
        acc.reverse();
        (_fn.clone(), acc)
    }

    /// `E` と `L` から、`L` が `Local` であるってことを確認して、Pi で
    /// 以下のように作って
    ///```pseudo
    /// let E = E.abstract(L)
    /// return (Π (L) (E'))
    ///```
    pub fn apply_pi(&self, domain : &Expr) -> Expr {
        assert!(domain.is_local());
        let abstracted = self.clone().abstract_(Some(domain).into_iter());
        mk_pi(Binding::from(domain), abstracted)
    }


    /// `Expr::Local` のリストと `E:Expr` から、fold_right あるいは空きなメソッド
    /// を使って、Pi コンストラクターで以下の形を作る
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
    ///                  /   \
    ///                L_n    E
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

    /// これは連続的な `Pi` コンストラクターの適用を分解してくれる物です。例示を見れば
    /// 分かると思います。この関数は繰り返し適用で用いられるはずだから、累算器は可変 Vector です。　
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

    /// `E` と `L` から、`L` が `Local` であるってことを確認して、Lambda で
    /// 以下のように作って:
    ///```pseudo
    ///  let E' = E.abstract(L)
    ///  return (λ L, E')
    ///```
    pub fn apply_lambda(&self, domain : &Expr) -> Expr {
        assert!(domain.is_local());
        let abstracted = self.clone().abstract_(Some(domain).into_iter());
        mk_lambda(Binding::from(domain), abstracted)
    }


    /// 与えられた List<Expr>、例えば [L_1, L_2, ... L_n] と `E : Expr` というボディー
    /// を撮って、Lambda のコンストラクターを繰り返し適用して、以下のような形を作る関数。
    /// fold_right のような動きです。
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



/// `Expr` のハッシュ・束縛されている変数の個数・`Local`を保持するかどうかという　
/// 情報をカッシュするものです。ここでの重要なところは、ハッシュマップがこれを鍵として
/// 検索・入っている時に、そのカッシュされる値しか見ないでもいいんだ。このハッシュを
/// カッシュしないと、ハッシュマップで検索する度に、`Expr` の木構造が全体再帰的に
/// ハッシュされて、パフォーマンスが非常に下がってしまいます。
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


// !! 部分的関数 !! //
impl From<&Expr> for Binding {
    fn from(e : &Expr) -> Binding {
        match e.as_ref() {
            Local(.., binding) => binding.clone(),
            owise => errors::err_binding_lc(line!(), owise),
        }
    }
}



/// ((Expr × int) |-> Expr) のマッピングです。
/// 「 (Expr A at offset B) って (Expr C) へマップされている」
/// ってことを表す物です。この構造を作る方法が勿論いくつありますが
/// ラストのタプルを支配する規則のせいで、我が作って測ったやつらから、
/// こっちが一番早かったです。
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

/// 与えられた `E : Expr` にある全ての `Name` を `S : Set<Name>` に
/// 集めてくれる関数です。これは `Definition` をコンパイルする内にしか
/// 使用されなくて、その `Definition` にある項の height・高さを環境で
/// 検索するためにあります。tc::def_height でその「高さ」について詳しくよめます。
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

/// 与えられた `E : Expr` と `S_X : Set<Level>` から、Eから全ての
/// `Level::Param` 要素を新たな `S_E : Set` に集めて、`S_E ⊆ S_X` 
/// を確かめる関数です。これは `Declaration` の `type` というフィールドを
/// 検査する事情だけで使用されます。`Declaration` のユニバース引数は
/// ちゃんと宣言されているかどうかを確かめてくれます。
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
