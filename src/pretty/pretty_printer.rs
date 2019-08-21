use std::cell::RefCell;
use std::sync::Arc;
use hashbrown::HashSet;
use parking_lot::RwLock;


use crate::name::Name;
use crate::level::{ Level, InnerLevel::* };
use crate::expr::{ Expr, InnerExpr::*, Binding, BinderStyle };
use crate::tc::TypeChecker;
use crate::env::{ Declaration, Env };
use crate::pretty::components::{ word_wrap_val, Notation, Parenable, Notation::*, Doc, InnerDoc::*, MAX_PRIORITY };





// この場合、可変借用を再帰的に取る機能が必要だから、RefCellを用います。
#[derive(Clone)]
pub struct PrettyPrinter {
    pub pp_options : PPOptions,
    pub tc : RefCell<TypeChecker>,
    pub used_lcs : RefCell<HashSet<Name>>
}


impl PrettyPrinter {
    pub fn new(options : Option<PPOptions>, env : &Arc<RwLock<Env>>) -> Self {
        let options = options.unwrap_or_else(|| PPOptions::new_default());
        PrettyPrinter {
            pp_options : options,
            tc : RefCell::new(TypeChecker::new(Some(true), env.clone())),
            used_lcs : RefCell::new(HashSet::with_capacity(100))
        }
    }

    pub fn lookup_notation(&self, name : &Name) -> Option<Notation> {
        self.tc.borrow().env.read().notations.get(name).cloned()
    }

    pub fn nest(&self, doc : Doc) -> Doc {
        doc.group().nest(self.pp_options.indent)
    }


    pub fn pp_name(&self, n : &Name) -> Doc {
        Text(format!("{}", n)).into()
    }

    pub fn pp_level(&self, lvl : &Level) -> Parenable {
        match lvl.as_ref() {
            Max(a, b) => {
                let doc = Doc::from("max")
                           .concat_plus(self.pp_level(a).parens(1))
                           .concat_line(self.pp_level(b).parens(1));
                Parenable::new(0, doc)
            },
            IMax(a, b) => {
                let doc = Doc::from("imax")
                           .concat_plus(self.pp_level(a).parens(1))
                           .concat_line(self.pp_level(b).parens(1));
                Parenable::new(0, doc)

            },
            Param(p) => Parenable::new_max(self.pp_name(p)),
            _ => {
                let (n, inner) = lvl.to_offset();
                match inner.as_ref() {
                    Zero  => Parenable::new_max(Doc::from(format!("{}", n))),
                    _ => {
                        let doc = self.pp_level(inner).parens(1)
                                  .concat("+")
                                  .concat(format!("{}", n));
                        Parenable::new(0, doc)
                    }
                }
            }
        }
    }

    pub fn already_used(&self, n : &Name) -> bool {
        self.used_lcs.borrow().contains(n) 
        || self.tc.borrow().env.read().declarations.get(n).is_some()
    }


    pub fn sanitize_name(&self, n : &Name) -> Name {
        let as_string = format!("{}", n);
        let filtered = as_string.chars()
                                .filter(|c| c.is_alphanumeric() || *c == '_')
                                .skip_while(|c| c.is_digit(10) || *c == '_')
                                .collect::<String>();
        if filtered.is_empty() {
            Name::from("a")
        } else {
            return Name::from(filtered.as_str())
        }
    }

    // This is why Lean's pretty printer is so hard to read w/ `B_ih_1_a_1_hwf`
    pub fn find_unused(&self, base : &Name, idx : usize) -> Name {
        let n = Name::from(format!("{}_{}", base, idx).as_str());
        if self.already_used(&n) {
            self.find_unused(base, idx + 1)
        } else {
            n
        }
    }


    pub fn fresh_name(&self, suggestion : &Name) -> Name {
        let sanitized = self.sanitize_name(suggestion);
        let fresh = if self.already_used(&sanitized) {
            self.find_unused(&sanitized, 0)
        } else {
            sanitized
        };

        self.used_lcs.borrow_mut().insert(fresh.clone());
        return fresh
    }

    pub fn remove_lc(&self, target : &Name) {
        self.used_lcs.borrow_mut().remove(target);
    }

    pub fn pp_bare_binder(&self, binding : &Binding) -> Doc {
        self.pp_name(&binding.pp_name)
        .concat_plus(":")
        .concat_line(self.pp_expr(&binding.ty).parens(1).group())
    }

