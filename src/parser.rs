use std::sync::Arc;
use std::str::SplitWhitespace;

use crate::name::{ Name, mk_anon };
use crate::env::{ Env, Modification, Axiom, Definition };
use crate::quot::new_quot;
use crate::inductive::Inductive;
use crate::pretty::components::Notation;
use crate::utils::{ Either::*, END_MSG_ADD, ModQueue };
use crate::errors;
use crate::level::{ Level, mk_imax, mk_max, mk_succ, mk_param, mk_zero };
use crate::expr::{ Expr, Binding, BinderStyle, mk_app, mk_prop, mk_sort,
                   mk_var, mk_let, mk_pi, mk_lambda, mk_const };

use parking_lot::RwLock;

use ParseErr::*;

pub type ParseResult<T> = std::result::Result<T, ParseErr>;

#[derive(Debug, Clone)]
pub enum ParseErr {
    Exhausted(usize, u32),
    ParseInt(usize, u32, std::num::ParseIntError),
    StringErr(usize, u32, String),
}

impl std::fmt::Display for ParseErr {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Exhausted(line, source) => write!(f, "Parse error at source line {}, source line {} : source iterator unexpectedly yielded None (was out of elements)", line, source),
            ParseInt(line, source, err) => write!(f, "Parse error at lean output line {}, source line {} : {}", line, source, err),
            StringErr(line, source, err) => write!(f, "Parse error at lean output line {}, source line {} : {}", line, source, err),
        }
    }
}


pub struct LineParser<'s> {
    pub line_num: usize,
    pub names  : Vec<Name>,
    pub levels : Vec<Level>,
    pub exprs  : Vec<Expr>,
    pub queue_handle : &'s ModQueue,
    pub env_handle : &'s Arc<RwLock<Env>>,
    pub prop : Expr
}

impl<'s> LineParser<'s> {
    pub fn new(queue_handle : &'s ModQueue, env_handle : &'s Arc<RwLock<Env>>) -> LineParser<'s> {
        let mut parser = LineParser {
            line_num: 1usize,
            names : Vec::with_capacity(12_000),
            levels : Vec::with_capacity(250),
            exprs : Vec::with_capacity(400_000),
            queue_handle,
            env_handle,
            prop : mk_prop()

        };

