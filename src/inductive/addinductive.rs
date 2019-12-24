use crate::inductive::newinductive::{ get_all_inductive_names, InductiveType };
use crate::recursor::RecInfo;

use crate::utils::ShortCircuit::*;
use crate::name::Name;
use crate::level::{ Level, mk_param, mk_zero };
use crate::recursor::{ RecursorVal, RecursorRule };
use crate::env::{ ArcEnv, 
                  ConstantInfo,
                  InductiveVal,
                  ConstructorVal,
                  ensure_no_dupe_lparams };
use crate::tc::TypeChecker;
use crate::expr::{ Expr, 
                   mk_local_declar,
                   mk_local_declar_for,
                   BinderStyle, 
                   InnerExpr::*, 
                   mk_const, 
                   mk_var,
                   mk_sort, 
                   mk_app };
use crate::errors::{ NanodaResult, NanodaErr::* };


// inductive.cpp ~78
#[derive(Debug, Clone)]
pub struct AddInductiveFn {
    //m_lctx : LocalCtx,
    name : Name,
    m_lparams : Vec<Level>,
    m_levels : Vec<Level>, 
    m_nparams : usize,
    m_is_unsafe : bool,
    m_ind_types : Vec<InductiveType>,
    env_handle : ArcEnv,
    m_nindices : Vec<usize>,
    m_result_level : Level,
    m_is_not_zero : Option<bool>, 
    m_params : Vec<Expr>, 
    m_ind_consts : Vec<Expr>, 
    m_elim_level : Level,
    m_K_target : bool,
    m_rec_infos : Vec<RecInfo>,
    use_dep_elim : Option<bool>,
}

impl AddInductiveFn {
    pub fn new(name : Name,
               m_lparams : Vec<Level>,
               m_nparams : usize,
               m_is_unsafe : bool,
               m_ind_types : Vec<InductiveType>,
               env_handle : ArcEnv) -> Self {
        AddInductiveFn {
            name,
            m_lparams,
            m_levels : Vec::new(),
            m_nparams,
            m_is_unsafe,
            m_ind_types,
            m_nindices : Vec::new(),
            m_result_level : mk_zero(),
            m_is_not_zero : None,
            m_params : Vec::new(),
            m_ind_consts : Vec::new(),
            m_elim_level : mk_zero(),
            m_K_target : false,
            m_rec_infos : Vec::new(),
            use_dep_elim : None,
            env_handle
        }
    }

    pub fn env_operator(&mut self) -> NanodaResult<()> {
        ensure_no_dupe_lparams(&self.m_lparams)?;
        self.check_inductive_types()?;
        self.declare_inductive_types()?;
        self.check_constructors()?;
        self.declare_constructors();
        self.init_elim_level()?;
        self.init_K_target()?;
        self.mk_rec_infos()?;
        self.declare_recursors()
    }

    pub fn get_param_type(&self, idx : usize) -> NanodaResult<&Expr> {
        match self.m_params.get(idx).map(|e| e.as_ref()) {
            Some(Local { binder, .. }) => Ok(&binder.ty),
            _ => Err(GetParamTypeErr)
        }
    }

    pub fn use_dep_elim(&self, base_type : &Expr) -> NanodaResult<bool> {
        match base_type.as_ref() {
            Sort { level, .. } => Ok(level.maybe_nonzero()),
            _ => Err(UseDepElimNotSortErr)
        }
    }

    pub fn check_inductive_types(&mut self) -> NanodaResult<()> {
        self.m_levels = self.m_lparams.clone();
        let mut tc = TypeChecker::new(None, self.env_handle.clone());

        // We might potentially have multiple types in the case of
        // mutual declarations.
        for (idx, elem) in self.m_ind_types.iter().enumerate() {
            let mut base_type = elem.type_.clone();
            assert!(!base_type.has_locals());

            // collect level param names for type check.
            // check that the base type is correctly formed.
            tc.check(&base_type, self.m_lparams.clone());

            let mut nindices_counter = 0usize;
            let mut i = 0usize;

            while let Pi { binder, body, .. } = base_type.as_ref() {
                if i < self.m_nparams {
                    if (idx == 0) {
                        let param_ = mk_local_declar_for(&base_type);
                        base_type = body.instantiate(Some(&param_).into_iter());
                        self.m_params.push(param_);
                    } else {
                        let indexed_param = self.m_params.get(i).ok_or_else(|| BadIndexErr(file!(), line!(), i))?;
                        assert!(tc.is_def_eq(&binder.ty, indexed_param.get_local_type()?) == EqShort) ;
                        base_type = body.instantiate(Some(indexed_param).into_iter());
                    }
                    i += 1;
                } else {
                    base_type = body.clone();
                    nindices_counter += 1;
                }
            }

            self.m_nindices.push(nindices_counter);

            if i != self.m_nparams {
                crate::errors::check_inductive_i_neq(line!(), i, self.m_nparams);
            }

            let infd_sort = tc.ensure_sort(&base_type);
            base_type = infd_sort.clone();

            self.use_dep_elim = Some(self.use_dep_elim(&base_type)?);

            if (idx == 0) {
                let result_level = base_type.get_sort_level()?;
                let is_not_zero = result_level.is_nonzero();
                self.m_result_level = result_level.clone();
                self.m_is_not_zero = Some(is_not_zero);
            } else if !(infd_sort.get_sort_level()?.eq_by_antisymm(&self.m_result_level)) {
                crate::errors::mutual_different_universes(line!(), infd_sort.get_sort_level()?, &self.m_result_level);
            }

            let ind_const = mk_const(elem.name.clone(), self.m_levels.clone());
            self.m_ind_consts.push(ind_const);
        }
        assert_eq!(self.m_lparams.len(), self.m_levels.len());
        assert_eq!(self.m_nindices.len(), self.m_ind_types.len());
        assert_eq!(self.m_ind_consts.len(), self.m_ind_types.len());
        assert_eq!(self.m_params.len(), self.m_nparams);
        Ok(())
    }

