use std::sync::Arc;
use std::str::SplitWhitespace;
use std::collections::VecDeque as VecD;

use crate::name::{ Name, mk_anon };
use crate::quot::new_quot;
//use crate::inductive::Inductive;
use crate::pretty::components::Notation;
use crate::utils::{ Either::*, END_MSG_ADD, END_MSG_ADD2, ModQueue, DeclarationKindQueue };
use crate::errors;
use crate::level::{ Level, mk_imax, mk_max, mk_succ, mk_param, mk_zero };
use crate::expr::{ Expr, Binding, BinderStyle, mk_app, mk_prop, mk_sort,
                   mk_var, mk_let, mk_pi, mk_lambda, mk_const };
use crate::new_inductive::newinductive::{ InductiveDeclar, InductiveType };
use crate::new_inductive::constructor::{ Constructor };
use crate::inductive::Inductive;

use crate::recursor::RecursorVal;
use crate::env::{ Env, 
                  Modification, 
                  CompiledModification,
                  Axiom, 
                  Definition, 
                  ConstantVal, 
                  ConstantInfo, 
                  ConstantInfo::*,
                  DefinitionVal,
                  DeclarationKind };

use parking_lot::RwLock;

use ParseErr::*;

pub type ParseResult<T> = std::result::Result<T, ParseErr>;

fn fork_inner_env(env : &Arc<RwLock<Env>>) -> Arc<RwLock<Env>> {
    let cloned_env = env.read().clone();
    Arc::new(RwLock::new(cloned_env))
}

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
    pub new_queue_handle : &'s DeclarationKindQueue,
    pub env_handle : &'s Arc<RwLock<Env>>,
    pub new_env_handle : &'s Arc<RwLock<Env>>,
    pub prop : Expr
}

impl<'s> LineParser<'s> {
    pub fn new(queue_handle : &'s ModQueue, env_handle : &'s Arc<RwLock<Env>>, new_queue_handle : &'s DeclarationKindQueue, new_env_handle : &'s Arc<RwLock<Env>>) -> LineParser<'s> {
        let mut parser = LineParser {
            line_num: 1usize,
            names : Vec::with_capacity(12_000),
            levels : Vec::with_capacity(250),
            exprs : Vec::with_capacity(400_000),
            queue_handle,
            new_queue_handle,
            env_handle,
            new_env_handle,
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

    pub fn parse_all(s : String, queue_handle : &'s ModQueue, env_handle : &'s Arc<RwLock<Env>>, new_queue_handle : &'s DeclarationKindQueue, new_env_handle : &'s Arc<RwLock<Env>>) -> ParseResult<()> {
        let mut parser = LineParser::new(queue_handle, env_handle, new_queue_handle, new_env_handle);
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

        parser.new_queue_handle.push(END_MSG_ADD2);
        parser.new_queue_handle.push(END_MSG_ADD2);

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
            otherwise => unreachable!("parser line : {} expectex expression cue, got {:?}", line!(), otherwise)
        };

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

        let new_axiom = crate::env::AxiomVal::new(name.clone(), VecD::from(uparams.clone()), ty.clone(), None);
        let axiom = Axiom::new(name.clone(), Arc::new(uparams), ty);

        self.new_queue_handle.push(Left(DeclarationKind::AxiomDeclar { val : new_axiom }));

        let result = Ok(self.queue_handle.push(Left(Modification::AxiomMod(axiom))));
        result

    }