    pub fn is_implicit(&self, fun : &Expr) -> bool {
        let inferred = self.tc.borrow_mut().infer(fun);
        match self.tc.borrow_mut().whnf(&inferred).as_ref() {
            Pi(_, dom, _) => dom.style != BinderStyle::Default,
            _ => false
        }
    }


    pub fn pp_levels(&self, lvls : &Vec<Level>) -> Doc {
        let as_docs = lvls.into_iter().map(|lvl| {
            self.pp_level(lvl).parens(0)
        });
        Doc::from("{")
        .concat(word_wrap_val(as_docs))
        .concat("}")
        .group()
    }

    pub fn telescope(&self, head : Option<Doc>, binders : &[ParsedBinder]) -> Vec<Doc> {
        let mut acc = Vec::with_capacity(binders.len() + 1);
        if let Some(hd) = head {
            acc.push(hd);
        }

        self.telescope_core(binders, &mut acc);
        acc
    }

    pub fn telescope_core(&self, binders : &[ParsedBinder], acc : &mut Vec<Doc>) {
        let (hd, _) = match binders.split_first() {
            Some((hd, tl)) => (hd, tl),
            None => return 
        };

        let (group, rest) = if hd.style() == BinderStyle::InstImplicit {
            (binders.split_at(1))
        } else {
            let closure = |b : &ParsedBinder| b.style() == hd.style() && b.ty() == hd.ty();
            take_while_slice(binders, closure)
        };

        let mapped_group = group.iter().map(|b| {
            match b.is_anon && !b.occurs_in_body {
                true => Doc::from("_"),
                false => self.pp_name(b.name())
            }
        });

        let bare = word_wrap_val(mapped_group)
                       .concat_plus(":")
                       .concat_line(self.pp_expr(hd.ty()).parens(1).group());

        let match_result = match hd.style() {
            BinderStyle::Default         => Doc::from("(").concat(bare).concat(")"),
            BinderStyle::Implicit        => Doc::from("{").concat(bare).concat("}"),
            BinderStyle::StrictImplicit  => Doc::from("{{").concat(bare).concat("}}"),
            BinderStyle::InstImplicit    => Doc::from("[").concat(bare).concat("]"),
        };

        acc.push(self.nest(match_result));
        self.telescope_core(rest, acc);
    }


    pub fn pp_binders(&self, binders : &[ParsedBinder], inner : Parenable) -> Parenable {
        if let Some((hd, tl)) = binders.split_first() {
            if hd.is_imp() {
                let doc = self.nest(self.pp_expr(hd.ty()).parens(25))
                              .concat_plus("→")
                              .concat(Doc::line()).group()
                              .concat(self.pp_binders(tl, inner).parens(24));
                Parenable::new(24, doc)
            } else if hd.is_forall() {
                let (group, rest) = take_while_slice(binders, |x| x.is_forall());
                let telescoped = word_wrap_val(self.telescope(None, group).into_iter());
                let doc = self.nest(Doc::from("∀").concat_plus(telescoped)
                                                  .concat(","))
                                                  .concat_line(self.pp_binders(rest, inner).parens(0));
                Parenable::new(0, doc)
            } else {
                assert!(hd.is_lambda());
                let (group, rest) = take_while_slice(binders, |x| x.is_lambda());
                let telescoped = word_wrap_val(self.telescope(None, group).into_iter());
                let doc = self.nest(Doc::from("λ").concat_plus(telescoped)
                                                  .concat(","))
                                                  .concat_line(self.pp_binders(rest, inner).parens(0));
                Parenable::new(0, doc)
            }
        } else {
            return inner
        }
    }



    pub fn const_name(&self, n : &Name) -> Parenable {
        if !self.pp_options.implicit {
            Parenable::new_max(self.pp_name(n))
        } else {
            Parenable::new_max(Doc::from("@").concat(self.pp_name(n)))
        }
    }