    pub fn declare_inductive_types(&self) -> NanodaResult<()> {
        for idx in 0..self.m_ind_types.len() {
            let ind_type = self.m_ind_types.get(idx)
                           .ok_or_else(|| BadIndexErr(file!(), line!(), idx))?;

            let inductive_val = InductiveVal::new(
                ind_type.name.clone(),
                self.m_lparams.clone(),
                ind_type.type_.clone(),
                self.m_nparams,
                *self.m_nindices.get(idx).ok_or_else(|| BadIndexErr(file!(), line!(), idx))?,
                get_all_inductive_names(&self.m_ind_types),
                ind_type.constructors.iter().map(|cnstr| cnstr.name.clone()).collect(),
                self.is_rec(),
                self.m_is_unsafe,
                self.is_reflexive()
            );

            self.env_handle.write()
            .add_constant_info(ind_type.name.clone(), ConstantInfo::InductiveInfo(inductive_val))
        }
        Ok(())
    }

    fn is_rec(&self) -> bool {
        let predicate = |e : &Expr| 
        if let Const { name, .. } = e.as_ref() {
            self.m_ind_consts.iter().filter_map(|c| c.get_const_name()).any(|n| n == name)
        } else {
            false
        };

        self.m_ind_types.iter()
            .flat_map(|ind_type| ind_type.constructors.iter())
            .any(|cnstr| {
                let mut cursor = &cnstr.type_;
                while let Pi { binder, body, .. } = cursor.as_ref() {
                    if binder.ty.find_matching(predicate).is_some() {
                        return true
                    }
                    cursor = body;
                }
                false
            })
    }

    // only used as a check in is_reflexive
    // does self.ind_consts contain any other items with the same
    // const name as e?
    fn is_ind_occurrence(&self, _e : &Expr) -> bool {
        if let Const { name, .. } = _e.as_ref() {
            self.m_ind_consts.iter().all(|cnst| {
                cnst.get_const_name().map(|n| n == name).unwrap_or(false)
            })
        } else {
            false
        }
    }


    // return true if the given declaration is reflexive.
    /*
    An inductive type `T` is reflexive if it contains at least one
    constructor that takes as an argument a function returning `T`,
    where `T` is another inductive datatype (possibly `T`) in the same
    mutual declaration
    */ 
    fn is_reflexive(&self) -> bool {
        self.m_ind_types.iter()
        .flat_map(|ind_type| ind_type.constructors.iter())
        .any(|cnstr| {
            let mut cursor = cnstr.type_.clone();

            while let Pi { binder, body, .. } = cursor.as_ref() {
                if binder.ty.is_pi() && self.is_ind_occurrence(&binder.ty) {
                    return true
                } else {
                    let local = mk_local_declar_for(&cursor);
                    cursor = body.instantiate(Some(&local).into_iter());
                }
            }

            false

        })
    }

    pub fn is_valid_ind_app2(&self, t : &Expr, idx : usize) -> bool {
        let (I, args) = t.unfold_apps_rev();
        let cond1 = ((I) != ((&self.m_ind_consts[idx])));
        let cond2 = args.len() != (self.m_nparams + (&self.m_nindices[idx]));
        if (cond1 || cond2) {
            return false
        } else {
            for i in 0..self.m_nparams {
                let cond_check = (self.m_params[i].eq_mod_serial(&args[i]));
                if !cond_check {
                    let _x = &self.m_params[i];
                    let _y = &args[i];

                    return false
                }
            }
        }
        return true
    }

    pub fn is_valid_ind_app(&self, _e : &Expr) -> Option<usize> {
        for idx in 0..self.m_ind_types.len() {
            if self.is_valid_ind_app2(_e, idx) {
                return Some(idx)
            }
        }
        None
    }

    pub fn is_rec_argument(&self, _e : &Expr, tc : &mut TypeChecker) -> Option<usize> {
        let mut cursor = tc.whnf(_e);
        while let Pi { body, .. } = cursor.as_ref() {
            let local = mk_local_declar_for(&cursor);
            let instd = body.instantiate(Some(&local).into_iter());
            cursor = tc.whnf(&instd);
        }

        self.is_valid_ind_app(&cursor)
    }

    pub fn check_positivity(&self, _t : &Expr, cnstr_name : &Name, arg_idx : usize, tc : &mut TypeChecker) -> NanodaResult<()> {
        let whnfd = tc.whnf(_t);
        if !self.is_ind_occurrence(&whnfd) {
            Ok(())
        } else if let Pi { binder, body, .. } = whnfd.as_ref() {
            if self.is_ind_occurrence(&binder.ty) {
                Err(NonposOccurrenceErr(file!(), line!()))
            } else {
                let local = mk_local_declar_for(&whnfd);
                let instd = body.instantiate(Some(&local).into_iter());
                self.check_positivity(&instd, cnstr_name, arg_idx, tc)
            }
        } else if self.is_valid_ind_app(&whnfd).is_some() {
            Ok(())
        } else {
            Err(InvalidOccurrenceErr(file!(), line!()))
        }
    }

    pub fn check_constructors(&self) -> NanodaResult<()> {
        let mut tc = TypeChecker::new(None,  self.env_handle.clone());
        for idx in 0..self.m_ind_types.len() {
            let ind_type = &self.m_ind_types[idx];
            for cnstr in ind_type.constructors.iter() {
                let n = &cnstr.name;
                let mut t = cnstr.type_.clone();
                // FIXME
                // m_env.check_name(n);
                assert!(t.var_bound() == 0);
                tc.check(&t, self.m_lparams.clone());
                let mut i = 0usize;
                while let Pi { binder : dom, body, .. } = t.as_ref() {
                    if i < self.m_nparams {
                        if (tc.is_def_eq(&dom.ty, self.get_param_type(i)?) == NeqShort) {
                            return Err(CnstrBadParamTypeErr)
                        } else {
                            let l = &self.m_params[i];
                            let instd = body.instantiate(Some(l).into_iter());
                            t = instd;
                        }
                    } else {
                        let s = tc.ensure_type(&dom.ty);
                        let cond1 = self.m_result_level.is_geq(s.get_sort_level()?);
                        let cond2 = self.m_result_level.is_zero();

                        if !(cond1 || cond2) {
                            return Err(CnstrUnivErr)
                        }

                        if !self.m_is_unsafe {
                            self.check_positivity(&dom.ty, n, i, &mut tc)?;
                        }

                        let local = mk_local_declar_for(&t);
                        let instd = body.instantiate(Some(&local).into_iter());
                        t = instd;
                    }
                    i += 1;
                }

                if !self.is_valid_ind_app2(&t, idx) {
                    return Err(CnstrBadTypeErr)
                }
            }
        }
        Ok(())
    }


