
use std::sync::Arc;

use crate::chain;
use crate::name::Name;
use crate::level::{ mk_param, };
use crate::reduction::ReductionRule;
use crate::env::{ Declaration, Modification, CompiledModification };
use crate::expr::{ BinderStyle::*, 
                   mk_prop, 
                   mk_local, 
                   mk_const, 
                   mk_app, 
                   mk_sort };


/// Quot ends up being four introduction rules and one reduction rule 
/// which are declared once, very early on in the export file 
/// (right after the inductive definition of equality). 
/// This module is pretty much just "by hand" assembly of Quot. I'm not sure
/// why the export file doesn't lend more help in putting this together.
#[derive(Clone)]
pub struct Quot {
    pub declarations: Vec<Declaration>,
    pub reduction_rule: ReductionRule,
}

pub fn new_quot() -> Modification {
    // There are a bunch of expressions that get used ad nauseum here,
    // so we define some of them as reusable omponents to make later definitions
    // (a little bit) more compact. The key definitions are annotated with their
    // lean equivalent
    let prop = mk_prop();
    let param_u = || mk_param("u");
    let param_v = || mk_param("v");
    let params_u = || Arc::new(vec![param_u()]);
    let params_uv = || Arc::new(vec![param_u(), param_v()]);
    let sort_u = mk_sort(mk_param("u"));
    let _A = mk_local("A", mk_sort(param_u()), Implicit);
    let _B = mk_local("B", mk_sort(mk_param("v")), Implicit);
    let _R = mk_local("R", _A.mk_arrow(&_A.mk_arrow(&prop)), Default); 
    let _f = mk_local("f", _A.mk_arrow(&_B), Default);
    let _a = mk_local("a", _A.clone(), Default);
    let _b = mk_local("b", _A.clone(), Default);


    let quot_const_univ_u = || mk_const("quot", vec![param_u()]);
    let quot_mk_const_univ_u = || mk_const(Name::from("quot").extend_str("mk"), vec![param_u()]);
    let quot_pi_app = sort_u.fold_pis(chain![&_A, &_R]);

    // First introduction rule. in Lean : 
    // quot : Π {α : Sort u}, (α → α → Prop) → Sort u
    let quot = Declaration::mk(Name::from("quot"),
                                    params_u(),
                                    quot_pi_app,
                                    None,
                                    Some(true));

    let quot_mk_f_a = mk_const("quot", params_u()).fold_apps(vec![&_A, &_R]);

    let quot_mk_f = _A.mk_arrow(&quot_mk_f_a);
        
    // Second introduction rule. In lean : 
    // quot.mk : Π {α : Sort u} (r : α → α → Prop), α → @quot α r
    let quot_mk = Declaration::mk(
        Name::from("quot").extend_str("mk"),
        params_u(),
        quot_mk_f.fold_pis(chain![&_A, &_R]),
        None,
        Some(true)
    );

    let eq_const = mk_const("eq", vec![param_v()]);
    let app1 = mk_app(_f.clone(), _a.clone());
    let app2 = mk_app(_f.clone(), _b.clone());
    let eq_app = eq_const.fold_apps(vec![&_B, &app1, &app2]);
    let eq_lhs = _R.fold_apps(vec![&_a, &_b]);

    let inner_app = eq_lhs.mk_arrow(&eq_app);
    let lower_const = quot_const_univ_u();
    let lower_apps = lower_const.fold_apps(vec![&_A, &_R]);
    let lhs_pis = inner_app.fold_pis(chain![&_a, &_b]);
    let triple_arrow = lhs_pis.mk_arrow(&(lower_apps.mk_arrow(&_B)));
    let pis_together = triple_arrow.fold_pis(chain![&_A, &_R, &_B, &_f]);


    // Third introduction rule. In lean : 
    // quot.lift : Π {α : Sort u} {r : α → α → Prop} {β : Sort v} (f : α → β), 
    //               (∀ (a b : α), r a b → f a = f b) → quot r → β
    let quot_lift = Declaration::mk(
        quot.name.extend_str("lift"),
        params_uv(),
        pis_together,
        None,
        Some(true)
    );

    let B2_arrows_lhs = quot_const_univ_u().fold_apps(vec![&_A, &_R]);
    let _B2 = mk_local("B",
                             B2_arrows_lhs.mk_arrow(&prop),
                             Implicit);
    let _q = mk_local("q",
                            quot_const_univ_u().fold_apps(vec![&_A, &_R]),
                            Default);

    let ind_pi_1_inner = quot_mk_const_univ_u().fold_apps(vec![&_A, &_R, &_a]);
    let ind_pi_1_mid = _B2.fold_apps(Some(&ind_pi_1_inner));
    let ind_pi_1 = ind_pi_1_mid.fold_pis(chain![&_a]);
    let B2_q = _B2.fold_apps(vec![&_q]);
    let ind_pi_2 = B2_q.fold_pis(chain![&_q]);
    let ind_arrows = ind_pi_1.mk_arrow(&ind_pi_2);

    // Last introduction rule. In Lean : 
    // quot.ind : ∀ {α : Sort u} {r : α → α → Prop} {β : @quot α r → Prop},
    //            (∀ (a : α), β (@quot.mk α r a)) → ∀ (q : @quot α r), β q
    let quot_ind = Declaration::mk(
        quot.name.extend_str("ind"),
        params_u(),
        ind_arrows.fold_pis(chain![&_A, &_R, &_B2]),
        None,
        Some(true)
    );

    let const_eq_v = mk_const("eq", vec![param_v()]);


    let _h = mk_local("h",
                            const_eq_v.fold_apps(vec![&_b, &app1, &app2]),
                            Default);

    let quot_red_arg_const = mk_const(quot_lift.name.clone(), vec![param_u()]);
    let arg2_rhs_const = mk_const(quot_mk.name.clone(), vec![param_u()]);
    let arg2_rhs_apps = arg2_rhs_const.fold_apps(vec![&_A, &_R, &_a]);
    let quot_red_arg2 = quot_red_arg_const.fold_apps(vec![&_A, 
                                                          &_R, 
                                                          &_B, 
                                                          &_f, 
                                                          &_h, 
                                                          &arg2_rhs_apps]);

    // Sole reduction rule.
    let quot_red = ReductionRule::new_nondef_rr(
        &[_A.clone(), _R.clone(), _B.clone(), _f.clone(), _a.clone(), _h.clone()],
        quot_red_arg2,
        _f.fold_apps(vec![&_a]),
        None.into_iter()
    );

    let q = Quot {
            declarations : vec![quot, quot_mk, quot_ind, quot_lift],
            reduction_rule : quot_red
    };

    Modification::QuotMod(q)
}

impl Quot {
    pub fn compile_self(self) -> CompiledModification {
        CompiledModification::CompiledQuotMod(self.declarations, self.reduction_rule)
    }
}
