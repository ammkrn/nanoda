use std::sync::Arc;
use hashbrown::HashSet;

use crate::name::Name;
use crate::errors;

use InnerLevel::*;

/// `Level` と `InnerLevel` は Lean のソート・ユニバース項を表す型です。
/// 構成は木構造で、ここにも `Level` は `InnerLevel` を包む物だけです。
/// `Zero` と `Param` はいつも葉・終端物であります。ゼロはただのゼロ
/// を表して、`Param` って名前を包むユニバース変数を表す物です。(例えば、Lean
/// でよく見る `Sort 0`)。
/// `Succ` ってユニバースのレベルを一で増やして、後者レベルは一つだけの子要素です。
/// `Max` っていつも子供ノードを２つ持っているもので、２つのレベルからの最大のやつ
/// を表すものです。`Param` のせいで、２つのレベルからどっちが大きいかってことを
/// 計算できない時もあります。
/// `IMax` って`Max` と似たようなものですが、`IMax` の左側が `0` になったら、その
/// ノードの結果は全体 `0` になります。この行動は `Prop` をちゃんと処理するため
/// の行為です。`Theorem Proving in Lean (英)` でそのことについて詳しく読めます。
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Level(Arc<InnerLevel>);

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum InnerLevel {
    Zero,
    Succ (Level),
    Max  (Level, Level),
    IMax (Level, Level),
    Param(Name),
}

pub fn mk_zero() -> Level {
    Level(Arc::new(InnerLevel::Zero))
}

pub fn mk_max(lhs : Level, rhs : Level) -> Level {
    Level(Arc::new(Max(lhs, rhs)))
}

pub fn mk_imax(lhs : Level, rhs : Level) -> Level {
    Level(Arc::new(IMax(lhs, rhs)))
}

pub fn mk_imax_refs(lhs : &Level, rhs : &Level) -> Level {
    Level(Arc::new(IMax(lhs.clone(), rhs.clone())))
}

pub fn mk_param(n : impl Into<Name>) -> Level {
    Level(Arc::new(Param(n.into())))
}

pub fn mk_succ(l : Level) -> Level {
    Level(Arc::new(Succ(l)))
}

impl Level {
    pub fn get_param_name(&self) -> &Name {
        match self.as_ref() {
            Param(n) => n,
            owise    => errors::err_param_name(line!(), owise)
        }
    }

    pub fn is_param(&self) -> bool {
        match self.as_ref() {
            Param(..) => true,
            _         => false
        }
    }

    pub fn is_any_max(&self) -> bool {
        match self.as_ref() {
            Max(..) | IMax(..) => true,
            _                  => false
        }
    }

    /// ２つのレベルを特別のように組む。これは `simplify`中に使用され。
    pub fn combining(&self, other : &Level) -> Self {
        match (self.as_ref(), other.as_ref()) {
            (Zero, _)              => other.clone(),
            (_, Zero)              => self.clone(),
            (Succ(lhs), Succ(rhs)) => mk_succ(lhs.combining(rhs)),
            _                      => mk_max(self.clone(), other.clone())

        }
    }

    /// レベルを軽く簡単にしてくれる手続きです。主に`IMax`を感嘆するための物です。
    pub fn simplify(&self) -> Level {
        match self.as_ref() {
            Zero | Param(..) => self.clone(),
            Succ(lvl)        => mk_succ(lvl.simplify()),
            Max(a, b)        => mk_max(a.simplify(), b.simplify()),
            IMax(a, b)       => {
                let b_prime = b.simplify();
                match b_prime.as_ref() {
                    Zero        => mk_zero(), 
                    Succ(..)    => a.simplify().combining(&b_prime),
                    _ => mk_imax(a.simplify(), b_prime)
                }
            }
        }
    }

    /// 任意のレベル `L` と `Level::Param |-> `Level` のマッピング `M` から、
    /// `L` を巡回しながら各ノード `n` に対して :
    /// IF (`n` is `Param(..)`) ∧ (mapping `n |-> x` ∈ M) 
    /// THEN `n` を `x` と交換して
    pub fn instantiate_lvl(&self, substs : &Vec<(Level, Level)>) -> Level {
        match self.as_ref() {
            Zero => mk_zero(),
            Succ(inner) => mk_succ(inner.instantiate_lvl(substs)),
            Max(a, b) => {
                let a_prime = a.instantiate_lvl(substs);
                let b_prime = b.instantiate_lvl(substs);
                mk_max(a_prime, b_prime)
            },
            IMax(a, b) => {
                let a_prime = a.instantiate_lvl(substs);
                let b_prime = b.instantiate_lvl(substs);
                mk_imax(a_prime, b_prime)
            },
            Param(..) => {
                substs.iter()
                      .find(|(l, _)| l == self)
                      .map(|(_, r)| r.clone())
                      .unwrap_or_else(|| self.clone())
            }
        }
    }



