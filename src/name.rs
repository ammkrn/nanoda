use std::sync::Arc;

use hashbrown::HashSet;

use InnerName::*;

/// `Name` は `InnerName` を包む物だけです。基本的に、Lean の「階層的名前」
/// を表すものであります。「階層的」ってことは `.` 記号で分裂される入れ子構造の
/// 名前ってことだけです。この型は逆の方向へ拡張する List と似たようなものです。
/// List と同様に、`Name`はいつも `Anon` から始めて、Str と Num は Cons
/// のようなコンストラクターです。Str と Num は同じ名前の構成で使用可能です。
#[derive(Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct Name(Arc<InnerName>);

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub enum InnerName {
    Anon,
    Str(Name, String),
    Num(Name, u64),
}

pub fn mk_anon() -> Name {
    Name(Arc::new(InnerName::Anon))
}

impl Name {

    pub fn is_anon(&self) -> bool {
        match self {
            Name(x) => match x.as_ref() {
                Anon => true,
                _ => false
            }
        }
    }


    /// 与えられた Name を文字列で拡張する。例えば、`nat` => `nat.rec`
    pub fn extend_str(&self, hd : &str) -> Self {
        Str(self.clone(), String::from(hd)).into()    // InnerName -> Name
    }

    /// 与えられた Name を数字で拡張する。例えば `prod` => `prod.3`
    pub fn extend_num(&self, hd : u64) -> Self {
        Num(self.clone(), hd).into()                  // InnerName -> Name
    }


    /// おすすめの名前 `n` と 既に使用されている名前の集合 `S` から、
    /// `n` が既に使用されているなら、`n`に位置で増やしている数字を追加して、
    /// 使用されていない名前作れる前に繰り返す関数。この関数は iterator
    /// の遅延性に頼っています。
    pub fn fresh_name(suggested : &str, forbidden : HashSet<&Name>) -> Self {
        let base = Name::from(suggested);
        if !forbidden.contains(&base) {
            return base
        }
        (0u64..).into_iter()
                .map(|n| base.extend_num(n))
                .filter(|candidate| !forbidden.contains(candidate))
                .next()
                .unwrap()

    }

}




impl std::convert::AsRef<InnerName> for Name {
    fn as_ref(&self) -> &InnerName {
        match self {
            Name(x) => x.as_ref()
        }
    }
}

impl From<Arc<InnerName>> for Name {
    fn from(x : Arc<InnerName>) -> Name {
        Name(x)
    }
}

impl From<InnerName> for Name {
    fn from(x : InnerName) -> Name {
        Name(Arc::new(x))
    }
}

impl From<&str> for Name {
    fn from(s : &str) -> Name {
        mk_anon().extend_str(s)
    }
}


/// `Name`　は左から右へ、各成分が `.` で分裂されているように印字されるべきです。
/// `Anon` はからの文字列のように印字されるべきです。
/// 例えば、Num(777 (Str (cases_on (Str (list (Anon))))))
/// は `list.cases_on.777` になるべき
impl std::fmt::Debug for Name {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.as_ref() {
            Anon => write!(f, ""),
            Str(pfx, hd) => match pfx.as_ref() {
                Anon     => write!(f, "{}", hd),
                owise        => write!(f, "{}.{}", owise, hd)
            },
            Num(pfx, hd) => match pfx.as_ref() {
                Anon     => write!(f, "{}", hd),
                owise        => write!(f, "{}.{}", owise, hd)
            }
        }
    }
}


impl std::fmt::Display for Name {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.as_ref() {
            Anon => write!(f, ""),
            Str(pfx, hd) => match pfx.as_ref() {
                Anon     => write!(f, "{}", hd),
                owise        => write!(f, "{}.{}", owise, hd)
            },
            Num(pfx, hd) => match pfx.as_ref() {
                Anon     => write!(f, "{}", hd),
                owise        => write!(f, "{}.{}", owise, hd)
            }
        }
    }
}


impl std::fmt::Display for InnerName {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Anon => write!(f, ""),
            Str(pfx, hd) => match pfx.as_ref() {
                Anon     => write!(f, "{}", hd),
                owise        => write!(f, "{}.{}", owise, hd)
            },
            Num(pfx, hd) => match pfx.as_ref() {
                Anon     => write!(f, "{}", hd),
                owise        => write!(f, "{}.{}", owise, hd)
            }
        }
    }
}