use std::sync::Arc;

use hashbrown::HashSet;
use parking_lot::RwLock;

use crate::seq;
use crate::name::Name;
use crate::level::{ Level, mk_param, mk_zero };
use crate::reduction::ReductionRule;
use crate::env::{ Env, Declaration, CompiledModification };
use crate::tc::TypeChecker;
use crate::utils::{ Either, Either::* };
use crate::errors;
use crate::expr::{ Expr, 
                   Binding, 
                   BinderStyle, 
                   InnerExpr::*, 
                   mk_const, 
                   mk_sort, 
                   mk_local, 
                   mk_app };


/// This module implements inductive types. The general flow is:
/// 1. The parser collects the elements needed to call `Inductive::new(..)`
/// 2. Eventually we call `compile()` on the thing from 1
/// 3. `compile()` creates the `Inductive`'s related declarations,
///    introduction, and elimination rules. Introduction rules have
///    two instances; one is `CompiledIntro`, which has intermediate info
///    used for checking the introduction, and the other is the persistent
///    `Declaration` spun off from an introduction rule that actually persists
///    in the environment after typechecking is done, and is a member
///    of the eventual `CompiledInductive`.
///    Formation of the reduction rules is done by the constructors for
///    `ReductionRule`, though there is special handling for k-like reduction
///    which is discussed more below.
/// 
/// Many of the functions defined on `Inductive` are just defined to 
/// pull them out of the body of `compile()` to keep it from just being
/// a giant list of instructions. Most of them are only called once per
/// inductive and could just as easily be placed inline.


#[derive(Debug, Clone)]
pub struct ProtoInd {
    pub name: Name,
    pub params: Arc<Vec<Level>>,
    pub ty: Expr,
    pub num_params: usize,
    pub intros: Vec<(Name, Expr)>,
}

#[derive(Debug, Clone)]
pub struct Inductive {
    pub num_params: usize,
    pub intros: Vec<(Name, Expr)>,
    pub tc : Arc<RwLock<TypeChecker>>,
    pub codomain_sort : Level,
    pub params_and_indices : Vec<Expr>,
    pub use_dep_elim : bool,
    pub minimal_const : Expr,
    pub base_declaration: Declaration,
}

impl Inductive {
    pub fn new(name: Name,
               univ_params: Arc<Vec<Level>>,
               type_: Expr,
               num_params: usize,
               intros: Vec<(Name, Expr)>,
               env : Arc<RwLock<Env>>) -> Self {

        let minimal_const = mk_const(name.clone(), univ_params.clone());
        let base_declaration = Declaration::mk(name, univ_params, type_, None, Some(true));

        let mut tc = TypeChecker::new(None, env);

        base_declaration.to_axiom().compile(&tc.env).add_only(&tc.env);

        let (codomain_expr, params_and_indices) = tc.normalize_pis(&base_declaration.ty);
        let codomain_sort = match codomain_expr.as_ref() {
            Sort(_, lvl) => lvl.clone(),
            owise => errors::err_normalize_pis(line!(), owise)
        };

        let use_dep_elim = codomain_sort.maybe_nonzero();

        Inductive {
            num_params,
            intros,
            tc : Arc::new(RwLock::new(tc)),
            codomain_sort,
            params_and_indices,
            use_dep_elim,
            minimal_const,
            base_declaration,
        }
    }

    pub fn map_tc<T>(&self, f : impl FnOnce(&mut TypeChecker) -> T) -> T {
        f(&mut self.tc.write())
    }

    pub fn get_params(&self) -> &[Expr] {
        &self.params_and_indices[0 .. self.num_params]
    }

    pub fn get_indices(&self) -> &[Expr] {
        &self.params_and_indices[self.num_params .. ]
    }


    pub fn get_name(&self) -> &Name {
        &self.base_declaration.name
    }

    pub fn get_univ_params(&self) -> &Vec<Level> {
        &self.base_declaration.univ_params.as_ref()
    }

    pub fn get_type(&self) -> &Expr {
        &self.base_declaration.ty
    }

    pub fn elim_into_prop(&self, compiled_intros : &Vec<CompiledIntro>) -> bool {
        let bool1 = self.codomain_sort.maybe_zero();
        let bool2 = self.intros.len() > 1;
        let bool3 = compiled_intros.iter().any(|intro| {
            intro.intro_arguments.iter().any(|arg| {
                !self.map_tc(|tc| tc.is_proof(arg).0) && !intro.intro_type_args.contains(arg)
            })
        });

        bool1 && (bool2 || bool3)
    }