        parser.names.push(mk_anon());
        parser.levels.push(mk_zero());
        parser
    }

    pub fn ref_anon(&self) -> Name {
        self.names[0].clone()
    }

    pub fn ref_zero(&self) -> Level {
        self.levels[0].clone()
    }

    pub fn ref_prop(&self) -> Expr {
        self.prop.clone()
    }

    pub fn parse_all(s : String, queue_handle : &'s ModQueue, env_handle : &'s Arc<RwLock<Env>>) -> ParseResult<()> {
        let mut parser = LineParser::new(queue_handle, env_handle);
        let mut as_lines = s.lines();

        while let Some(line) = &mut as_lines.next() {
            match parser.try_next(line) {
                Ok(_) => (),
                Err(e) => return Err(e)
            }
            parser.line_num  += 1;
        }

        parser.queue_handle.push(END_MSG_ADD);
        parser.queue_handle.push(END_MSG_ADD);
        Ok(())
    }

    pub fn try_next(&mut self, line : &str) -> ParseResult<()> {
        let mut ws = line.split_whitespace();
        match ws.next().ok_or(Exhausted(self.line_num, line!()))? {
            "#AX"          => self.make_axiom(&mut ws),
            "#DEF"         => self.make_definition(&mut ws),
            "#QUOT"        => self.make_quotient(),
            "#IND"         => self.make_inductive(&mut ws),
            s @ "#INFIX"   => self.make_notation(s, line, &mut ws),
            s @ "#PREFIX"  => self.make_notation(s, line, &mut ws),
            s @ "#POSTFIX" => self.make_notation(s, line, &mut ws),
            owise1         => {
                let leading_num = owise1.parse::<usize>()
                                        .map_err(|e| ParseInt(self.line_num, line!(), e))?;
                let mut as_chars = ws.next()
                                     .ok_or(Exhausted(self.line_num, line!()))?
                                     .chars();
                assert!(as_chars.next() == Some('#')); 

                match as_chars.next() {
                    Some('N') => self.make_name(leading_num, as_chars.next().ok_or(Exhausted(self.line_num, line!()))?, &mut ws),
                    Some('U') => self.make_level(leading_num, as_chars.next().ok_or(Exhausted(self.line_num, line!()))?, &mut ws),
                    Some('E') => self.make_expr(leading_num, as_chars.next().ok_or(Exhausted(self.line_num, line!()))?, &mut ws),
                    owise2 => return Err(StringErr(self.line_num, line!(), errors::err_parse_kind(&owise2)))
                }
            }
        }
    }


    fn parse_usize(&mut self, ws : &mut SplitWhitespace) -> ParseResult<usize> {
          ws.next()
            .ok_or(Exhausted(self.line_num, line!()))
            .and_then(|item| item.parse::<usize>().map_err(|e| ParseInt(self.line_num, line!(), e)))
    }

    fn parse_u64(&mut self, ws : &mut SplitWhitespace) -> ParseResult<u64> {
          ws.next()
            .ok_or(Exhausted(self.line_num, line!()))
            .and_then(|item| item.parse::<u64>().map_err(|e| ParseInt(self.line_num, line!(), e)))
    }
    
    
    fn parse_rest_usize(&mut self, ws : &mut SplitWhitespace) -> ParseResult<Vec<usize>> {
           ws.map(|elem| elem.parse::<usize>().map_err(|e| ParseInt(self.line_num, line!(), e)))
             .collect::<ParseResult<Vec<usize>>>()
    }
    
    fn parse_rest_string(&mut self, ws : &mut SplitWhitespace) -> String {
        ws.collect::<String>()
    }

    pub fn get_levels(&mut self, ws : &mut SplitWhitespace) -> ParseResult<Vec<Level>> {
          ws.into_iter()
            .map(|elem| elem.parse::<usize>().map_err(|e| ParseInt(self.line_num, line!(), e)))
            .map(|res| res.map(|idx| self.levels.get(idx).map(|x| x).cloned().unwrap_or_else(|| self.ref_zero())))
            .collect::<ParseResult<Vec<Level>>>()
    }

    pub fn get_uparams(&mut self, ws : &mut SplitWhitespace) -> ParseResult<Vec<Level>> {
          ws.into_iter()
            .map(|elem| elem.parse::<usize>().map_err(|e| ParseInt(self.line_num, line!(), e)))
            .map(|res| res.map(|idx| {
                let name = self.names.get(idx).cloned().unwrap_or_else(|| self.ref_anon());
                mk_param(name)
            }))
            .collect::<ParseResult<Vec<Level>>>()
    }

    pub fn parse_binder_info(&mut self, ws : &mut SplitWhitespace) -> ParseResult<BinderStyle> {
        ws.next().map(|elem| match elem {
            s if s.contains("#BD") => BinderStyle::Default,
            s if s.contains("#BI") => BinderStyle::Implicit,
            s if s.contains("#BC") => BinderStyle::InstImplicit,
            s if s.contains("#BS") => BinderStyle::StrictImplicit,
            _ => unreachable!(),
        }).ok_or(Exhausted(self.line_num, line!()))
    }

    pub fn get_name(&mut self, ws : &mut SplitWhitespace) -> ParseResult<Name> {
        self.parse_usize(ws)
            .map(|idx| self.names.get(idx).map(|x| x).cloned().unwrap_or_else(|| self.ref_anon()))
    }


    pub fn get_level(&mut self, ws : &mut SplitWhitespace) -> ParseResult<Level> {
        self.parse_usize(ws)
            .map(|idx| self.levels.get(idx).map(|x| x).cloned().unwrap_or_else(|| self.ref_zero()))
    }

    pub fn get_expr(&mut self, ws : &mut SplitWhitespace) -> ParseResult<Expr> {
        self.parse_usize(ws)
            .map(|idx| self.exprs.get(idx).map(|x| x).cloned().unwrap_or_else(|| self.ref_prop()))
    }

    pub fn make_name(&mut self, new_pos : usize, kind : char, ws : &mut SplitWhitespace) -> ParseResult<()> {
        let prefix_name       = self.get_name(ws)?;
        let new_name = match kind {
            'S' => prefix_name.extend_str(self.parse_rest_string(ws).as_str()),
            'I' => self.parse_u64(ws).map(|hd| prefix_name.extend_num(hd))?,
            _ => unreachable!("parser line : {}", line!())
        };

        #[cfg(feature = "tracing")]
        {
            ((*crate::tracing::UNIV_TRACE_ITEMS)).write().unique_inner.insert(crate::tracing::TraceItem::N(new_name.clone()));
        }


        write_elem_strict(&mut self.names, new_name, new_pos)
    }


    pub fn make_level(&mut self, new_pos : usize, kind : char, ws : &mut SplitWhitespace) -> ParseResult<()> {

        let new_level = match kind {
            'S'  => mk_succ(self.get_level(ws)?),
            'M'  => mk_max(self.get_level(ws)?, self.get_level(ws)?),
            'I'  => mk_imax(self.get_level(ws)?, self.get_level(ws)?),
            'P'  => mk_param(self.get_name(ws)?),
            _ => unreachable!("parser line : {}", line!())
        };

        #[cfg(feature = "tracing")]
        {
            ((*crate::tracing::UNIV_TRACE_ITEMS)).write().unique_inner.insert(crate::tracing::TraceItem::L(new_level.clone()));
        }

        write_elem_strict(&mut self.levels, new_level, new_pos)
    }


    pub fn make_expr(&mut self, new_pos : usize, kind : char, ws : &mut SplitWhitespace) -> ParseResult<()> {

        let new_expr = match kind {
            'V' => mk_var(self.parse_u64(ws)?),
            'S' => mk_sort(self.get_level(ws)?),
            'C' => mk_const(self.get_name(ws)?, self.get_levels(ws)?),
            'A' => mk_app(self.get_expr(ws)?, self.get_expr(ws)?),
            'L' => {
                let binder_info = self.parse_binder_info(ws)?;
                let binder_name = self.get_name(ws)?;
                let domain = self.get_expr(ws)?;
                let lambda = mk_lambda(Binding::mk(binder_name, domain, binder_info), self.get_expr(ws)?);
                lambda
            },
            'P' => {
                let binder_info = self.parse_binder_info(ws)?;
                let binder_name = self.get_name(ws)?;
                let dom = self.get_expr(ws)?;
                mk_pi(Binding::mk(binder_name, dom, binder_info), self.get_expr(ws)?)
            },
            'Z' => {
                let name = self.get_name(ws)?;
                let ty = self.get_expr(ws)?;
                let val = self.get_expr(ws)?;
                let body = self.get_expr(ws)?;
                mk_let(Binding::mk(name, ty, BinderStyle::Default), val, body)
            },
            otherwise => unreachable!("parser line : {} expected expression cue, got {:?}", line!(), otherwise)
        };

        #[cfg(feature = "tracing")]
        {
            ((*crate::tracing::UNIV_TRACE_ITEMS)).write().unique_inner.insert(crate::tracing::TraceItem::E(new_expr.clone()));
        }

        write_elem_strict(&mut self.exprs, new_expr, new_pos)
    }



    pub fn make_notation(&mut self, kind : &str, line : &str, ws : &mut SplitWhitespace) -> ParseResult<()> {
        let name = self.get_name(ws)?;
        let priority = self.parse_usize(ws)?;
        // Elegance.
        let symbol = line.chars().skip_while(|x| !x.is_whitespace())
                                 .skip(1)
                                 .skip_while(|x| !x.is_whitespace())
                                 .skip(1)
                                 .skip_while(|x| !x.is_whitespace())
                                 .skip(1)
                                 .collect::<String>();
        let made = match kind {
            "#PREFIX"  => Notation::new_prefix(name.clone(), priority, symbol),
            "#INFIX"   => Notation::new_infix(name.clone(), priority, symbol),
            "#POSTFIX" => Notation::new_postfix(name.clone(), priority, symbol),
            _ => unreachable!()
        };

        self.env_handle.write().add_notation(&name, made);
        Ok(())
    }

    pub fn make_axiom(&mut self, ws : &mut SplitWhitespace) -> ParseResult<()> {
        let name = self.get_name(ws)?;
        let ty = self.get_expr(ws)?;
        let uparams = self.get_uparams(ws)?;
        let axiom = Axiom::new(name, Arc::new(uparams), ty);
        Ok(self.queue_handle.push(Left(Modification::AxiomMod(axiom))))

    }

    pub fn make_definition(&mut self, ws : &mut SplitWhitespace) -> ParseResult<()> {
        let name = self.get_name(ws)?;
        let ty = self.get_expr(ws)?;
        let val = self.get_expr(ws)?;
        let uparams = self.get_uparams(ws)?;
        let def = Definition::new(name, Arc::new(uparams), ty, val);
        Ok(self.queue_handle.push(Left(Modification::DefMod(def))))
    }

    pub fn make_quotient(&mut self) -> ParseResult<()> {
        self.queue_handle.push(Left(new_quot()));
        Ok(())
    }

    pub fn make_inductive(&mut self, ws : &mut SplitWhitespace) -> ParseResult<()> {
        let num_params = self.parse_usize(ws)?;
        let name = self.get_name(ws)?;
        let ty = self.get_expr(ws)?;
        let num_intros = self.parse_usize(ws)?;
        let rest_usize = self.parse_rest_usize(ws)?;
        let (intros, params) = rest_usize.split_at(2 * num_intros);

        let param_vec = params.into_iter().map(|idx| {
            let fetched_name = self.names.get(*idx).cloned().unwrap_or_else(|| self.ref_anon());
            mk_param(fetched_name)
        }).collect::<Vec<Level>>();

        let mut intros_buf : Vec<(Name, Expr)> = Vec::new();

        for two_slice in intros.chunks(2usize) {
            let name = self.names.get(two_slice[0]).cloned().unwrap_or_else(|| self.ref_anon());
            let ty = self.exprs.get(two_slice[1]).cloned().unwrap_or_else(|| self.ref_prop());
            intros_buf.push((name, ty));
        }

        let ind_mod = Inductive::new(name, Arc::new(param_vec), ty, num_params, intros_buf, self.env_handle.clone());
        Ok(self.queue_handle.push(Left(Modification::IndMod(ind_mod))))
    }

}


// FIXME add command-line flag for strict/non-strict export file parsing.
// Strict assumes that well-formed export files will not have 'holes' when filling
// in comopnent arrays; IE all items will be placed consecutively.
fn write_elem_strict<T>(v : &mut Vec<T>, new_elem : T, pos : usize) -> ParseResult<()> {
    assert!(v.len() == pos);
    match v.get_mut(pos) {
        Some(_) => { 
            eprintln!("malformed export file; components should never require replacement within vectors.");
            std::process::exit(-1);
        },
        None => {
            v.push(new_elem);
        }
    }
    Ok(())
}