    pub fn declare_constructors(&self) {
        for idx in 0..self.m_ind_types.len() {
            let ind_type = &self.m_ind_types[idx];
            let mut cidx = 0usize;
            for cnstr in ind_type.constructors.iter() {
                let n = cnstr.name.clone();
                let t = cnstr.type_.clone();
                let mut arity = 0usize;
                let mut it = t.clone();
                while let Pi { body, .. } = it.as_ref() {
                    it = body.clone();
                    arity += 1;
                }

                assert!(arity >= self.m_nparams);
                let nfields = arity - self.m_nparams;

                let cval = ConstructorVal::new(
                    n.clone(),
                    self.m_lparams.clone(),
                    t.clone(),
                    ind_type.name.clone(),
                    cidx,
                    self.m_nparams,
                    nfields,
                    self.m_is_unsafe
                );

                self.env_handle.write().add_constant_info(n, ConstantInfo::ConstructorInfo(cval));
                cidx += 1;
            }
        }
    }

    pub fn elim_only_at_universe_zero(&self, tc : &mut TypeChecker) -> NanodaResult<bool> {
        if self.m_is_not_zero
           .ok_or_else(|| NoneErr(file!(), line!(), "elim_only_at_universe_zero::m_is_not_zero"))? {
            return Ok(false)
        }

        if self.m_ind_types.len() > 1 {
            return Ok(true)
        }

        let num_intros = (&self.m_ind_types[0]).constructors.len();

        if num_intros > 1 {
            return Ok(true)
        }

        if num_intros == 0 {
            return Ok(false)
        }

        let mut cnstr_type = (&self.m_ind_types[0].constructors.get(0))
                    .ok_or_else(|| NoneErr(file!(), line!(), "inductive::elim_only_at_univserse_zero::cnstr"))?
                    .type_
                    .clone();

        let mut i = 0usize;
        let mut to_check = Vec::new();

        while let Pi { binder : dom, body, .. } = cnstr_type.as_ref() {
            let fvar = mk_local_declar_for(&cnstr_type);
            if i >= self.m_nparams {
                let s = tc.ensure_type(&dom.ty);
                if (!(s.get_sort_level()?.is_zero())) {
                    to_check.push(fvar.clone());
                }
            }

            let instd = body.instantiate(Some(&fvar).into_iter());
            cnstr_type = instd;
            i += 1;
        }

        let (_, result_args) = cnstr_type.unfold_apps_rev();

        for arg in to_check.iter() {
            if !(result_args.contains(&arg)) {
                return Ok(true)
            }
        }

        Ok(false)
    }

    pub fn init_elim_level(&mut self) -> NanodaResult<()> {
        let mut tc = TypeChecker::new(None, self.env_handle.clone());
        let result = if self.elim_only_at_universe_zero(&mut tc)? {
            self.m_elim_level = mk_zero();
        } else {
            let mut n = Name::from("u");
            let mut counter = 1u64;
            while self.m_lparams.iter().any(|x| x.get_param_name() == &n) {
                n = n.extend_num(counter);
                counter += 1;
            }

            self.m_elim_level = mk_param(n);

        };
        Ok(result)

    }

    // This one doesn't admit punit as a target for k-like reduction since 
    // its result_level is Sort(u). The other ones are eq, heq, and true, which
    // are all m_result_level == Sort(0) AKA Props.
    pub fn init_K_target(&mut self) -> NanodaResult<()> {
        let cond1 = self.m_ind_types.len() == 1;
        let cond2 = self.m_result_level.is_zero();
        let cond3 = (&self.m_ind_types[0]).constructors.len() == 1;
        self.m_K_target = cond1 && cond2 && cond3;

        if (!self.m_K_target) {
            return Ok(())
        } 

        let mut it = (&self.m_ind_types[0])
        .constructors
        .get(0)
        .ok_or_else(|| NoneErr(file!(), line!(), "inductive::init_k_target(0)"))?
        .type_
        .clone();

        let mut i = 0usize;
        while let Pi { body, .. } = it.as_ref() {
            if (i < self.m_nparams) {
                it = body.clone();
            } else {
                self.m_K_target = false;
                break;
            }
            i += 1;
        }
        Ok(())
    }

    pub fn get_I_indices(&self, t : Expr, indices : &mut Vec<Expr>) -> NanodaResult<usize> {
        let r : usize = self.is_valid_ind_app(&t)
                        .ok_or_else(|| NoneErr(file!(), line!(), "inductive::get_I_indices"))?;

        let (_, all_args) = t.unfold_apps_rev();
        for i in self.m_nparams .. all_args.len() {
            indices.push((&all_args[i]).clone().clone());
        }

        Ok(r)
    }