    pub fn make_definition(&mut self, ws : &mut SplitWhitespace) -> ParseResult<()> {
        let name = self.get_name(ws)?;
        let ty = self.get_expr(ws)?;
        let val = self.get_expr(ws)?;


        let uparams = self.get_uparams(ws)?;

        let NEW_definition = DefinitionVal::new(self.env_handle.clone(), name.clone(), uparams.clone(), ty.clone(), val.clone());

        let def = Definition::new(name.clone(), Arc::new(uparams), ty, val);
        // compiled_old & unwrapped are for debugging only.
        let compiled_old = match Modification::DefMod(def.clone()).compile(&self.env_handle.clone()) {
            crate::env::CompiledModification::CompiledDefinition(declar, rr, TY, VAL) => {
                assert_eq!(&declar.ty, &TY);
                declar
            }
            _ => panic!()
        };


        assert_eq!(&NEW_definition.constant_val.name, &compiled_old.name);
        assert_eq!(&NEW_definition.constant_val.lparams.iter().cloned().collect::<Vec<Level>>(), &compiled_old.univ_params.as_ref().clone());
        assert_eq!(&NEW_definition.constant_val.type_, &compiled_old.ty);
        assert_eq!(NEW_definition.hint.debug_get_regular_height() as usize, compiled_old.height as usize);


        self.new_queue_handle.push(Left(DeclarationKind::DefinitionDeclar { val : NEW_definition }));
        let result = Ok(self.queue_handle.push(Left(Modification::DefMod(def))));
        result
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

        let ind_mod = Inductive::new(name.clone(), Arc::new(param_vec.clone()), ty.clone(), num_params, intros_buf.clone(), self.env_handle.clone());

        let NEW_constr_buf = intros_buf.clone().into_iter().map(|(n, e)| {
            Constructor::new(&n, &e)
        }).collect::<VecD<Constructor>>();

        let NEW_ind_type = InductiveType::new(name.clone(), ty.clone(), NEW_constr_buf);

        //assert_eq!(NEW_ind_type.constructors.len(), ind_mod.intros.len());
        let NEW_cnstr_names = NEW_ind_type.constructors.iter().map(|x| x.name.clone()).collect::<Vec<Name>>();
        let NEW_cnstr_types = NEW_ind_type.constructors.iter().map(|x| x.type_.clone()).collect::<Vec<Expr>>();
        let zipped = NEW_cnstr_names.iter().zip(NEW_cnstr_types);
        for ((n1, t1), (n2, t2)) in zipped.zip(intros_buf) {
            assert_eq!(n1, &n2);
            assert_eq!(t1, t2);
        }



        let NEW_ind = InductiveDeclar::new(
            name.clone(),
            VecD::from(param_vec.clone()), 
            num_params, 
            VecD::from(vec![NEW_ind_type]), 
            false);

        inductive_assertions(&self.env_handle, ind_mod.clone(), &self.new_env_handle, NEW_ind.clone());

        self.new_queue_handle.push(Left(DeclarationKind::InductiveDeclar_ { val : NEW_ind }));
        self.queue_handle.push(Left(Modification::IndMod(ind_mod)));
        Ok(())


        //Ok(self.queue_handle.push(Left(Modification::IndMod(ind_mod))))
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




fn inductive_assertions(old_env : &Arc<RwLock<Env>>, old_ind : Inductive, new_env : &Arc<RwLock<Env>>, new_ind : InductiveDeclar) {
        // DEBUG
        let old_env_clone = fork_inner_env(&old_env);
        let as_mod = Modification::IndMod(old_ind);
        let old_compiled : CompiledModification = as_mod.compile(&old_env_clone);

        let old_major_idx : Option<usize> = old_compiled.get_major_idx();

        let new_env_clone = fork_inner_env(new_env);
        let new_ind_name = new_ind.name.clone();
        let rec_name = new_ind_name.extend_str("rec");
        let added = DeclarationKind::InductiveDeclar_ { val : new_ind }.add_to_env(new_env_clone.clone(), true);

        let fetched_const_info = new_env_clone.read().constant_infos.get(&rec_name).cloned();
        let new_rec_val = match new_env_clone.read().constant_infos.get(&rec_name) {
            Some(ConstantInfo::RecursorInfo(recursor_val @ RecursorVal { .. })) => recursor_val.clone(),
            _ => panic!("Found no recursor for new!")
        };

        assert_eq!(old_compiled.is_k(), new_rec_val.is_k);

        if (!new_rec_val.is_k) {
            let new_major_idx = new_rec_val.nparams 
                              + new_rec_val.nmotives 
                              + new_rec_val.nminors 
                              + new_rec_val.nindices;
            let old_major_idx = match old_major_idx {
                Some(x) => x.clone(),
                None => panic!("old major idx was none, but new was some for {} !", new_ind_name)
            };
            assert_eq!(new_major_idx, old_major_idx);
        }

}