    /// この関数は「一方側がImaxで、そのIMaxはParam子供を持っている」ケースを処理するため、
    /// is_leq_core中に用いられます。`IMax`の左側がゼロ場合の特別な行為を考慮して、
    /// lhs <= rhs って本当かどうかって言える前、ケース分析ををしなければなりません。
    /// 見るケースは :
    /// 1. `P` はいつか `0` になる
    /// 2. `P` はいつか `0` ではない要素になる
    ///
    /// だから、２つの置換を作って、両ケースで lhs が本当かどうかを調査します。
    ///```pseudo
    /// let (lhs', rhs') = (lhs[Zero/P], rhs[Zero/P])
    /// let (lhs'', rhs'') = (lhs[Succ(P)/P], rhs[Succ(P)/P])
    /// return (lhs' ≤ rhs') && (lhs'' ≤ rhs'')
    pub fn ensure_imax_leq(&self, lhs : &Level, rhs : &Level, diff : i32) -> bool {
        assert!(self.is_param());

        let zero_map =  vec![(self.clone(), mk_zero())];
        let nonzero_map = vec![(self.clone(), mk_succ(self.clone()))];


        let closure = |subst : &Vec<(Level, Level)>, left : &Level, right : &Level| {
            let left_prime  = left.instantiate_lvl(subst).simplify();
            let right_prime = right.instantiate_lvl(subst).simplify();
            left_prime.leq_core(&right_prime, diff)
        };

        closure(&zero_map, lhs, rhs)
        &&
        closure(&nonzero_map, lhs, rhs)
    }

    /// ParamとIMaxからの複雑性を念頭に置きながら、２つの与えられた`L1` と `L2` 
    /// `Level` が与えられた場合、大きいなケース分析で `L1 ≤ L2` が真か偽か
    /// ってことを検査する。この `≤` って Lean の型システムの `≤` だ、Rustの
    /// ではない。`diff` っていう値は再帰呼出しに、前回の影響を追ってくれる値だけです。
    pub fn leq_core(&self, other : &Level, diff : i32) -> bool {

        match (self.as_ref(), other.as_ref()) {
            (Zero, _) if diff >= 0             => true,
            (_, Zero) if diff < 0              => false,
            (Param(a), Param(x))               => a == x && diff >= 0,
            (Param(..), Zero)                  => false,
            (Zero, Param(..))                  => diff >= 0,

            (Succ(s), _)                       => s.leq_core(other, diff - 1),
            (_, Succ(s))                       => self.leq_core(s, diff + 1),

            (Max(a, b), _)                     => a.leq_core(other, diff) 
                                               && b.leq_core(other, diff),

            (Param(..), Max(x, y))             => self.leq_core(x, diff)
                                               || self.leq_core(y, diff),

            (Zero, Max(x, y))                  => self.leq_core(x, diff)
                                               || self.leq_core(y, diff),

            (IMax(a, b), IMax(x, y)) if a == x 
                                     && b == y => true,

            (IMax(.., b), _) if b.is_param()   => b.ensure_imax_leq(self, other, diff),

            (_, IMax(.., y)) if y.is_param()   => y.ensure_imax_leq(self, other, diff),

            (IMax(a, b), _) if b.is_any_max()  => match b.as_ref() {
                IMax(x, y) => {
                    let new_max = mk_max(mk_imax_refs(a, y), 
                                         mk_imax_refs(x, y));
                    Level::leq_core(&new_max, other, diff)
                },

                Max(x, y) => {
                    let new_max = mk_max(mk_imax_refs(a, x), 
                                         mk_imax_refs(a, y)).simplify();
                    Level::leq_core(&new_max, other, diff)

                },
                _ => unreachable!(),
            }

            (_, IMax(x, y)) if y.is_any_max()  => match y.as_ref() {
                IMax(j, k) => {
                    let new_max = mk_max(mk_imax_refs(x, k),
                                         mk_imax_refs(j, k));
                    self.leq_core(&new_max, diff)
                },
                Max(j, k) => {
                    let new_max = mk_max(mk_imax_refs(x, j), 
                                         mk_imax_refs(x, k)).simplify();
                    self.leq_core(&new_max, diff)
                },
                _ => unreachable!(),
            }
            _ => unreachable!()
        }
    }
    