    // This function is horrifying.
    pub fn mk_rec_infos(&mut self) -> NanodaResult<()> {
        let mut tc = TypeChecker::new(None, self.env_handle.clone());
        let mut d_idx = 0usize;

        for ind_type in self.m_ind_types.iter() {
            // FIXME
            let mut rec_info = RecInfo::new(mk_var(0), Vec::new(), Vec::new(), mk_var(0));

            let mut t : Expr = ind_type.type_.clone();
            let mut i = 0usize;

            while let Pi { body, .. } = t.as_ref() {
                if (i < self.m_nparams) {
                    let l = &self.m_params[i];
                    let instd = body.instantiate(Some(l).into_iter());
                    t = instd;
                } else {
                    let idx = mk_local_declar_for(&t);
                    rec_info.m_indices.push(idx.clone());
                    // set m_indices
                    let instd = body.instantiate(Some(&idx).into_iter());
                    t = instd;
                }

                i += 1;
            }


            let _app = (&self.m_ind_consts[d_idx])
                       .foldl_apps(self.m_params.iter())
                       .foldl_apps(rec_info.m_indices.iter());
            let major_local = mk_local_declar(Name::from("t"), _app, BinderStyle::Default);
            rec_info.m_major = major_local;

            let MotiveBase = mk_sort(self.m_elim_level.clone());
            let use_dep_elim_res = self.use_dep_elim.ok_or_else(|| NoneErr(file!(), line!(), "mk_rec_infos::use_dep_elim"))?;
            let MotiveType = if use_dep_elim_res {
                let _x = MotiveBase.fold_pis(Some(&rec_info.m_major).into_iter());
                _x.fold_pis(rec_info.m_indices.iter())
            } else {
                MotiveBase.fold_pis(rec_info.m_indices.iter())
            };

            let mut MotiveName = Name::from("C");
            if self.m_ind_types.len() > 1 {
                MotiveName = MotiveName.extend_num((d_idx + 1) as u64);
            }

            let Motive = mk_local_declar(MotiveName.clone(), MotiveType.clone(), BinderStyle::Implicit);
            rec_info.m_C = Motive.clone();
            self.m_rec_infos.push(rec_info);
            d_idx += 1;
        }

        let mut minor_idx = 1usize;
        d_idx = 0;

        for ind_type in self.m_ind_types.iter() {
            for cnstr in ind_type.constructors.iter() {
                let mut b_u = Vec::<Expr>::new();
                let mut u = Vec::<Expr>::new();
                let mut v = Vec::<Expr>::new();
                let mut t : Expr = cnstr.type_.clone();

                let mut i = 0usize;

                while let Pi { binder : dom, body, .. } = t.as_ref() {
                    if (i < self.m_nparams) {
                        let instd = body.instantiate(Some(&self.m_params[i]).into_iter());
                        t = instd;
                    } else {
                        let l = mk_local_declar_for(&t);
                        b_u.push(l.clone());
                        if self.is_rec_argument(&dom.ty, &mut tc).is_some() {
                            u.push(l.clone());
                        }
                        let instd = body.instantiate(Some(&l).into_iter());
                        t = instd;

                    }
                    i += 1;
                }

                let mut it_indices = Vec::<Expr>::new();

                let it_idx = self.get_I_indices(t.clone(), &mut it_indices)?;

                let use_dep_elim_result = self.use_dep_elim
                                         .ok_or_else(|| NoneErr(file!(), line!(), "inductive::declare_recursors, use_dep_elim_result"))?;

                let MotiveAppBase = (&self.m_rec_infos[it_idx].m_C).foldl_apps(it_indices.iter());
                let MotiveApp = if use_dep_elim_result {
                    let rhs = mk_const(cnstr.name.clone(), self.m_levels.clone())
                              .foldl_apps(self.m_params.iter())
                              .foldl_apps(b_u.iter());

                    mk_app(MotiveAppBase, rhs)
                } else {
                    MotiveAppBase
                };


                for i in 0..u.len() {
                    let u_i = &u[i];
                    let infd = tc.infer_type(&u_i);
                    let mut u_i_ty = tc.whnf(&infd);

                    let mut xs = Vec::new();

                    while let Pi { body, .. } = u_i_ty.as_ref() {
                        let x = mk_local_declar_for(&u_i_ty);
                        xs.push(x.clone());
                        let instd = body.instantiate(Some(&x).into_iter());
                        let whnfd = tc.whnf(&instd);
                        u_i_ty = whnfd;
                    }

                    let mut it_indices = Vec::<Expr>::new();
                    let it_idx = self.get_I_indices(u_i_ty.clone(), &mut it_indices)?;
                    let C_Base = (&self.m_rec_infos[it_idx].m_C).foldl_apps(it_indices.iter());

                    let C_Base2 = if use_dep_elim_result {
                        let u_app = u_i.foldl_apps(xs.iter());
                        mk_app(C_Base, u_app)
                    } else {
                        C_Base
                    };

                    let v_i_ty = C_Base2.fold_pis(xs.iter());
                    let v_i = mk_local_declar(Name::from("v").extend_num(i as u64), v_i_ty.clone(), BinderStyle::Default);
                    v.push(v_i);
                }

                let minor_ty_pre = MotiveApp.fold_pis(v.iter());
                let minor_ty = minor_ty_pre.fold_pis(b_u.iter());
                let minor = mk_local_declar(Name::from("m").extend_num(minor_idx as u64), minor_ty, BinderStyle::Default);
                (&mut self.m_rec_infos[d_idx]).m_minors.push(minor);
                minor_idx += 1;
            }

            d_idx += 1;

        }
        Ok(())
    }

    pub fn get_rec_levels(&self) -> Vec<Level> {
        if self.m_elim_level.is_param() {
            let mut v = Vec::new();
            v.push(self.m_elim_level.clone());
            for l in self.m_levels.clone() {
                v.push(l)
            }
            v
        } else {
            Vec::from(self.m_levels.clone())
        }
    }

// return the level parameter names for the recursor
    pub fn get_rec_lparam_names(&self) -> Vec<Name> {
        if self.m_elim_level.is_param() {
            let mut names = Vec::<Name>::new();
            names.push(self.m_elim_level.get_param_name().clone());
            for l in self.m_lparams.iter() {
                names.push(l.get_param_name().clone())
            }
            names
        } else {
            let mut names = Vec::<Name>::new();
            for l in self.m_lparams.iter() {
                names.push(l.get_param_name().clone())
            }
            names
        }
    }

    pub fn get_rec_lparams(&self) -> Vec<Level> {
        self.get_rec_lparam_names().into_iter().map(|x| mk_param(x)).collect::<Vec<Level>>()
    }

    // These are implemented as `fill a given mutable ref to a vec` functions.
    pub fn collect_Cs(&self) -> Vec<Expr> {
        let mut v = Vec::new();
        for i in 0 .. self.m_ind_types.len() {
            v.push((&self.m_rec_infos[i]).m_C.clone());
        }
        v
    }

    pub fn collect_minor_premises(&self) -> Vec<Expr> {
        let mut v = Vec::new();
        for i in 0 .. self.m_ind_types.len() {
            v.extend((&self.m_rec_infos[i]).m_minors.clone());
        }
        v
    }


