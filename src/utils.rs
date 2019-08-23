use std::collections::VecDeque;
use std::sync::Arc;

use hashbrown::HashMap;
use parking_lot::RwLock;

use crate::expr::Expr;
use crate::env::{ Modification, CompiledModification };
use crate::pretty::components::Notation;

use Either::*;
use ShortCircuit::*;

///「仕事終わりました」ってことをワーカースレッドに伝えるためのメッセージです。
/// 仕事を待っている状態もあるから、これは別のものとして定義されたんです　。
pub const END_MSG_ADD : QueueMsg<Modification> = Right(EndMsg(()));
pub const END_MSG_NOTATION : QueueMsg<Notation> = Right(EndMsg(()));
pub const END_MSG_CHK : QueueMsg<CompiledModification> = Right(EndMsg(()));


pub fn foldr<A, B, I>(f : impl Fn(A, B) -> B, i : I, init : B) -> B 
where I : IntoIterator<Item = A>,
      I :: IntoIter : DoubleEndedIterator {
    i.into_iter().rev().fold(init, |acc, next| f(next, acc))
}

/// 構文糖衣の量を減らすためのマクロだけです。いくつかの自立の物をイテレータに
/// 組んでくれるマクロだけだ。
#[macro_export]
macro_rules! chain {
    ( $( $e:expr),* ) => {
        {
            let acc = None.into_iter();

            $(
                let acc = acc.chain(Some($e).into_iter());
            )*

            acc
        }

    };
}

/// 複数の連続(vecなど)を単一のイテレータに組んでくれるマクロだ。
#[macro_export]
macro_rules! seq {
    ( $( $e:expr),* ) => {
        {
            let mut buf = Vec::new();

            $(
                for elem in $e.into_iter() {
                    buf.push(elem.to_owned());
                }
            )*
            buf
        }

    };
}


pub fn safe_minus_one(n : u16) -> u16 {
    if n == 0 {
        n
    } else {
        n - 1
    }
}

pub fn max3(n1 : u16, n2 : u16, n3 : u16) -> u16 {
    n1.max(n2).max(n3)
}



/// これは TypeChecker がよく使用するものです。def_eq ってかなり
/// 重い計算になれるので、２つの Expr が可能な限り縮小・推論される前に
/// 等しさが明るくなった場合、そこで停止して結果を返していきたいんです。
/// EqShort って「もう等しい」、NeqShortって「もう等しくない」、
/// Unknown って「まだ分からないから続けよう」ってことを伝えるメッセージです。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortCircuit {
    EqShort,
    NeqShort,
    Unknown,
}

impl ShortCircuit {
}

pub fn ss_forall(mut seq : impl Iterator<Item = ShortCircuit>) -> ShortCircuit {
    match seq.all(|elem| elem == EqShort) {
        true => EqShort,
        false => NeqShort
    }
}

pub fn ss_and(ss1 : ShortCircuit, ss2 : ShortCircuit) -> ShortCircuit {
    match ss1 {
        EqShort => ss2,
        NeqShort => NeqShort,
        Unknown => Unknown
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

/// ハッシュマップに基づくカッシュです。任意の `e1`と`e2` Expr が与えられたら、
/// TypeChecker がそのペアをみたことがあったら、定義的等値性比較の計算
/// された結果を返してくれる。HashMap<(Expr, Expr), ShortCircuit> の方が
/// 直感的だと思いますが、そうすれば rust は参照・ポインターだけで鍵を
/// 検索することができなくなってしまいます。
#[derive(Clone)]
pub struct EqCache {
    inner : HashMap<Expr, Vec<(Expr, ShortCircuit)>>
}

impl EqCache {
    pub fn new() -> Self {
        EqCache {
            inner : HashMap::with_capacity(500)
        }
    }

    pub fn get(&self, e1 : &Expr, e2 : &Expr) -> Option<ShortCircuit> {
        match self.inner.get(e1) {
            None => match self.inner.get(e2) {
                Some(v) => v.iter().find(|(a, _)| a == e1).map(|(_, b)| b.clone()),
                None => return None
            },
            Some(v) => {
                v.iter().find(|(a, _)| a == e2).map(|(_, b)| b.clone())
            }
        }
    }

    pub fn insert(&mut self, e : Expr, ee : Expr, val : ShortCircuit) {
        match self.inner.get_mut(&e) {
            Some(v) => {
                v.push((ee, val));
            },
            None => {
                let mut v = Vec::with_capacity(10);
                v.push((ee, val));
                self.inner.insert(e, v);
            }
        }
    }
}



/// Queue backed by a thread-safe VecDeque. 
#[derive(Debug, Clone)]
pub struct RwQueue<T>(Arc<RwLock<VecDeque<T>>>);

impl<T> RwQueue<T> {
    pub fn with_capacity(n : usize) -> Self {
        let inner = VecDeque::with_capacity(n);
        RwQueue(Arc::new(RwLock::new(inner)))
    }

    pub fn push(&self, t : T) {
        match self {
            RwQueue(inner) => inner.write().push_back(t)
        }
    }

    pub fn pop(&self) -> Option<T> {
        match self {
            RwQueue(inner) => inner.write().pop_front()
        }
    }
}

#[derive(Debug, Clone)]
pub struct EndMsg(());
impl EndMsg {
    pub fn mk() -> Self { EndMsg(()) }
}

pub type QueueMsg<T> = Either<T, EndMsg>;


pub type ModQueue = RwQueue<QueueMsg<Modification>>;
pub type CompiledQueue = RwQueue<QueueMsg<CompiledModification>>;