    pub fn elim_level(&self, compiled_intros : &Vec<CompiledIntro>) -> Level {
        if self.elim_into_prop(&compiled_intros) {
            mk_zero()
        } else {
            let forbidden_names = self.get_univ_params()
                                      .iter()
                                      .map(|x| x.get_param_name())
                                      .collect::<HashSet<&Name>>();
            let fresh_name = Name::fresh_name("l", forbidden_names);
            mk_param(fresh_name)
        }
    }

    pub fn elim_level_params(&self, elim_level : &Level) ->  Arc<Vec<Level>> {
        if elim_level.is_param() {
            let lvls = seq![Some(elim_level).into_iter(), self.get_univ_params().iter()];
            Arc::new(lvls)
        } else {
            Arc::new(self.get_univ_params().clone())
        }
    }

    pub fn mk_motive_app(&self, e : &Expr, indices : &[Expr], motive : &Expr) -> Expr {
        if self.use_dep_elim {
            mk_app(motive.fold_apps(indices.into_iter()), e.clone())
        } else {
            motive.fold_apps(indices.iter())
        }
    }

    pub fn compile(self, env : &Arc<RwLock<Env>>) -> CompiledModification {


        let base_type_folded_w_params = (&self.minimal_const).fold_apps(self.get_params().into_iter());
        let base_type_folded_w_params_and_indices = &self.minimal_const.fold_apps(self.params_and_indices.iter());

        let compiled_intros = 
            self.intros.iter().map(|(intro_name, raw_intro_type)| {
                CompiledIntro::new(&self,
                                   raw_intro_type,
                                   intro_name,
                                   &base_type_folded_w_params)
            }).collect::<Vec<CompiledIntro>>();


        let elim_level = self.elim_level(&compiled_intros);
        let elim_level_params = self.elim_level_params(&elim_level);
        let sort_of_elim_lvl = mk_sort(elim_level);

        let motive_type = if self.use_dep_elim {
            let lc = mk_local(Name::from("c"), 
                                    base_type_folded_w_params_and_indices.clone(), 
                                    BinderStyle::Default);
            sort_of_elim_lvl.fold_pis(self.get_indices().into_iter().chain(Some(&lc)))
        } else {
            sort_of_elim_lvl.fold_pis(self.get_indices().into_iter())
        };

        let motive = mk_local(Name::from("C"), motive_type, BinderStyle::Implicit);

        // Motive is the reason why you can't set it from the start.
        let intro_minors = compiled_intros.iter().map(|intro| {
            intro.mk_intro_minor_premise(&motive)
        }).collect::<Vec<Expr>>();

        let major_premise = 
            mk_local(Name::from("x"),
                           base_type_folded_w_params_and_indices.clone(),
                           BinderStyle::Default);

        let elim_type_args = seq![&self.get_params(),
                            Some(&motive),
                            &intro_minors, 
                            &self.get_indices(), 
                            Some(&major_premise)];

        let elim_type = self.mk_motive_app(&major_premise,
                                           self.get_indices(),
                                           &motive).fold_pis(elim_type_args.iter());

        let elim_declaration = Declaration::mk(
                                        self.get_name().extend_str("rec"),
                                        elim_level_params.clone(),
                                        elim_type,
                                        None,
                                        Some(true)
                                    );

        // The 'flag' for whether you're going to end up using a k value is :
        // `compiled_intros` has only one element `e`,
        // AND the intro_arguments of `e` are empty
        let detect_k = compiled_intros.len() == 1 
                       && compiled_intros.get(0)
                                         .map(|intro| intro.intro_arguments.is_empty())
                                         .unwrap_or(false);

        let k_intro_rule = if detect_k {
            let k_intro_arg2 = 
                mk_const(elim_declaration.name.clone(), 
                               elim_level_params.clone())
                                   .fold_apps(elim_type_args.iter());
        
            let k_intro_arg3 = intro_minors[0].clone();
        
            let k_intro_arg4 = compiled_intros[0]
                                    .intro_type_args
                                    .iter()
                                    .zip(self.params_and_indices.iter())
                                    .filter(|(a, b)| a != b);

            let rr = ReductionRule::new_nondef_rr(
                elim_type_args.as_slice(),
                k_intro_arg2,
                k_intro_arg3,
                k_intro_arg4,
            );
            Some(rr)
         } else {
              None
         };


        let intro_declarations = 
            compiled_intros
            .iter()
            .map(|intro| {
                Declaration::mk(
                    intro.intro_name.clone(),
                    Arc::new(self.get_univ_params().clone()),
                    intro.raw_type.clone(),
                    None,
                    Some(true)
                )
            }).collect::<Vec<Declaration>>();

        let reduction_rules = if let Some(k_intro) = k_intro_rule {
            vec![k_intro]
        } else {
            compiled_intros.iter()
                           .enumerate()
                           .map(|(intro_minors_idx, intro)| intro.mk_reduction_rule(
                intro_minors_idx,
                &intro_minors,
                &motive,
                &elim_declaration.name,
                &elim_declaration.univ_params,
            )).collect::<Vec<ReductionRule>>()
        };

        for i in compiled_intros.iter() {
            i.check_intro(env)
        }

        // We want to be able to drop non-essential
        // info about the original inductive and intro rules
        // before we reach the function boundary and return 
        // the `CompiledInductive` item. This is also what lets
        // us take `parent` by reference in CompiledIntro.

        CompiledModification::CompiledInductive(self.base_declaration,
                                                intro_declarations,
                                                elim_declaration,
                                                reduction_rules)
    }
}