    pub fn mk_rec_rules(&self, tc : &mut TypeChecker, d_idx : usize, Cs : &mut Vec<Expr>, minors : &mut Vec<Expr>, mut minor_idx : usize) -> NanodaResult<Vec<RecursorRule>> {
        let d = &self.m_ind_types[d_idx].clone();
        let lvls = self.get_rec_levels();
        let mut rules = Vec::<RecursorRule>::new();

        for cnstr in d.constructors.iter() {
            let mut b_u = Vec::<Expr>::new();
            let mut u = Vec::<Expr>::new();
            let mut t = cnstr.type_.clone();

            let mut i = 0usize;

            while let Pi { binder : dom, body, .. } = t.as_ref() {
                if (i < self.m_nparams) {
                    let instd = body.instantiate(Some(&self.m_params[i]).into_iter());
                    t = instd;
                } else {
                    let l = mk_local_declar_for(&t);
                    b_u.push(l.clone());
                    if (self.is_rec_argument(&dom.ty, tc).is_some()) {
                        u.push(l.clone());
                    }
                    let instd = body.instantiate(Some(&l).into_iter());
                    t = instd
                }

                i += 1;

            }

            let mut v = Vec::<Expr>::new();

            for i in 0..u.len() {
                let u_i = &u[i].clone();
                let infd = tc.infer_type(&u_i);
                let mut u_i_ty = tc.whnf(&infd);

                let mut xs = Vec::<Expr>::new();

                while let Pi { body, .. } = u_i_ty.as_ref() {
                    let x = mk_local_declar_for(&u_i_ty);
                    xs.push(x.clone());
                    let instd = body.instantiate(Some(&x).into_iter());
                    u_i_ty = tc.whnf(&instd);
                }

                let mut it_indices = Vec::<Expr>::new();
                let it_idx = self.get_I_indices(u_i_ty.clone(), &mut it_indices)?;
                let rec_name = (&self.m_ind_types[it_idx]).name.mk_rec_name();

                let rec_app_lhs = mk_const(rec_name, Vec::from(lvls.clone()))
                                  .foldl_apps(self.m_params.iter())
                                  .foldl_apps(Cs.iter())
                                  .foldl_apps(minors.iter())
                                  .foldl_apps(it_indices.iter());
                let rec_app = mk_app(rec_app_lhs, u_i.foldl_apps(xs.iter()));
                v.push(rec_app.fold_lambdas(xs.iter()));
            }



            let comp_rhs = (&minors[minor_idx]).foldl_apps(b_u.iter())
                            .foldl_apps(v.iter())
                            .fold_lambdas(b_u.iter())
                            .fold_lambdas(minors.iter())
                            .fold_lambdas(Cs.iter())
                            .fold_lambdas(self.m_params.iter());

            let rec_rule = RecursorRule::new(cnstr.name.clone(), b_u.len(), comp_rhs);

            rules.push(rec_rule);
            minor_idx += 1;
        }


        Ok(rules)

    }

    pub fn get_all_inductive_names(&self) -> Vec<Name> {
        let mut v = Vec::new();
        for elem in self.m_ind_types.iter() {
            v.push(elem.name.clone());
        }
        v
    }

    pub fn declare_recursors(&self) -> NanodaResult<()> {
        let mut tc = TypeChecker::new(None, self.env_handle.clone());

        let mut Cs = self.collect_Cs();
        let mut minors = self.collect_minor_premises();

        let nminors = minors.len();
        let nmotives = Cs.len();

        let all : Vec<Name> = self.get_all_inductive_names();

        let minor_idx = 0usize;

        for d_idx in 0..(self.m_ind_types.len()) {
            let use_dep_elim_result = self.use_dep_elim
                                     .ok_or_else(|| NoneErr(file!(), line!(), "inductive::declare_recursors, use_dep_elim_result"))?;

            let info = &self.m_rec_infos[d_idx].clone();

            let MotiveAppBase = info.m_C.foldl_apps(info.m_indices.iter());

            let MotiveApp = if use_dep_elim_result {
                mk_app(MotiveAppBase, info.m_major.clone())
            } else {
                MotiveAppBase
            };

            let rec_ty = MotiveApp.fold_pis(Some(&info.m_major).into_iter())
                         .fold_pis(info.m_indices.iter())
                         .fold_pis(minors.iter())
                         .fold_pis(Cs.iter())
                         .fold_pis(self.m_params.iter());

            //// This is unused (by the kernel) apparently.
            //let rec_ty = rec_ty.infer_implicit(true);
            let rules = self.mk_rec_rules(&mut tc, d_idx, &mut Cs, &mut minors, minor_idx)?;
            let rec_name = (&self.m_ind_types[d_idx].name).mk_rec_name();

            let recursor_val = RecursorVal::new(
                rec_name.clone(),
                self.get_rec_lparams(),
                self.get_rec_lparam_names(),
                rec_ty.clone(),
                all.clone(),
                self.m_nparams.clone(),
                self.m_nindices[d_idx],
                nmotives,
                nminors,
                rules,
                self.m_K_target,
                self.m_is_unsafe,
            );

            tc.env.write().add_constant_info(rec_name, ConstantInfo::RecursorInfo(recursor_val));
        }
        Ok(())
    }
}