    /// 外から呼ぶべきの `L1 <= L2` を調査してくれる関数です。
    pub fn leq(&self, other : &Level) -> bool {
        self.simplify().leq_core(&other.simplify(), 0)
    }

    /// leq と反対称性を使って、２つのレベルが等しいかどうかを調査する関数
    ///```pseudo
    ///(x ≤ y ∧ y ≤ x) → x = y
    ///```
    pub fn eq_by_antisymm(&self, other : &Level) -> bool {
        let l1 = self.simplify();
        let l2 = other.simplify();
        
        l1.leq_core(&l2, 0) && l2.leq_core(&l1, 0)
    }

    /// `Level` が `Zero` かどうかを確かめるための関数です。
    ///```pseudo
    /// `∀ (L : Level), (L ≤ Zero) ∧ (¬ ∃ L' : Level, L' < Zero) → L = Zero`
    ///```
    pub fn is_zero(&self) -> bool {
        self.leq(&mk_zero())
    }

    /// `Level` が Zero ではないことを確かめる関数だえす。　
    ///```pseudo
    /// ∀ (L : Level), S (Zero) ≤ L → L ≠ 0
    ///```
    pub fn is_nonzero(&self) -> bool {
        mk_succ(mk_zero()).leq(self)
    }

    pub fn maybe_zero(&self) -> bool {
        !self.is_nonzero()
    }

    pub fn maybe_nonzero(&self) -> bool {
        !self.is_zero()
    }

    pub fn to_offset(&self) -> (usize, &Level) {
        let (mut succs, mut inner) = (0usize, self);

        while let Succ(x) = inner.as_ref() {
            succs += 1;
            inner = x;
        }

        return (succs, inner)
    }
}


pub fn unique_univ_params<'l, 's>(lvl : &'l Level) -> HashSet<&'l Level> {
    let mut acc = HashSet::with_capacity(40);
    unique_univ_params_core(lvl, &mut acc);
    acc
}

pub fn unique_univ_params_core<'l, 's>(lvl : &'l Level, acc : &'s mut HashSet<&'l Level>) {
    match lvl.as_ref() {
        Zero             => (),
        Succ(lvl)        => unique_univ_params_core(lvl, acc),
        | Max(lhs, rhs)
        | IMax(lhs, rhs) => {
            unique_univ_params_core(lhs, acc);
            unique_univ_params_core(rhs, acc);
        },
        Param(..)        => { 
            acc.insert(lvl); 
        }
    }
}



impl std::convert::AsRef<InnerLevel> for Level {
    fn as_ref(&self) -> &InnerLevel {
        match self {
            Level(x) => x.as_ref()
        }
    }
}

impl From<Arc<InnerLevel>> for Level {
    fn from(x : Arc<InnerLevel>) -> Level {
        Level(x)
    }
}

impl From<InnerLevel> for Level {
    fn from(x : InnerLevel) -> Level {
        Level(Arc::new(x))
    }
}

impl From<&str> for Level {
    fn from(s : &str) -> Level {
        mk_param(s)
    }
}

impl std::fmt::Debug for Level {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.as_ref())
    }
}

impl std::fmt::Debug for InnerLevel {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Zero           => write!(f, "Zero"),
            Succ(_)        => {
                let outer = Level::from(self.clone());
                let (succs, inner) = outer.to_offset();
                let s = if inner.is_zero() {
                    format!("Sort {}", succs)
                } else {
                    format!("{} + {:?}", succs, inner)
                };

                write!(f, "{}", s)
            }
            Max(lhs, rhs)  => write!(f, "Max({:?}, {:?})", lhs, rhs),
            IMax(lhs, rhs) => write!(f, "IMax({:?}, {:?})", lhs, rhs),
            Param(n)       => write!(f, "Param({:?})", n)
        }
    }
}