#[derive(Debug)]
pub struct CompiledIntro<'p> {
    pub parent : &'p Inductive,
    pub intro_name : Name,
    pub intro_arguments : Vec<Expr>,
    pub intro_type : Expr,
    pub raw_type : Expr,
    pub intro_arg_data : Vec<ArgData>,
    pub intro_type_args : Vec<Expr>,
}

type ArgData = Either<Expr, (Vec<Expr>, Vec<Expr>)>;

impl<'p> CompiledIntro<'p> {
    pub fn new(parent : &'p Inductive,
               raw_intro_type : &Expr,
               intro_name : &Name,
               ind_ty_w_params : &Expr) -> Self {

        let instd_pi = parent.map_tc(|tc| tc.instantiate_pis(raw_intro_type, parent.get_params()));
        let (fn_f, arguments) = parent.map_tc(|tc| tc.normalize_pis(&instd_pi));
        let (new_intro_type, intro_type_args) = fn_f.unfold_apps_special();

        let all_arg_infos = arguments.iter().map(|arg| {
            if let Local(.., binding) = arg.as_ref() {
                let (fn_, binders) = parent.map_tc(|tc| tc.normalize_pis(&binding.ty));
                let (rec_arg_ind_ty, rec_args) = fn_.unfold_apps_special();

                match rec_arg_ind_ty.as_ref() {
                    Const(_, name, _) if name == parent.get_name() => {
                        assert!(rec_args.len() >= parent.num_params);
                        let (rec_args_lhs, rec_args_rhs) = rec_args.split_at(parent.num_params);
                        parent.map_tc(|tc| {
                            tc.require_def_eq(&rec_arg_ind_ty.fold_apps(rec_args_lhs), 
                                              ind_ty_w_params);
                        });
                        Right((binders, rec_args_rhs.to_vec()))
                    },
                    _ => Left(arg.clone())
                }
            } else {
                Left(arg.clone())
            }
        }).collect::<Vec<ArgData>>();

        let compiled_intro = CompiledIntro {
            parent,
            intro_name : intro_name.clone(),
            intro_arguments : arguments,
            intro_type : new_intro_type,
            raw_type : raw_intro_type.clone(),
            intro_arg_data : all_arg_infos,
            intro_type_args : intro_type_args,
        };

        compiled_intro

    }

    // Create a declaration's inductive hypotheses
    pub fn ihs(&self, motive : &Expr) -> Vec<Expr> {
        self.intro_arguments.iter().zip(&self.intro_arg_data).filter_map(|(a, b)| {
            match b {
                Right((v1, v2)) => {
                    let apps = a.fold_apps(v1);
                    let motive_app = self.parent.mk_motive_app(&apps, &v2, &motive);
                    let pis = motive_app.fold_pis(v1.iter());
                    Some(mk_local(Name::from("ih"), pis, BinderStyle::Default))
                },
                _ => None
            }
        }).collect()
    }