/*
#[derive(Clone, Debug)]
pub struct ElimNestedInductiveResult {
    pub m_params : Vec<Expr>,
    pub m_aux2nested : HashMap<Name, Expr>,
    pub m_aux_decl : Option<DeclarationKind>,
}



impl ElimNestedInductiveResult {
    pub fn new() -> Self {
        ElimNestedInductiveResult {
            m_params : Vec::new(),
            m_aux2nested : HashMap::new(),
            m_aux_decl : None
        }
    }

    pub fn elim_nested_inductive_result(params : Vec<Expr>, nested_aux : Vec<(Name, Expr)>, d : DeclarationKind) -> Self {
        let mut map = HashMap::new();
        for (n, e) in nested_aux.into_iter() {
            map.insert(n, e);
        }
        ElimNestedInductiveResult {
            m_params : params,
            m_aux2nested : map,
            m_aux_decl : Some(d)
        }
    }

// PROBLEM : There may be some issues with induct/constructor names getting mixed up.
// if ...
// 1. The name `c` is mapped in the current environment to an inductive constructor
// AND
// 2. in the m_aux2nested <Name |-> Expr> mapping, `c`'s base name is mapped to something,
// THEN return the base name of the inductive type and the nested Expr
// ELSE return None if any of the conditions fail.
// From the C++ docs :
// If `c` is an constructor name associated with an auxiliary inductive type, 
// then return the nested inductive associated with it and 
// the name of its inductive type.
// c : <base>.mk
    pub fn get_nested_if_aux_constructor(&self, aux_env : &ArcEnv, c : &Name) -> Option<(Name, Expr)> {
        match aux_env.read().get_constant_info(c) {
            Some(ConstantInfo::ConstructorInfo(cnstr_val)) => {
                let auxI_base_name = &cnstr_val.induct;
                // .induct is the base name
                match self.m_aux2nested.get(auxI_base_name) {
                    Some(nested) => {
                        // base inductive name (no .mk)
                        Some((auxI_base_name.clone(), nested.clone()))
                    },
                    None => return None
                }
            },
            _ => return None
        }
    }

// let new_cnstr_name = res.restore_constructor_name(aux_env, cnstr_name);
// where cnstr_name is <base>.mk

// gets called with tne name field of a RecursorRule, which is of the form
// <base>.mk
    pub fn restore_constructor_name(&mut self, aux_env : &ArcEnv, cnstr_name : &Name) -> Name {
        match self.get_nested_if_aux_constructor(aux_env, cnstr_name) {
            None => panic!("bad `None` @ restore_constructor_name"),
            // I would assume this is also <aux_base>.mk
            Some((base_name1, e)) => {
                match e.get_app_fn().as_ref() {
                    Const(_, n2, _) => {
                        cnstr_name.replace_prefix(&base_name1, n2)
                    },
                    _ => panic!("should have been Const in restore_constructor_name!")
                }
            }
        }
    }

// The two parts of this that are NOT checked
// and need to be are `back()` and instantiate_rev
    pub fn restore_nested(&self, original_e : &Expr, aux_env : &ArcEnv, aux_rec_map : &HashMap<Name, Name>, ) -> Expr {
        // let aux_rec_map : HashMap<Name, Name> = HashMap::new();
        let mut e = original_e.clone();
        let mut As = Vec::new();

        let pi = e.is_pi();

        for i in 0..self.m_params.len() {
            assert!(e.is_pi() || e.is_lambda());
            let binding_body = match e.as_ref() {
                Pi {.., body) | Lambda(.., body) => body,
                _ => panic!("restore_nested loop requires lambda or pi")
            };
            As.push(e.mk_local_declar_auto());
            let As_back = As.back();
            assert!(As_back.is_some());
            e = binding_body.instantiate(As_back.into_iter());
        }

        let f = |t : &Expr| {

            if let (Const(_, n, lvls), Some(rec_name)) = (t.as_ref(), t.get_const_name_opt().and_then(|x| aux_rec_map.get(x))) {
                Some(mk_const(rec_name.clone(), lvls.as_ref().clone()))
            } else {
                let fn_ = t.get_app_fn();
                if let Const(_, n, vals) = fn_.as_ref() {
                    if let Some(nested) = self.m_aux2nested.get(n) {
                        let (new_t, args) = t.unfold_apps_rev();
                        assert!(args.len() >= self.m_params.len());
                        // FIXME not sure if this needs to be reversed
                        let abstrd = nested.abstract_(self.m_params.iter());
                        // not sure how either inst_rev or iter need to be oriented;
                        let new_t = abstrd.instantiate_rev(As.iter());

                        let num_args = args.len() - self.m_params.len();
                        let slice = args.iter().skip(self.m_params.len()).collect::<Vec<&Expr>>();
                        Some(new_t.mk_app_ptr(num_args, slice))
                    } else if let Some((auxI_name, nested)) = self.get_nested_if_aux_constructor(aux_env, n) {
                        let (new_t, args) = t.unfold_apps_rev();
                        assert!(args.len() >= self.m_params.len());
                        let abstrd = nested.abstract_(self.m_params.iter()); 
                        let instd = abstrd.instantiate_rev(As.iter());
                        let (I, I_args) = instd.unfold_apps_rev();
                        let I_args_vec = I_args.iter().collect::<Vec<&Expr>>();

                        assert!(I.is_const());
                        let new_fn_name = n.replace_prefix(&auxI_name, &I.get_const_name());
                        let new_fn = mk_const(new_fn_name, I.get_const_levels());
                        let inner_t = new_fn.mk_app_all(I_args_vec);

                        let ptr_range = args.len() - self.m_params.len();
                        let slice = args.iter().skip(self.m_params.len()).collect::<Vec<&Expr>>();
                        let new_t = inner_t.mk_app_ptr(ptr_range, slice);
                        Some(new_t)
                    } else {
                        eprintln!("This part is unimplemented bcecause I'm not sure if it should return None or what. line : {}", line!());
                        std::process::exit(-1);
                    }
                } else {
                    None
                }
            }

        };

        e = e.replace_expr(f);

        if pi {
            e.fold_pis(As.iter())
        } else {
            e.fold_lambdas(As.iter())
        }
    }
}

#[derive(Clone)]
pub struct ElimNestedInductiveFn {
    m_env : ArcEnv,
    m_d : DeclarationKind,
    m_params : Vec<Expr>,
    m_nested_aux : Vec<(Name, Expr)>,
    m_lvls : Vec<Level>,
    m_new_types : Vec<InductiveType>,
    m_next_idx : usize,
}

impl ElimNestedInductiveFn {
    pub fn new(env : &ArcEnv, d : DeclarationKind) -> Self {
        let m_lvls = Vec::from(d.get_lparams());

        ElimNestedInductiveFn {
            m_env : env.clone(),
            m_d : d,
            m_params : Vec::new(),
            m_nested_aux : Vec::new(),
            m_lvls,
            m_new_types : Vec::new(),
            m_next_idx : 0,
        }
    }

    fn mk_unique_name(&mut self, n : &Name) -> Name {
        loop {
            let r : Name = n.extend_num(self.m_next_idx as u64);
            self.m_next_idx += 1;
            if ((self.m_env.read().declarations.contains_key(&r)) || (self.m_env.read().constant_infos.contains_key(&r))) {
                continue
            } else {
                return r
            }
        }
    }

    pub fn replace_params(&self, e : &Expr, apps : Vec<Expr>) -> Expr {
        assert!(self.m_params.len() == apps.len());
        let abstrd = e.abstract_(apps.iter());
        // FIXME this is instantiate_rev in the source.
        let instd = abstrd.instantiate(self.m_params.iter().rev());
        instd
    }

    pub fn is_nested_inductive_app(&self, e : &Expr) -> Option<InductiveVal> {
        if !(e.is_app()) {
            return None
        }

        let _fn = e.get_app_fn();

        if !(e.is_const()) {
            return None
        }

        //let info = self.m_env.read().const_vals.get(&_fn.get_const_name())?.clone();
        let info = self.m_env.read().get_constant_info(&_fn.get_const_name())?.clone();

        let (nparams, inductive_val) = match &info {
            ConstantInfo::InductiveInfo(induct_val) => (induct_val.nparams, induct_val.clone()),
            _ => return None
        };

        let (e, args) = e.unfold_apps_rev();

        if nparams > args.len() {
            return None
        }

        let mut is_nested = false;
        let mut loose_bvars = false;

        for i in 0..nparams {
            if (args[i].has_locals()) {
                loose_bvars = true;
            }

            let pred = |e : &Expr| {
                match e.as_ref() {
                    Const(_, n, _) => {
                        self.m_new_types.iter().any(|x| &x.id_name == n)
                    },
                    _ => false
                }
            };

            let find_result = args[i].find_matching(pred);
            if find_result.is_some() {
                is_nested = true;
            }
        }

        if (!is_nested) {
            return None
        }

        if (loose_bvars) {
            panic!("Invalid nested inductive datatype {:#?}; nested inductive parameters cannot contain locals", info)
        }

        Some(inductive_val)
    }

    pub fn instantiate_pi_params(&self, mut e : &Expr, nparams : usize, params : Vec<Expr>) -> Expr {
        for i in 0..nparams {
            match e.as_ref() {
                Pi {.., body) => {
                    e = body;
                },
                _ => panic!("Throw ill formed (not pi)")
            }
        }

        // FIXME source is instantiate_rev
        e.instantiate(params.iter().take(nparams))
    }

    pub fn replace_if_nested(&mut self, e : &Expr, As : &Vec<Expr>) -> Option<Expr> {
        let I_val = self.is_nested_inductive_app(e)?;

        let (_fn, args) = e.unfold_apps_rev();
        let I_name = _fn.get_const_name();
        let I_lvls = _fn.get_const_levels();
        assert!(I_val.nparams <= args.len());

        let I_nparams = I_val.nparams;

        let IAs = _fn.foldl_apps(args.iter().take(I_nparams));

        let Iparams = self.replace_params(&IAs, As.clone());

        let mut auxI_name : Option<Name> = None;

        for (n_, e_)  in self.m_nested_aux.iter() {
            if e_ == &Iparams {
                auxI_name = Some(n_.clone());
                break
            }
        }

        if let Some(n__) = auxI_name {
            let mut auxI = mk_const(n__, Vec::from(self.m_lvls.clone()));
            auxI = auxI.mk_app_all(As.iter().collect::<Vec<&Expr>>());
            let retval = auxI.mk_app_ptr(args.len() - I_nparams, args.iter().skip(I_nparams).collect::<Vec<&Expr>>());
            Some(retval)
        } else {
            let mut res : Option<Expr> = None;

            for J_name in I_val.all.iter() {
                let const_info = self.m_env.read().get_constant_info(J_name).cloned().expect("asopdfij");
                let J_info = match const_info {
                    ConstantInfo::InductiveInfo(ind_val) => ind_val,
                    _ => panic!("Should have been an InductiveVal")
                };

                let J = mk_const(J_name.clone(), I_lvls.clone());
                let JAs = J.mk_app_ptr(I_nparams, args.iter().collect::<Vec<&Expr>>());
                let auxJ_name = self.mk_unique_name(&Name::from("_nested").concat(J_name));
                let params_vec = J_info.constant_val.lparams.iter().cloned().zip(I_lvls.iter().cloned()).collect::<Vec<(Level, Level)>>();
                let mut auxJ_type = (&J_info.constant_val.type_).instantiate_lparams(&params_vec);
                auxJ_type = self.instantiate_pi_params(&auxJ_type, I_nparams, Vec::from(args.clone()));
                auxJ_type = auxJ_type.fold_pis(Vec::from(As.clone()).iter());
                let replaced = self.replace_params(&JAs, As.clone());
                self.m_nested_aux.push((auxJ_name.clone(), replaced));

                if (J_name == &I_name) {
                    let mut auxI = mk_const(auxJ_name.clone(), Vec::from(self.m_lvls.clone()));
                    auxI = auxI.mk_app_all(As.iter().collect::<Vec<&Expr>>());
                    res = Some(auxI.mk_app_ptr(args.len() - I_nparams, args.iter().skip(I_nparams).collect::<Vec<&Expr>>()));
                }

                let mut auxJ_constructors = Vec::new();

                // : &Name
                for J_cnstr_name in J_info.cnstrs.iter() {
                    let J_cnstr_info = match self.m_env.read().get_constant_info(J_cnstr_name) {
                        Some(ConstantInfo::ConstructorInfo(constructor_val)) => constructor_val.clone(),
                        _ => panic!("Should have been cosntructor")
                    };

                    let auxJ_cnstr_name = J_cnstr_name.replace_prefix(J_name, &auxJ_name);
                    let lvl_subs = J_cnstr_info.constant_val.lparams.iter().cloned().zip(I_lvls.iter().cloned()).collect::<Vec<(Level, Level)>>();
                    let mut auxJ_cnstr_type = (&J_cnstr_info.constant_val.type_).instantiate_lparams(&lvl_subs);
                    auxJ_cnstr_type = self.instantiate_pi_params(&auxJ_cnstr_type, I_nparams, Vec::from(args.clone()));
                    auxJ_cnstr_type = auxJ_cnstr_type.fold_pis(Vec::from(As.clone()).iter());
                    auxJ_constructors.push(Constructor::new(&auxJ_cnstr_name, &auxJ_cnstr_type));
                }

                let new_ind_type = InductiveType::new(auxJ_name, auxJ_type, auxJ_constructors);
                self.m_new_types.push(new_ind_type);


            }
            assert!(res.is_some());
            res
        }
    }

    pub fn repalce_all_nested(&mut self, e : &Expr, As : &Vec<Expr>) -> Expr {

        let mut cache = crate::expr::OffsetCache::new();
        self.replace_all_nested_core(e, As, 0usize, &mut cache)
    } 

    pub fn replace_all_nested_core(&mut self, e_orig : &Expr, As : &Vec<Expr>, offset : usize, cache : &mut crate::expr::OffsetCache) -> Expr {
        if let Some(cached) = cache.get(e_orig, offset) {
            return cached.clone()
        } else if let Some(e) = self.replace_if_nested(e_orig, As) {
            cache.insert(e_orig.clone(), e.clone(), offset);
            e
        } else {
            let cache_key = e_orig.clone();

            let result = match e_orig.as_ref()  {
                App(_, lhs, rhs) => {
                    let new_lhs = self.replace_all_nested_core(lhs, As, offset, cache);
                    let new_rhs = self.replace_all_nested_core(rhs, As, offset, cache);
                    mk_app(new_lhs, new_rhs)
                },
                | Lambda(_, dom, body) => {
                    let new_dom_ty = self.replace_all_nested_core(&dom.ty, As, offset, cache);
                    let new_body = self.replace_all_nested_core(body, As, offset + 1, cache);
                    crate::expr::mk_lambda(dom.swap_ty(new_dom_ty), new_body)
                }
                | Pi {_, dom, body) => {
                    let new_dom_ty = self.replace_all_nested_core(&dom.ty, As, offset, cache);
                    let new_body = self.replace_all_nested_core(body, As, offset + 1, cache);
                    crate::expr::mk_pi(dom.swap_ty(new_dom_ty), new_body)
                },
                Let(_, dom, val, body) => {
                    let new_dom_ty = self.replace_all_nested_core(&dom.ty, As, offset, cache);
                    let new_val = self.replace_all_nested_core(val, As, offset, cache);
                    let new_body = self.replace_all_nested_core(body, As, offset + 1, cache);
                    crate::expr::mk_let(dom.swap_ty(new_dom_ty), new_val, new_body)
                },
                // Not sure if this is supposed to keep the same serial or not.
                Local {.., serial, binder) => {
                    let new_binder_ty = self.replace_all_nested_core(&binder.ty, As, offset, cache);
                    crate::expr::mk_local(binder.pp_name.clone(), new_binder_ty, binder.style)
                    //mk_local_w_serial(*serial, bind, new_bind_ty)
                },
                Var(..) | Sort(..) | Const(..) => e_orig.clone()
            };

            cache.insert(cache_key, result.clone(), offset);

            result

        }
    }

    pub fn get_params(&mut self, mut type_ : Expr, nparams : usize, mut params : Vec<Expr>) -> Expr {
        assert!(params.is_empty());
        for i in 0..nparams {
            match type_.as_ref() {
                Pi {_, dom, body) => {
                    let this_local = mk_local_declar(dom.pp_name.clone(), dom.ty.clone(), dom.style);
                    params.push(this_local);
                    let back = params.back();
                    assert!(back.is_some());
                    type_ = body.instantiate(back.into_iter());
                }
                _ => panic!("Should have been pi; more details on error in C++"),
            }
        }

        type_
    }

}


fn mk_motive_app(e : &Expr, indices : Vec<&Expr>, motive : &Expr, use_dep_elim : Option<bool>) -> Expr {
    let use_dep_elim = use_dep_elim.expect("use dep elim should not be none");
    if use_dep_elim {
        mk_app(motive.foldl_apps(indices.into_iter()), e.clone())
    } else {
        motive.foldl_apps(Vec::from(indices))
    }
}


*/



