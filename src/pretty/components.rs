use std::sync::Arc;

use crate::name::Name;
use Notation::*;

pub const MAX_PRIORITY : usize = 1024;
#[derive(Clone, PartialEq)]
pub enum Notation {
    //    function, priority, op
    Prefix  (Name, usize, String),
    Infix   (Name, usize, String),
    Postfix (Name, usize, String),
}


impl Notation {
    pub fn new_prefix(func : Name, priority : usize, op : String) -> Self {
        Prefix(func, priority, op)
    }

    pub fn new_infix(func : Name, priority : usize, op : String) -> Self {
        Infix(func, priority, op)
    }

    pub fn new_postfix(func : Name, priority : usize, op : String) -> Self {
        Postfix(func, priority, op)
    }


    pub fn fn_(&self) -> &Name {
        match self {
            | Prefix  ( func, .. ) 
            | Infix   ( func, .. ) 
            | Postfix ( func, .. ) => func,
        }
    }

    pub fn priority(&self) -> usize {
        match self {
            | Prefix  ( _, priority, _ ) 
            | Infix   ( _, priority, _ )
            | Postfix ( _, priority, _ ) => *priority,
        }
    }

    pub fn op(&self) -> &String {
        match self {
            | Prefix  ( _, _, op )
            | Infix   ( _, _, op )
            | Postfix ( _, _, op ) => op
        }
    }
}

#[derive(Debug, Clone)]
pub struct Doc(Arc<InnerDoc>);

impl From<&String> for Doc {
    fn from(s : &String) -> Doc {
        Text(s.clone()).into()
    }
}

impl From<String> for Doc {
    fn from(s : String) -> Doc {
        Text(s).into()
    }
}

impl From<&str> for Doc {
    fn from(s : &str) -> Doc {
        Text(String::from(s)).into()
    }
}

#[derive(Debug, Clone)]
pub enum InnerDoc {
    Concat(Doc, Doc),
    Nest(usize, Doc),
    Text(String),
    Line(String),
    Group(Doc)
}

use InnerDoc::*;

impl std::convert::AsRef<InnerDoc> for Doc {
    fn as_ref(&self) -> &InnerDoc {
        match self {
            Doc(x) => x.as_ref()
        }
    }
}

impl From<InnerDoc> for Doc {
    fn from(t : InnerDoc) -> Doc {
        Doc(Arc::new(t))
    }
}

impl From<&InnerDoc> for Doc {
    fn from(t : &InnerDoc) -> Doc {
        Doc(Arc::new(t.clone()))
    }
}

impl Doc {

    pub fn line() -> Doc {
        Line(format!(" ")).into()
    }

    pub fn zero_width_line() -> Doc {
        Line(format!("")).into()
    }

    pub fn as_text(t : String) -> Doc {
        Text(t).into()
    }

    // unused
    //pub fn sep(&self, docs : &[Doc]) -> Doc {
    //    let mut as_iter = docs.into_iter().cloned();
    //    // pull off initial element for fold
    //    match as_iter.next() {
    //        None => Doc::from(""),
    //        Some(fst) => as_iter.fold(fst, |acc, next| {
    //            self.clone().concat(next)
    //            //let lhs = Doc::concat(acc, self.clone());
    //            //Doc::concat(lhs, next)
    //        })
    //    }
    //}


    pub fn group(&self) -> Doc {
        Group(self.clone()).into()
    }

    pub fn nest(&self, idx : usize) -> Doc {
        Nest(idx, self.clone()).into()
    }




    pub fn flat_size(&self) -> usize {
        match self.as_ref() {
            Concat(a, b) => a.flat_size() + b.flat_size(),
            Nest(_, d) => d.flat_size(),
            Text(t) => t.len(),
            Line(x) => x.len(),
            Group(a) => a.flat_size()
        }
    }

    pub fn contains_line(&self) -> bool {
        match self.as_ref() {
            Line(_) => true,
            Concat(a, b) => a.contains_line() || b.contains_line(),
            Nest(_, d) => d.contains_line(),
            Text(_) => false,
            Group(a) => a.contains_line()
        }
    }

    pub fn dist_to_first_line(&self) -> usize {
        match self.as_ref() {
            Line(_) => 0,
            Concat(a, b) => a.dist_to_line(b.dist_to_first_line()),
            Nest(_, d) => d.dist_to_first_line(),
            Text(t) => t.len(),
            Group(a) => a.dist_to_first_line()
        }
    }

    pub fn dist_to_line(&self, after : usize) -> usize {
        if self.contains_line() {
            self.dist_to_first_line()
        } else {
            self.dist_to_first_line() + after
        }
    }

    pub fn render(self, line_width : usize) -> String {
        let mut acc = String::new();
        let mut eol = acc.len() + line_width;

        self.render_core(0, false, 0, line_width, &mut eol, &mut acc);
        acc
    }

    pub fn render_core(&self,  
                       nest : usize, 
                       flatmode : bool, 
                       dist_to_next_line : usize, 
                       line_width : usize,
                       eol : &mut usize,
                       acc : &mut String) {
        match self.as_ref() {
            Concat(a, b) => {
                a.render_core(nest, 
                              flatmode, 
                              b.dist_to_line(dist_to_next_line), 
                              line_width, 
                              eol, 
                              acc);
                b.render_core(nest, flatmode, dist_to_next_line, line_width, eol, acc);
            },
            Nest(idx, a) => {
                a.render_core(nest + idx, flatmode, dist_to_next_line, line_width, eol, acc);
            },
            Text(t) => {
                acc.push_str(t.as_str());
            },
            Line(x) => {
                if flatmode {
                    acc.push_str(x.as_str());
                } else {
                    assert!(!flatmode);
                    acc.push_str("\n");
                    std::mem::replace(eol, (acc.len() + line_width));
                    for _ in 0..nest {
                        acc.push(' ');
                    }
                }
            },
            Group(a) => {
                a.render_core(nest, 
                              flatmode || acc.len() + a.flat_size() + dist_to_next_line <= *eol,
                              dist_to_next_line, 
                              line_width, 
                              eol, 
                              acc);
            }
        }
    }

    pub fn concat(self, other : impl Into<Doc>) -> Doc {
        Concat(self, other.into()).into()
    }

    pub fn concat_line(self, other : impl Into<Doc>) -> Doc {
        let lhs = Concat(self, Doc::line()).into();
        Concat(lhs, other.into()).into()
    }

    pub fn concat_plus(self, rhs : impl Into<Doc>) -> Doc {
        let lhs = Concat(self, Text(format!(" ")).into()).into();
        Concat(lhs, rhs.into()).into()
    }

}

pub fn word_wrap_val(s : impl Iterator<Item = Doc>) -> Doc {
    let mut fold_source = s.enumerate()
                           .map(|(idx, elem)| {
                               if idx == 0 {
                                   elem.clone()
                               } else {
                                   Doc::line().concat(elem.clone()).group()
                               }
                           });
    match fold_source.next() {
        None => Doc::from(""),
        Some(init) => fold_source.fold(init, |acc, next| acc.concat(next))
    }
}


pub struct Parenable {
    pub priority : usize,
    pub doc : Doc
}



impl Parenable {

    pub fn new(priority : usize, doc : Doc) -> Self {
        Parenable {
            priority,
            doc
        }
    }

    pub fn new_max(doc : Doc) -> Self {
        Parenable {
            priority : MAX_PRIORITY,
            doc
        }
    }

    pub fn parens(&self, new_priority : usize) -> Doc {
        if new_priority > self.priority {
            Doc::from("(").concat(self.doc.clone()).concat(")")

        } else {
            self.doc.clone()
        }
    }
}