    pub fn pp_app_core(&self, e : &Expr) -> Parenable {
        let mut apps = Vec::new();
        let mut acc = e;

        while let App(_, lhs, rhs) = acc.as_ref() {
            if !self.pp_options.implicit && self.is_implicit(lhs) {
                acc = lhs;
            } else {
                apps.push(rhs.clone());
                acc = lhs;
            }
        }

        match acc.as_ref() {
            _ if apps.is_empty() => self.pp_expr(acc),
            Const(_, name, _) if self.pp_options.notation => {
                match self.lookup_notation(name) {
                    Some(Prefix(_, ref prio, ref op)) if apps.len() == 1 => {
                        let z = &apps[apps.len() - 1];
                        let doc = Doc::from(op)
                                  .concat(Doc::zero_width_line())
                                  .group()
                                  .concat(self.pp_expr(z).parens(*prio));
                        Parenable::new(prio - 1, doc)
                    },
                    Some(Postfix(_, ref prio, ref op)) if apps.len() == 1 => {
                        let z = &apps[apps.len() - 1];
                        let doc = Doc::from(self.pp_expr(z).parens(*prio))
                                  .concat(Doc::zero_width_line())
                                  .concat(op).group();
                        Parenable::new(prio - 1, doc)
                    },
                    Some(Infix(_, ref prio, ref op)) if apps.len() == 2 => {
                        let z = &apps[apps.len() - 1];
                        let s = &apps[apps.len() - 2];
                        let doc = self.pp_expr(z).parens(*prio)
                                  .concat(op)
                                  .concat(Doc::zero_width_line())
                                  .concat(self.pp_expr(s).parens(*prio));
                        Parenable::new(prio - 1, self.nest(doc))
                    },
                    _ => self.print_default(acc, &apps)
                }
            },
            _ => self.print_default(acc, &apps)
        }
    }

    pub fn print_default(&self, f : &Expr, apps : &Vec<Expr>) -> Parenable {
        let iter = Some(self.pp_expr(f).parens(MAX_PRIORITY - 1).group())
                   .into_iter()
                   .chain(apps.into_iter().rev().map(|app| {
                       self.pp_expr(&app).parens(MAX_PRIORITY).group()
                   }));

        Parenable::new(MAX_PRIORITY - 1, self.nest(word_wrap_val(iter)))
    }

    pub fn pp_sort_core(&self, level : &Level) -> Parenable {
        if level.is_zero() && self.pp_options.notation {
            Parenable::new_max(Doc::from("Prop"))
        } else if let Succ(x) = level.as_ref() {
            Parenable::new_max(Doc::from("Type").concat_plus(self.pp_level(x).parens(MAX_PRIORITY)))
        } else {
            Parenable::new_max(Doc::from("Sort").concat_plus(self.pp_level(level).parens(MAX_PRIORITY)))
        }
    }

    pub fn pp_const_core(&self, name : &Name, levels : &Vec<Level>) -> Parenable {
        if self.tc.borrow().env.read().declarations.get(name).is_some() {
            self.const_name(name)
        } else {
            let uparams = if levels.is_empty() {
                Doc::from("")
                .concat(self.pp_levels(levels.as_ref()))
            } else {
                Doc::from(".")
                .concat(self.pp_levels(levels.as_ref()))
            };
            let doc = Doc::from("@")
                      .concat(self.pp_name(name))
                      .concat(uparams);

            Parenable::new_max(doc)
        }
    }


    pub fn pp_let_core(&self, dom : &Binding, val : &Expr, body : &Expr) -> Parenable {
        let suggestion = dom.clone().as_local();
        assert!(suggestion.is_local());
        let binding = Binding::from(&suggestion);
        let fresh_lc_name = self.fresh_name(&binding.pp_name);
        let swapped_lc = suggestion.swap_local_binding_name(&fresh_lc_name);

        let instd = body.instantiate(Some(&swapped_lc).into_iter());
        let doc = self.nest(Doc::from("let").concat_plus(self.pp_bare_binder(&swapped_lc.lc_binding()).group())
                      .concat_plus(":=")
                      .concat_line(self.pp_expr(val).parens(0).group())
                      .concat("in"))
                      .concat_line(self.pp_expr(&instd).parens(0)).group();
        let result = Parenable::new(0, doc);

        self.remove_lc(&fresh_lc_name);
        result
    }