//impl std::cmp::PartialEq for AddInductiveFn {
//    fn eq(&self, other : &AddInductiveFn) -> bool {
//
//        let recs_eq = self.m_rec_infos.iter().zip(other.m_rec_infos.iter()).all(|(inf1, inf2)| {
//            inf1.m_C.eq_mod_locals(&inf2.m_C)
//            && inf1.m_minors.iter().zip(inf2.m_minors.iter()).all(|(x, y)| x.eq_mod_locals(y))
//            && inf1.m_indices.iter().zip(inf2.m_indices.iter()).all(|(x, y)| x.eq_mod_locals(y))
//            && inf1.m_major.eq_mod_locals(&inf2.m_major)
//        });
//        (self.name == other.name)
//        && (&self.m_lparams == &other.m_lparams)
//        && (&self.m_levels == &other.m_levels)
//        && (&self.m_nparams == &other.m_nparams)
//        && (&self.m_is_unsafe == &other.m_is_unsafe)
//        && (&self.m_nindices == &other.m_nindices)
//        && (&self.m_result_level == &other.m_result_level)
//        && (&self.m_is_not_zero == &other.m_is_not_zero)
//        && (self.m_params.iter().zip(other.m_params.iter()).all(|(x, y)| x.eq_mod_locals(y)))
//        && (&self.m_elim_level == &other.m_elim_level)
//        && (&self.m_K_target == &other.m_K_target)
//        && (&self.use_dep_elim == &other.use_dep_elim)
//        && (self.m_ind_types.iter().zip(other.m_ind_types.iter()).all(|(x, y)| x.type_.eq_mod_locals(&y.type_)))
//        && (self.m_ind_consts.iter().zip(other.m_ind_consts.iter()).all(|(x, y)| x.eq_mod_locals(y)))
//        && recs_eq
//    }
//}
//
//impl std::cmp::Eq for AddInductiveFn {}

