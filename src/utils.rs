use std::collections::VecDeque;
use std::sync::Arc;

use hashbrown::HashMap;
use parking_lot::RwLock;

use crate::expr::Expr;
use crate::env::{ Modification, CompiledModification };
use crate::pretty::components::Notation;

use Either::*;
use ShortCircuit::*;

/// Items used to communicate with the threads looping through
/// the queues that hold the typechecker's work. Needed in order
/// to discriminate between the case of "the queue doesn't have
/// any work for you right now" and "the job this queue was needed
/// for is complete"
pub const END_MSG_ADD : QueueMsg<Modification> = Right(());
pub const END_MSG_NOTATION : QueueMsg<Notation> = Right(());
pub const END_MSG_CHK : QueueMsg<CompiledModification> = Right(());


pub fn foldr<A, B, I>(f : impl Fn(A, B) -> B, i : I, init : B) -> B 
where I : IntoIterator<Item = A>,
      I :: IntoIter : DoubleEndedIterator {
    i.into_iter().rev().fold(init, |acc, next| f(next, acc))
}

/// Used to try and ease some of the pain of working with long sequences.
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

/// Used to try and ease some of the pain of working with long sequences.
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



/// Used frequently the typechecker; we want to be able to communicate
/// the following states to the observer of some return value :
/// 1. These two expressions can be further reduced/inferred, but I can
///    already tell you they're definitionally equal, so don't bother.
/// 2. These two expressions can be further reduced/inferred, but I can
///    already tell you they're NOT definitionally equal.
/// 3. These need more work before I can tell whether or not they're equal.
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

/// HashMap based cache; given two expressions, will tell you whether
/// the TypeChecker has seen this particular pair before, and if so,
/// what the result of a definitional equality comparison was. 
/// HashMap<(Expr, Expr), ShortCircuit> would be more intuitive, but
/// would require cloning both keys on every lookup due to the memory
/// layout of tuples.
#[derive(Clone)]
pub struct EqCache {
    inner : HashMap<Expr, Vec<(Expr, ShortCircuit)>>
}

impl EqCache {
    pub fn with_capacity(n : usize) -> Self {
        EqCache {
            inner : HashMap::with_capacity(n)
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

pub type QueueMsg<T> = Either<T, ()>;

pub type ModQueue = RwQueue<QueueMsg<Modification>>;
pub type CompiledQueue = RwQueue<QueueMsg<CompiledModification>>;