    pub fn pp_expr(&self, e : &Expr) -> Parenable {
        if !self.pp_options.proofs && self.tc.borrow_mut().is_proof(e) {
            return Parenable::new_max("_".into())
        }

        match e.as_ref() {
            Var(_, idx) => Parenable::new_max(format!("#{}", idx).into()),
            Sort(_, level) => self.pp_sort_core(level),
            Const(_, name, levels) => self.pp_const_core(name, levels.as_ref()),
            Local(.., of) => self.const_name(&of.pp_name),
            | Lambda(..)
            | Pi(..) => {
                let (binders, instd) = self.parse_binders(e);
                let new_inner = self.pp_expr(&instd);
                let new_vec = Vec::from(binders.clone());
                let new_result = self.pp_binders(new_vec.as_slice(), new_inner);
                self.restore_lc_names(&binders);
                new_result
            }
            Let(_, dom, val, body) => self.pp_let_core(dom, val, body),
            App(..) => self.pp_app_core(e)
        }

    }


    pub fn restore_lc_names(&self, binders : &Vec<ParsedBinder>) {
        for elem in binders.into_iter().rev() {
            self.used_lcs.borrow_mut().remove(&elem.lc.lc_binding().pp_name);
        }
    }

    pub fn get_ups(&self, declar : &Declaration) -> Doc {
        match declar.univ_params.as_ref() {
            v if v.is_empty() => Doc::from(""),
            v => Doc::from(" ").concat(self.pp_levels(v))
        }
    }


    pub fn main_def(&self, declar : &Declaration, val : Expr) -> Doc {
        let (binders, ty) = self.parse_binders(&declar.ty);

        // inlined parse_params
        let mut slice_split_idx = 0usize;
        let mut val_acc = &val;
        // break loop when at least one of these three conditions is true :
        // 1. binders is exhausted
        // 2. val is no longer a Lambda
        // 3. is_forall(popped element) == false
        for elem in binders.iter() {
            match val_acc.as_ref() {
                Lambda(.., inner_val) if elem.is_forall() => {
                    slice_split_idx += 1;
                    val_acc = inner_val;
                },
                _ => break
            }
        }
        let (params_slice, binders_slice) = binders.split_at(slice_split_idx);
        let instd = val_acc.instantiate(params_slice.into_iter().rev().map(|x| &x.lc));
        // end inlined

        let is_prop = self.tc.borrow_mut().is_proposition(&declar.ty);
        let cmd = match is_prop {
            true => "lemma",
            false => "def"
        };

        let pp_val = match is_prop && !self.pp_options.proofs {
            true => "_".into(),
            false => self.pp_expr(&instd).parens(0).group()
        };


        let new_telescoped = self.telescope(Some(self.pp_name(&declar.name)), params_slice);

        let sub_doc_new = self.nest(word_wrap_val(new_telescoped.into_iter()))
                          .concat_plus(":")
                          .concat_line(self.pp_binders(binders_slice, self.pp_expr(&ty)).parens(0).group())
                          .concat_plus(":=");


        let result = Doc::from(cmd).concat(self.get_ups(declar))
                      .concat_plus(self.nest(sub_doc_new))
                      .concat_line(pp_val)
                      .concat(Doc::line());

        self.restore_lc_names(&binders);
        result
    }


    pub fn main_axiom(&self, declar : &Declaration) -> Doc {
        let (binders, instd) = self.parse_binders(&declar.ty);
        let doc = {
            let (prms, rst) = take_while_slice(binders.as_slice(), |x| x.is_forall()); 
            let prms_as_vec = Vec::from(prms.clone());
            let slice = prms_as_vec.as_slice();
            let telescoped = self.telescope(Some(self.pp_name(&declar.name)), slice);
            let sub_doc_new = self.nest(word_wrap_val(telescoped.into_iter())
                              .concat_plus(":")
                              .concat_line(
                                  self.pp_binders(
                                      rst, self.pp_expr(&instd)).parens(0).group()));
            Doc::from("axiom").concat(self.get_ups(declar))
                              .concat_plus(sub_doc_new)
                              .concat(Doc::line())
        };
        self.restore_lc_names(&binders);
        match declar.builtin {
            true => Doc::from("/- builtin -/").concat_plus(doc),
            false => doc
        }
    }

    pub fn pp_main(&self, declar : &Declaration) -> Doc {

        let env_result = self.tc.borrow()
                                .env
                                .read()
                                .get_value(&declar.name)
                                .cloned();
        match env_result {
            // definition/lemma branch
            Some(val) => self.main_def(declar, val.clone()),
            // axiom branch
            None => self.main_axiom(declar)
        }

    }

    pub fn render_expr(&self, e : &Expr) -> String {
        self.pp_expr(e).doc.group().render(80)
    }