    pub fn mk_intro_minor_premise(&self, motive : &Expr) -> Expr {
        let params_and_args = seq![self.parent.get_params(), &self.intro_arguments];
        let lhs_const = mk_const(self.intro_name.clone(), self.parent.get_univ_params().clone());
        let lhs_app = lhs_const.fold_apps(params_and_args.iter());
        let motive_app = self.parent.mk_motive_app(&lhs_app,
                                          &self.intro_type_args[self.parent.num_params..],
                                          &motive);
        let args_and_ihs = seq![&self.intro_arguments, self.ihs(motive)];
        let pis = motive_app.fold_pis(args_and_ihs.iter());
        let hypothesis_binding = Binding::mk(Name::from("h"), pis, BinderStyle::Default);
        hypothesis_binding.as_local()
    }


    pub fn recursive_calls(&self, 
                           motive : &Expr, 
                           minor_premises : &Vec<Expr>,
                           elim_declar_name : &Name,
                           elim_level_params : &Vec<Level>) -> Vec<Expr> {
        let mut results_vec = Vec::with_capacity(self.intro_arguments.len().max(self.intro_arg_data.len()));

        for (rec_arg, x) in self.intro_arguments.clone().into_iter().zip(self.intro_arg_data.clone()) {
            match x {
                Right((eps, rec_arg_indices)) => {
                    let apps_rhs = seq![self.parent.get_params(),
                                        Some(motive),
                                        &minor_premises,
                                        &rec_arg_indices,
                                        Some(rec_arg.fold_apps(eps.iter()))];
                    let apps_lhs = mk_const(elim_declar_name.clone(), elim_level_params.clone());
                    let fold_result = apps_lhs.fold_apps(apps_rhs.iter());
                    results_vec.push(fold_result.fold_lambdas(eps.iter()));
                },
                _ => continue
            }
        }

        results_vec
    }

    // `intro_idx` is just the position of this particular intro 
    // rule in the `intro_minors` seq
    pub fn mk_reduction_rule(&self, 
                             intro_minors_idx : usize, 
                             intro_minors : &Vec<Expr>, 
                             motive : &Expr, 
                             elim_declar_name : &Name, 
                             elim_level_params : &Vec<Level>) -> ReductionRule {
        

        let rr_arg1 = seq![self.parent.get_params(),
                           Some(motive),
                           &intro_minors,
                           &self.parent.get_indices(),
                           &self.intro_arguments];
        let fold_initial_val = mk_const(self.intro_name.clone(),
                                              self.parent.get_univ_params().clone());
        let fold_list = seq![self.parent.get_params(), &self.intro_arguments];
        let tail_apps = fold_initial_val.fold_apps(fold_list.iter());

        let app_rhs = seq![self.parent.get_params(),
                           Some(motive),
                           &intro_minors,
                           &self.parent.get_indices(),
                           Some(tail_apps)];
        let const_2 = mk_const(elim_declar_name.clone(), elim_level_params.clone());
        let rr_arg2 = const_2.fold_apps(app_rhs.iter());

        let rec_calls = self.recursive_calls(motive, intro_minors, elim_declar_name, elim_level_params);

        let rr_arg3 = intro_minors[intro_minors_idx].fold_apps(seq![&self.intro_arguments, rec_calls].iter());

        ReductionRule::new_nondef_rr(rr_arg1.as_slice(),
                                     rr_arg2,
                                     rr_arg3,
                                     None.into_iter())
    }


    // check an introduction rule
    pub fn check_intro(&self, env : &Arc<RwLock<Env>>) {
        assert!(self.intro_type_args.len() >= self.parent.num_params);
        let req_lhs_rhs = self.intro_type_args.iter().take(self.parent.num_params);

        let req_lhs = self.intro_type.fold_apps(req_lhs_rhs);
        let req_rhs = self.parent.minimal_const.fold_apps(self.parent.get_params().into_iter());
        self.parent.map_tc(|tc| tc.require_def_eq(&req_lhs, &req_rhs));

        // ATTN
        let mut tc0 = TypeChecker::new(None, env.clone());

        for elem in self.intro_arg_data.iter() {
            match elem {
                Left(e) => {
                    let infd1 = tc0.infer(e);
                    tc0.infer_universe_of_type(&infd1);
                },
                Right((eps, _)) => {
                    for e in eps {
                        let inferred = tc0.infer(e);
                        tc0.infer_universe_of_type(&inferred);
                    }
                }
            }
        }

        if self.parent.codomain_sort.maybe_nonzero() {
            for arg in self.intro_arguments.iter() {
                let inferred = self.parent.map_tc(|tc| tc.infer(arg));
                let arg_level = self.parent.map_tc(|tc| tc.infer_universe_of_type(&inferred));
                assert!(arg_level.leq(&self.parent.codomain_sort));
            }
        }
    }
}

