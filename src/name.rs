use std::sync::Arc;

use hashbrown::HashSet;

use InnerName::*;

/// `Name` is an Arc wrapper for the `InnerName` enum, which together represent Lean's hierarchical names, where
/// hierarchical just means "nested namespaces that can be accessed with a dot", like `nat.rec`. They have a very 
/// similar structure to an inductive `List` type, with `Anon`, the anonymous name acting as `Nil`, 
/// while `Str` and `Num` act like `cons`, but specialized to consing string and integer elements respectively.
/// Name values always begin with `Anon`, and can contain any combination of `Str` and `Num` applications, 
/// IE (in pseudo-code) `Num n (Str s (Num n' (Str s' (Anon))))` would be a valid construction.
#[derive(Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct Name(Arc<InnerName>);

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub enum InnerName {
    Anon,
    Str { pfx : Name, hd : String },
    Num { pfx : Name, hd : u64 },
}

pub fn mk_anon() -> Name {
    Name(Arc::new(InnerName::Anon))
}

impl Name {

    pub fn is_anon(&self) -> bool {
        match self {
            Name(inner) => inner.as_ref() == &Anon
        }
    }


    /// Extend some hierarchical name with a string. IE `nat` => `nat.rec`
    pub fn extend_str(&self, hd : &str) -> Self {
        Name::from(Str { pfx : self.clone(), hd : String::from(hd) }) // InnerName -> Name
    }

    /// Extend some hierarchical name with an integer. IE `prod` => `prod.3`
    pub fn extend_num(&self, hd : u64) -> Self {
        Name::from(Num { pfx : self.clone(), hd : hd }) // InnerName -> Name
    }


    /// Given a suggested prefix and a set of names we want to avoid collisions with,
    /// extend the suggestion with an incrementing integer until we get a name that doesn't collide with
    /// any of the names given in `forbidden`. This implementation relies on the laziness of iterators.
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




/// Convenience function to get the `InnerName` from a `Name`    
impl std::convert::AsRef<InnerName> for Name {
    fn as_ref(&self) -> &InnerName {
        match self {
            Name(x) => x.as_ref()
        }
    }
}

/// Convenience function for converting an Arc<InnerName> into its newtype `Name`
impl From<Arc<InnerName>> for Name {
    fn from(x : Arc<InnerName>) -> Name {
        Name(x)
    }
}
// Convenience function for converting an InnerName to a Name
impl From<InnerName> for Name {
    fn from(x : InnerName) -> Name {
        Name(Arc::new(x))
    }
}

/// Creates a Name value from a string slice. 
impl From<&str> for Name {
    fn from(s : &str) -> Name {
        mk_anon().extend_str(s)
    }
}


/// Hierarchical names should display from left to right, with a `.` separating elements, and the anonymous name
/// should display as an empty string.
/// IE the formatted version of Anon ++ Str(list) ++ Str(cases_on) ++ Num(777) should display as
/// `list.cases_on.777`


impl std::fmt::Debug for Name {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.as_ref() {
            Anon => write!(f, "Anon"),
            Str { pfx, hd } => write!(f, "{:?} :: {:?}", pfx, hd),
            Num { pfx, hd } => write!(f, "{:?} :: {:?}", pfx, hd),
        }
    }
}


impl std::fmt::Display for InnerName {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Anon => write!(f, ""),
            Str { pfx, hd } => match pfx.as_ref() {
                Anon     => write!(f, "{}", hd),
                owise        => write!(f, "{}.{}", owise, hd)
            },
            Num { pfx, hd } => match pfx.as_ref() {
                Anon     => write!(f, "{}", hd),
                owise        => write!(f, "{}.{}", owise, hd)
            }
        }
    }
}

impl std::fmt::Display for Name {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}