    pub fn print_declar(options : Option<PPOptions>, n : &Name, env : &Arc<RwLock<Env>>) -> String {
        let declar = match env.read().declarations.get(n) {
            Some(d) => d.clone(),
            None => return String::new()
        };

        let pp = PrettyPrinter::new(options, env);

        pp.pp_main(&declar)
          .group()
          .render(pp.pp_options.width)
    }

    pub fn parse_binders(&self, e : &Expr) -> (Vec<ParsedBinder>, Expr) {
        let mut acc = e;
        let mut ctx = Vec::<ParsedBinder>::new();

        while let | Pi(_, dom, body) 
                  | Lambda(_, dom, body) = acc.as_ref() {
            let new_name = self.fresh_name(&dom.pp_name);
            let new_ty = dom.ty.instantiate(ctx.iter().rev().map(|x| &x.lc));
            let new_dom = Binding::mk(new_name, new_ty, dom.style);
            let new_local = new_dom.as_local(); 
            let new_parsed_binder = ParsedBinder::new(acc.binder_is_pi(),
                                                      has_var(body, 0),
                                                      dom.pp_name.is_anon(),
                                                      new_local);
            ctx.push(new_parsed_binder);
            acc = body;
        }

        let instd = acc.instantiate(ctx.iter().rev().map(|x| &x.lc));
        (ctx, instd)
    }

}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedBinder {
    pub is_pi : bool,
    pub occurs_in_body : bool,
    pub is_anon : bool,
    pub lc : Expr,
}

impl ParsedBinder {
    pub fn new(is_pi : bool, 
               occurs_in_body : bool, 
               is_anon : bool, 
               lc : Expr) -> Self {
        
        ParsedBinder {
            is_pi,
            occurs_in_body,
            is_anon,
            lc,
        }
    }

    pub fn is_imp(&self) -> bool {
        self.is_pi 
        && self.lc.lc_binding().style == BinderStyle::Default 
        && self.is_anon 
        && !self.occurs_in_body
    }

    pub fn is_forall(&self) -> bool {
        self.is_pi && !self.is_imp()
    }

    pub fn is_lambda(&self) -> bool {
        !self.is_pi
    }

    pub fn style(&self) -> BinderStyle {
        self.lc.lc_binding().style
    }

    pub fn ty(&self) -> &Expr {
        &self.lc.lc_binding().ty
    }

    pub fn name(&self) -> &Name {
        &self.lc.lc_binding().pp_name
    }
}


pub fn has_var(e : &Expr, i : u64) -> bool {
    if e.var_bound() as u64 <= i {
        return false
    }
    match e.as_ref() {
        Var(_, idx) => *idx == i,
        App(_, a, b) => has_var(a, i) || has_var(b, i),
        Lambda(_, dom, body) => has_var(&dom.ty, i) || has_var(body, i + 1),
        Pi(_, dom, body) => has_var(&dom.ty, i) || has_var(body, i + 1),
        Let(_, dom, val, body) => has_var(&dom.ty, i) || has_var(val, i) || has_var(body, i + 1),
        _ => unreachable!()
    }
}

pub fn take_while_slice<T>(s : &[T], f : impl Fn(&T) -> bool) -> (&[T], &[T]) {
    let mut idx = 0usize;
    while idx < s.len() && f(&s[idx]) {
        idx += 1
    }
    let lhs = &s[0..idx];
    let rhs = &s[idx..];
    (lhs, rhs)
}

pub fn render_expr(e : &Expr, env : &Arc<RwLock<Env>>) -> String {
    let pp = PrettyPrinter::new(None, env);
    pp.pp_expr(e)
      .doc
      .group()
      .render(pp.pp_options.width)
}


#[derive(Clone)]
pub struct PPOptions {
    pub all : bool,
    pub implicit : bool,
    pub notation : bool,
    pub proofs : bool,
    pub locals_full_names : bool,
    pub indent : usize,
    pub width : usize
}

impl PPOptions {
    pub fn new_all_false() -> Self {
        PPOptions {
            all : false,
            implicit : false,
            notation : false,
            proofs : false,
            locals_full_names : false,
            indent : 0usize,
            width : 0usize
        }
    }

    pub fn new_default() -> Self {
        PPOptions {
            all : false,
            implicit : false,
            notation : true,
            proofs : true,
            locals_full_names : false,
            indent : 2usize,
            width : 80usize
        }
    }
}