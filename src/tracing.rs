use indexmap::IndexSet;

use crate::name::{ Name, InnerName::*, mk_anon };
use crate::level::{ Level, InnerLevel::*, mk_zero };
use crate::expr::{ Expr, InnerExpr::* };
use crate::env::{ Declaration, CompiledModification, CompiledModification::*, };
use crate::tc::Flag;
use crate::reduction::ReductionRule;
use crate::utils::{ ShortCircuit, sep_spaces, comma_sep_list, comma_sep_list_parens, print_binderstyle };

use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use once_cell::sync::Lazy;
use std::sync::Arc;
use parking_lot::RwLock;

use Op::*;
use ItemIdx::*;
use TraceItem::*;

pub static TRACE_DATA_COUNTER : AtomicUsize = AtomicUsize::new(0);

pub static UNIV_TRACE_ITEMS : Lazy<Arc<RwLock<UnivItems>>> = Lazy::new(|| {
    let mut set = IndexSet::new();
    // set constant items
    set.insert(N(mk_anon()));
    set.insert(L(mk_zero()));
    set.insert(OptionNone);
    set.insert(EqShort);
    set.insert(NeqShort);
    set.insert(FlagTrue);
    set.insert(FlagFalse);
    set.insert(BoolTrue);
    set.insert(BoolFalse);
    set.insert(Unit);
    assert!(set.len() == 10);
    let univ_items = UnivItems {
        unique_inner : set
    };
    Arc::new(RwLock::new(univ_items))
});







// Having two separate enum variants lets us maintain a universal
// set and a forked set without having to later try and reason
// about offsets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemIdx {
    UnivIdx(usize),
    ForkIdx(usize),
}

// when doing string interpolation with the `Display` formatter,
// show elements of the base set as their inner usize. Show elements
// of a forked set as their inner usize, but with a `!` prefix
impl std::fmt::Display for ItemIdx {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            UnivIdx(n) => write!(f, "{}", n),
            ForkIdx(n) => write!(f, "!{}", n)
        }
    }
}

// Needed because of recursive insertion of items
// to account for instances where the user isn't tracing
// literally every possible item, meaning we'll receive 
// expressions which has children that aren't yet tracked.
// In those cases, when we insert the root element (for example
// the result of type inference) we need to recursively insert
// all of the children first. Since we're ultimately using
// collections that rely on a hash function, we need to reuse
// the pre-calculated hashes in Expr terms or we'll get crushed.
impl std::hash::Hash for TraceItem {
    fn hash<H : std::hash::Hasher>(&self, state : &mut H) {
        match self {
            N(n) => n.hash(state),
            L(l) => l.hash(state),
            E(e) => e.get_digest().hash(state),
            Seq(v) => v.hash(state),
            Tuple(fst, snd) => {
                std::mem::discriminant(self).hash(state);
                fst.hash(state);
                snd.hash(state);
            },
            Usize(n) => {
                std::mem::discriminant(self).hash(state);
                n.hash(state);
            }
            SomeItem(n) => {
                std::mem::discriminant(self).hash(state);
                n.hash(state);
            },
            Rr(r) => r.digest.hash(state),
            Declar(d) => d.hash(state),
            CompiledMod(c) => c.hash(state),
            OptionNone => std::mem::discriminant(self).hash(state),
            EqShort => std::mem::discriminant(self).hash(state),
            NeqShort => std::mem::discriminant(self).hash(state),
            FlagTrue => std::mem::discriminant(self).hash(state),
            FlagFalse => std::mem::discriminant(self).hash(state),
            BoolTrue => std::mem::discriminant(self).hash(state),
            BoolFalse => std::mem::discriminant(self).hash(state),
            Unit => ().hash(state),
        }
    }
}


// Enum that wraps items that can be traced. Kind of wonky right now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraceItem {
    N(Name),
    L(Level),
    E(Expr),
    Seq(Vec<ItemIdx>),
    SomeItem(ItemIdx),
    Tuple(ItemIdx, ItemIdx),
    Usize(usize),
    Rr(ReductionRule),
    Declar(Declaration),
    CompiledMod(CompiledModification),
    OptionNone,
    EqShort,
    NeqShort,
    FlagTrue,
    FlagFalse,
    Unit,
    BoolTrue,
    BoolFalse,
}


// Basic type for this module.
#[derive(Debug)]
pub struct TraceData {
    pub items_fork : ItemsFork,
    pub ops : OpSlab,
    pub current_parent_op : Option<OpIdx>,
    pub serial : usize,
}

// All of the information is just printed to stdout when a TraceData
// item is dropped from memory (meaning all of its calculations are done)
impl std::ops::Drop for TraceData {
    fn drop(&mut self) {
        println!("TraceData {} items :", self.serial);
        for (idx, _) in self.items_fork.inner.iter().enumerate() {
            println!("    {}", self.items_fork.format_item_declar_by_idx(ForkIdx(idx)));
        }
        println!("TraceData {} ops :", self.serial);
        for op in self.ops.slab.iter() {
            println!("    {}", op);
        }
        println!("\n");
    }
}

impl TraceData {
    pub fn new() -> Self {
        TraceData {
            items_fork : ItemsFork::new(),
            ops : OpSlab::new(),
            current_parent_op : None,
            serial : TRACE_DATA_COUNTER.fetch_add(1, Relaxed),
        }
    }




    // when you go to push a new item, if the `current_parent` is None, 
    // you can use that to automatically detect a root.

    pub fn push_child_op(&mut self, parent_idx : OpIdx, child_idx : OpIdx) {
        match self.ops.get_mut_infallible(parent_idx) {
            OpRoot { ref mut children, .. } => { children.push(child_idx); },
            OpNonroot { ref mut children, .. } => { children.push(child_idx); },
        }

    }

    pub fn push_arg(&mut self, op_idx : OpIdx, arg_idx : ItemIdx) {
        match self.ops.get_mut_infallible(op_idx) {
            OpRoot { ref mut args, .. } => { args.push(arg_idx); },
            OpNonroot { ref mut args, .. } => { args.push(arg_idx); }
        }
    }

    // Can take any T, where exists implementation `HasInsertItem<T>`
    // FIXME have to reimplement this due to HasInsertItem
    pub fn push_ret_val<A>(&mut self, op_idx : OpIdx, ret_val : A) 
    where TraceData: HasInsertItem<A> {
        let ret_val_idx = self.insert_item(ret_val);
        match self.ops.get_mut_infallible(op_idx) {
            OpRoot { ref mut ret_val, .. } => {
                assert!(ret_val.is_none());
                std::mem::replace(ret_val, Some(ret_val_idx));
            },
            OpNonroot { ref mut ret_val, .. } => {
                // Should never `re-set` a return value
                assert!(ret_val.is_none());
                std::mem::replace(ret_val, Some(ret_val_idx));
            },
        }
    }

    pub fn new_root_op(&mut self, ident : &'static str) -> OpIdx {
        let self_idx_usize = self.ops.slab.len();
        let self_idx = OpIdx::new(self_idx_usize);
        // make new root op
        let new_op = OpRoot {
            ident,
            self_idx,
            args : Vec::new(),
            ret_val : None,
            children : Vec::new(),
        };
        assert!(self.ops.slab.is_empty());

        // push actual op item onto slab.
        self.ops.slab.push(new_op.clone());
        assert_eq!(&self.ops.slab[self_idx_usize], &new_op);
        assert!(self.ops.slab.len() == 1);
        self.current_parent_op = Some(OpIdx(self_idx_usize));
        assert!(self.current_parent_op.is_some());
        self_idx
    }

    // args are added via mutability in the generated code.
    // need to do it this way to accomodate the way 
    // attribute macros work.
    pub fn new_nonroot_op(&mut self, ident : &'static str, parent_idx : OpIdx) -> OpIdx {
        let self_idx_usize = self.ops.slab.len();
        let self_idx = OpIdx::new(self_idx_usize);
        // 1. make new Op item
        let new_op = OpNonroot {
            ident,
            parent : parent_idx,
            self_idx,
            args : Vec::new(),
            ret_val : None,
            children : Vec::new(),
        };
        // 2. insert new op item into the slab
        self.ops.slab.push(new_op.clone());

        assert_eq!(&self.ops.slab[self_idx_usize], &new_op);

        // 3. Since this is non-root, it's a child node to some other op,
        // so push the new index onto it's parents children vec.
        self.push_child_op(parent_idx, self_idx);
        self_idx
    }

    pub fn op_is_root(&self, idx : OpIdx) -> bool {
        match self.ops.get_infallible(idx) {
            OpRoot { .. } => true,
            _ => false
        }
    }

    pub fn get_current_parent_op(&self) -> Option<OpIdx> {
        self.current_parent_op
    }

    pub fn get_current_parent_op_infallible(&self) -> OpIdx {
        self.current_parent_op.expect("`get_current_parent_op should never return `None`!")
    }
    
    //pub fn roll_back_to(&mut self, rollback_point : OpIdx) {
    //    self.current_parent_op = Some(rollback_point);
    //}

    //pub fn roll_back_one(&mut self, current_idx : OpIdx) {
    //    match self.ops.get_infallible(current_idx) {
    //        OpRoot { .. } => panic!("roll_back_one should never be executed with a root node!"),
    //        OpNonroot { parent, .. } => { self.current_parent_op = Some(*parent) }
    //    }
    //}

    pub fn set_parent_as(&mut self, new_parent : OpIdx) {
        self.current_parent_op = Some(new_parent)
    }
}









// The universal set of items originally laid out in the export
// file produced by Lean. Keep in mind that not all TraceData
// elements can see the whole set. For instance, type checking
// operations done on an inductive type declared as item 100
// should not be able to see universal set elements declared
// after that.
#[derive(Debug)]
pub struct UnivItems {
    pub unique_inner : IndexSet<TraceItem>,
}

#[derive(Debug, Clone)]
pub struct ItemsFork {
    inner : IndexSet<TraceItem>,
    pub forked_at : usize,
}

impl ItemsFork {
    // the action of `forking` this set of TraceItems is implicit. 
    // All it means is that when we go to do a look up or insertion,
    // we'll also check the subset of the globally available universal set
    // that our fork should be aware of before checking our
    // own forked set of items.
    pub fn new() -> Self {
        ItemsFork {
            inner : IndexSet::new(),
            forked_at : (*UNIV_TRACE_ITEMS).read().unique_inner.len(),
        }
    }

    // For some item `I`, if it exist in the universal set at a position
    // that should be visible to this fork. If so, return its index.
    // If not, check whether it exists in the forked set, returning its 
    // index if so. If `I` is in neither set, return `None`
    pub fn get_idx_if_exists(&self, item : &TraceItem) -> Option<ItemIdx> {
        if let Some((u_idx, _)) = (*UNIV_TRACE_ITEMS).read().unique_inner.get_full(item) {
            if u_idx < self.forked_at {
                Some(UnivIdx(u_idx))
            } else {
                self.inner.get_full(item).map(|(f_idx, _)| ForkIdx(f_idx))
            }
        } else {
            self.inner.get_full(item).map(|(f_idx, _)| ForkIdx(f_idx))
        }
    }

    pub fn get_idx_infallible(&self, item : &TraceItem) -> ItemIdx {
        match self.get_idx_if_exists(item) {
            Some(x) => x,
            None => panic!("`get_idx_infallible failed getting item : {:?}", item)
        }
    }


    pub fn fork_contains(&self, item : &TraceItem) -> bool {
        self.get_idx_if_exists(item).is_some()
    }

    // for some item, either get the index at which it already
    // existed or insert it and get the new idx
    pub fn get_idx_or_insert_head(&mut self, item : TraceItem) -> ItemIdx {
        match self.get_idx_if_exists(&item) {
            Some(idx) => idx,
            None => ForkIdx(self.inner.insert_full(item).0)
        }
    }

    pub fn get_by_idx_infallible(&self, item_idx : ItemIdx) -> TraceItem {
        match item_idx {
            UnivIdx(u_idx) => {
                if (u_idx < self.forked_at) {
                    if let Some(u_item) = (*UNIV_TRACE_ITEMS).read().unique_inner.get_index(u_idx).cloned() {
                        u_item
                    } else {
                        panic!("`get_by_idx_infallible should never fail on the univ_set. Tried to get {:?}\n", item_idx)

                    }
                } else {
                    panic!("`get_by_idx_infallible should never be given a UnivIdx greater than the point at which it was forked. Got arg {:?}, was forked at {}\n", item_idx, self.forked_at)
                }
            },
            ForkIdx(f_idx) => {
                if let Some(item) = self.inner.get_index(f_idx).cloned() {
                    item
                } else {
                    panic!("`get_by_idx_infallible` should never fail on items_fork. Looked for {:?}\n", item_idx)
                }
            }
        }
    }


    pub fn format_item_declar_by_idx(&self, item_idx : ItemIdx) -> String {
        let mut base = format!("{} ", item_idx);

        let rest = match self.get_by_idx_infallible(item_idx) {
            N(n) => self.format_name_declar(item_idx, n),
            L(l) => self.format_level_declar(item_idx, l),
            E(e) => self.format_expr_declar(e),
            Seq(v) => {
                let mut items_base = Vec::new();
                for elem in v {
                    items_base.push(format!("{}", elem));
                }
                format!("#SEQ {}", sep_spaces(items_base))
            },
            EqShort => String::from("#SSEQ"),
            NeqShort => String::from("#SSNEQ"),
            Tuple(idx1, idx2) => format!("#TUP {} {}", idx1, idx2),
            Usize(n) => format!("#INT {}", n),
            SomeItem(idx) => format!("#SOME {}", idx),
            Rr(rr) => {
                let mut ext = format!("#RR {} {} {} {} {}",
                self.get_idx_infallible(&N(rr.lhs_const_name)),
                self.get_idx_infallible(&E(rr.lhs)),
                self.get_idx_infallible(&E(rr.rhs)),
                rr.lhs_var_bound,
                rr.lhs_args_size);
                for elem in rr.majors.iter() {
                    ext.push_str(format!(" {}", elem).as_str());
                }
                ext
            },
            Declar(d) => {
                let name_idx = self.get_idx_infallible(&N(d.name.clone()));
                let univ_vec = d.univ_params.as_ref()
                                            .clone()
                                            .iter()
                                            .map(|x| self.get_idx_infallible(&L(x.clone())))
                                            .collect::<Vec<ItemIdx>>();
                let univ_idx = self.get_idx_infallible(&Seq(univ_vec));
                let ty_idx = self.get_idx_infallible(&E(d.ty.clone()));
                format!("DEC {} {} {} {}", name_idx, univ_idx, ty_idx, d.height)
            },
            CompiledMod(m) => self.format_compiled_mod(m),
            OptionNone => String::from("#NONE"),
            FlagTrue => String::from("#FLAGT"),
            FlagFalse => format!("#FLAGF"),
            Unit => String::from("#UNIT"),
            BoolTrue => String::from("#TT"),
            BoolFalse => String::from("#FF")
        };
        base.push_str(rest.as_str());
        base

    }

    fn format_compiled_mod(&self, m : CompiledModification) -> String {
        match &m {
            CompiledAxiomMod(d) => {
                format!("#CAX {}", self.get_idx_infallible(&Declar(d.clone())))
            },
            CompiledDefinition(d, r, t, v) => {
                format!("#CDEF {} {} {} {}",
                self.get_idx_infallible(&Declar(d.clone())),
                self.get_idx_infallible(&Rr(r.clone())),
                self.get_idx_infallible(&E(t.clone())),
                self.get_idx_infallible(&E(v.clone())))
            },
            CompiledQuotMod(ds, r) => {
                let mut declar_idxs = Vec::new();
                for d in ds.iter() {
                    declar_idxs.push(self.get_idx_infallible(&Declar(d.clone())));
                }
                let declars_seq_idx = self.get_idx_infallible(&Seq(declar_idxs));
                format!("#CQUOT {} {}", declars_seq_idx, self.get_idx_infallible(&Rr(r.clone())))
            },
            // This will go away in the next revision.
            CompiledInductive(d1, ds, d2, rs) => {
                let mut declar_idxs = Vec::new();
                for d in ds.iter() {
                    declar_idxs.push(self.get_idx_infallible(&Declar(d.clone())));
                }

                let mut rr_idxs = Vec::new();
                for r in rs.iter() {
                    rr_idxs.push(self.get_idx_infallible(&Rr(r.clone())));
                }
                format!("#CIND {} {} {} {}", 
                self.get_idx_infallible(&Declar(d1.clone())),
                self.get_idx_infallible(&Seq(declar_idxs)),
                self.get_idx_infallible(&Declar(d2.clone())),
                self.get_idx_infallible(&Seq(rr_idxs)),
                )
            }

        }
    }

// only takes the item_idx so we can make an extra sanity assertion.
    fn format_name_declar(&self, item_idx : ItemIdx, n : Name) -> String {
        match n.as_ref() {
            Anon => {
                assert_eq!(item_idx, UnivIdx(0));
                String::from("Anon")
            }
            Str(pfx, hd) => format!("#NS {} {}", 
                                        self.get_idx_infallible(&N(pfx.clone())),
                                        hd),
            Num(pfx, hd) => format!("#NI {} {}", 
                                        self.get_idx_infallible(&N(pfx.clone())),
                                        hd),
        }
    }

// only takes the item_idx so we can make an extra sanity assertion.
    fn format_level_declar(&self, item_idx : ItemIdx, l : Level) -> String {
        match l.as_ref() {
            Zero => {
                assert_eq!(item_idx, UnivIdx(1));
                String::from("Zero")
            },
            Succ(inner) => format!("#US {}", self.get_idx_infallible(&L(inner.clone()))),
            Max(lhs, rhs) => format!("#UM {} {}", 
                                                self.get_idx_infallible(&L(lhs.clone())),
                                                self.get_idx_infallible(&L(rhs.clone()))),

            IMax(lhs, rhs) => format!("#UIM {} {}", 
                                                self.get_idx_infallible(&L(lhs.clone())),
                                                self.get_idx_infallible(&L(rhs.clone()))),
            Param(p) => format!("#UP {}", self.get_idx_infallible(&N(p.clone()))),
        }
    }

    fn format_expr_declar(&self, e : Expr) -> String {
        match e.as_ref() {
            Var(_, dbj) => format!("#EV {}", dbj),
            Sort(_, lvl) => format!("#ES {}", self.get_idx_infallible(&L(lvl.clone()))),
            Const(_, n, lvls) => {
                let mut base = format!("#EC {} ", self.get_idx_infallible(&N(n.clone())));
                for elem in lvls.as_ref().iter() {
                    base.push_str(format!("{} ", self.get_idx_infallible(&L(elem.clone()))).as_str());
                }
                base
            },
            App(_, lhs, rhs) => format!("#EA {} {}",
                self.get_idx_infallible(&E(lhs.clone())),
                self.get_idx_infallible(&E(rhs.clone())),
                ),
            Pi(_, bind, body) => format!("#EP {} {} {} {}", 
                print_binderstyle(&bind.style),
                self.get_idx_infallible(&N(bind.pp_name.clone())),
                self.get_idx_infallible(&E(bind.ty.clone())),
                self.get_idx_infallible(&E(body.clone())),
                ),
            Lambda(_, bind, body) => format!("#EL {} {} {} {}",
                print_binderstyle(&bind.style),
                self.get_idx_infallible(&N(bind.pp_name.clone())),
                self.get_idx_infallible(&E(bind.ty.clone())),
                self.get_idx_infallible(&E(body.clone())),
                ),
            Let(_, bind, val, body) => format!("#EZ {} {} {} {} {}", 
                print_binderstyle(&bind.style),
                self.get_idx_infallible(&N(bind.pp_name.clone())),
                self.get_idx_infallible(&E(bind.ty.clone())),
                self.get_idx_infallible(&E(val.clone())),
                self.get_idx_infallible(&E(body.clone()))
                ),
            Local(_, serial, bind) => format!("#ELO {} {} {} {}", 
                print_binderstyle(&bind.style),
                serial,
                self.get_idx_infallible(&N(bind.pp_name.clone())),
                self.get_idx_infallible(&E(bind.ty.clone())),
                ),
        }
    }


}



#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OpIdx(pub usize);

impl std::fmt::Display for OpIdx {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            OpIdx(n) => write!(f, "${}", n)
        }
    }
}

impl OpIdx {
    pub fn new(n : usize) -> Self {
        OpIdx(n)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    OpRoot { 
        ident : &'static str, 
        self_idx : OpIdx,
        args : Vec<ItemIdx>, 
        ret_val : Option<ItemIdx>, 
        children : Vec<OpIdx> 
    },
    OpNonroot {
        ident : &'static str, 
        self_idx : OpIdx,
        args : Vec<ItemIdx>, 
        ret_val : Option<ItemIdx>, 
        children : Vec<OpIdx> ,
        parent : OpIdx
    }
}

impl std::fmt::Display for Op {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.format_op_line())
    }
}

impl Op {

    fn format_op_line(&self) -> String {
        let mut strings = Vec::new();

        match self {
            OpRoot { ident, self_idx, args, ret_val, children } => {
                strings.push(format!("{}", self_idx));
                strings.push(ident.to_string());

                strings.push(
                    comma_sep_list_parens(args.iter().map(|x| format!("{}", x)))
                );

                if let Some(item_idx) = ret_val {
                    strings.push(format!("{}", item_idx))
                } else {
                    panic!("format_op_line should not receive an op with no return value!")
                }

                strings.push(
                    comma_sep_list(children.iter().map(|x| format!("{}", x)))
                );

                sep_spaces(strings)
            },
            OpNonroot { ident, self_idx, args, ret_val, children, parent } => {
                strings.push(format!("{}", self_idx));
                strings.push(format!("{}", parent));
                strings.push(ident.to_string());

                strings.push(
                    comma_sep_list_parens(args.iter().map(|x| format!("{}", x)))
                );

                if let Some(item_idx) = ret_val {
                    strings.push(format!("{}", item_idx))
                } else {
                    panic!("format_op_line should not receive an op with no return value!")
                }

                strings.push(
                    comma_sep_list(children.iter().map(|x| format!("{}", x)))
                );
                sep_spaces(strings)
            }
        }
    }
}


#[derive(Debug, Clone)]
pub struct OpSlab {
    pub slab : Vec<Op>,
}

impl OpSlab {
    pub fn new() -> Self {
        OpSlab {
            slab : Vec::new()
        }
    }

    pub fn len(&self) -> usize {
        self.slab.len()
    }


    pub fn get_infallible(&self, idx : OpIdx) -> &Op {
        match idx {
            OpIdx(n) => self.slab.get(n)
                                 .expect("OpSlab::get() should never return `None`"),
        }
    }

    pub fn get_mut_infallible(&mut self, idx : OpIdx) -> &mut Op {
        match idx {
            OpIdx(n) => self.slab.get_mut(n)
                                 .expect("OpSlab::get_mut() should never return `None`"),
        }
    }
}


pub trait HasInsertItem<T> {
    fn insert_item(&mut self, t : T) -> ItemIdx;
}

impl HasInsertItem<Name> for TraceData {
    fn insert_item(&mut self, n : Name) -> ItemIdx {
        if let Some(idx) = self.items_fork.get_idx_if_exists(&N(n.clone())) {
            idx
        } else {
            let _wait_for = match n.as_ref() {
                Anon => panic!("name `Anon` should already exist!"),
                Str(pfx, _) | Num(pfx, _) => self.insert_item(pfx),
            };

            let as_item = N(n.clone());
            assert!(!(self.items_fork.fork_contains(&as_item)));
            self.items_fork.get_idx_or_insert_head(as_item)
        }
    }
}

impl HasInsertItem<Level> for TraceData {
    fn insert_item(&mut self, l : Level) -> ItemIdx {
        if let Some(idx) = self.items_fork.get_idx_if_exists(&L(l.clone())) {
            idx
        } else {
            let _wait_for = match l.as_ref() {
                Zero => panic!("Sort `Zero` should already exist!"),
                Succ(inner) => self.insert_item(inner),
                Max(lhs, rhs) | IMax(lhs, rhs) => {
                    self.insert_item(lhs);
                    self.insert_item(rhs)
                },
                Param(p) => self.insert_item(p),
            };

            let as_item = L(l.clone());
            assert!(!(self.items_fork.fork_contains(&as_item)));
            self.items_fork.get_idx_or_insert_head(as_item)
        }
    }
}



impl HasInsertItem<Expr> for TraceData {
    fn insert_item(&mut self, e : Expr) -> ItemIdx {
        if let Some(idx) = self.items_fork.get_idx_if_exists(&E(e.clone())) {
            idx
        } else {
            let _wait_for = match e.as_ref() {
                Var(..) => (),
                Sort(_, lvl) => { self.insert_item(lvl); },
                Const(_, n, lvls) => {
                    self.insert_item(n);
                    self.insert_item(lvls.as_ref());
                },
                App(_, lhs, rhs) => {
                    self.insert_item(lhs);
                    self.insert_item(rhs);
                },
                Lambda(_, bind, body) | Pi(_, bind, body) => {
                    self.insert_item(&bind.pp_name);
                    self.insert_item(&bind.ty);
                    self.insert_item(body);
                },
                Let(_, bind, val, body) => {
                    self.insert_item(&bind.pp_name);
                    self.insert_item(&bind.ty);
                    self.insert_item(val);
                    self.insert_item(body);
                },
                Local(_, _, bind) => {
                    self.insert_item(&bind.pp_name);
                    self.insert_item(&bind.ty);
                }
            };

            let as_item = E(e.clone());
            assert!(!(self.items_fork.fork_contains(&as_item)));
            self.items_fork.get_idx_or_insert_head(as_item)
        }
    }
}

impl HasInsertItem<()> for TraceData {
    fn insert_item(&mut self, _ : ()) -> ItemIdx {
        self.items_fork.get_idx_or_insert_head(Unit)
    }
}

impl HasInsertItem<usize> for TraceData {
    fn insert_item(&mut self, n : usize) -> ItemIdx {
        self.items_fork.get_idx_or_insert_head(Usize(n))
    }
}

impl HasInsertItem<bool> for TraceData {
    fn insert_item(&mut self, b : bool) -> ItemIdx {
        if b {
            self.items_fork.get_idx_or_insert_head(BoolTrue)
        } else {
            self.items_fork.get_idx_or_insert_head(BoolFalse)
        }
    }
}

impl HasInsertItem<Flag> for TraceData {
    fn insert_item(&mut self, f : Flag) -> ItemIdx {
        match f {
            Flag::FlagT => self.items_fork.get_idx_or_insert_head(FlagTrue),
            Flag::FlagF => self.items_fork.get_idx_or_insert_head(FlagFalse),
        }
    }
}

impl<T> HasInsertItem<Option<T>> for TraceData
where TraceData : HasInsertItem<T> {
    fn insert_item(&mut self, t : Option<T>) -> ItemIdx {
        match t {
            None => self.items_fork.get_idx_or_insert_head(OptionNone),
            Some(x) => {
                let inner_idx = self.insert_item(x);
                self.items_fork.get_idx_or_insert_head(SomeItem(inner_idx))
            }
        }
    }
}

impl<A, B> HasInsertItem<(A, B)> for TraceData 
where TraceData : HasInsertItem<A>,
      TraceData : HasInsertItem<B> {
    fn insert_item(&mut self, pair : (A, B)) -> ItemIdx {
        let l = self.insert_item(pair.0);
        let r = self.insert_item(pair.1);
        self.items_fork.get_idx_or_insert_head(Tuple(l, r))
    }
}

impl HasInsertItem<ShortCircuit> for TraceData {
    fn insert_item(&mut self, x : ShortCircuit) -> ItemIdx {
        match x {
            ShortCircuit::EqShort => self.items_fork.get_idx_or_insert_head(EqShort),
            ShortCircuit::NeqShort => self.items_fork.get_idx_or_insert_head(NeqShort),
        }
    }
}

impl HasInsertItem<ReductionRule> for TraceData {
    fn insert_item(&mut self, rr : ReductionRule) -> ItemIdx {
        self.insert_item(&rr.lhs_const_name);
        self.insert_item(&rr.lhs);
        self.insert_item(&rr.rhs);
        self.items_fork.get_idx_or_insert_head(Rr(rr))
    }

}

impl HasInsertItem<Declaration> for TraceData {
    fn insert_item(&mut self, d : Declaration) -> ItemIdx {
        self.insert_item(&d.name);
        self.insert_item(d.univ_params.as_ref());
        self.insert_item(&d.ty);
        self.items_fork.get_idx_or_insert_head(Declar(d))
    }

}

impl HasInsertItem<CompiledModification> for TraceData {
    fn insert_item(&mut self, m : CompiledModification) -> ItemIdx {
        match &m {
            CompiledModification::CompiledAxiomMod(dd) => {
                self.insert_item(&dd);
                self.items_fork.get_idx_or_insert_head(CompiledMod(m))
            }
            CompiledModification::CompiledDefinition(dd, rr, e1, e2) => {
                self.insert_item(&dd);
                self.insert_item(&rr);
                self.insert_item(&e1);
                self.insert_item(&e2);
                self.items_fork.get_idx_or_insert_head(CompiledMod(m))
            },
            CompiledModification::CompiledQuotMod(dds, rr) => {
                self.insert_item(&dds);
                self.insert_item(&rr);
                self.items_fork.get_idx_or_insert_head(CompiledMod(m))
            },
            CompiledModification::CompiledInductive(a, b, c, d) => {
                self.insert_item(&a);
                self.insert_item(&b);
                self.insert_item(&c);
                self.insert_item(&d);
                self.items_fork.get_idx_or_insert_head(CompiledMod(m))
            }

        }
    }
}

impl<T> HasInsertItem<Vec<T>> for TraceData 
where TraceData: HasInsertItem<T>,
      T : Clone {
    fn insert_item(&mut self, v : Vec<T>) -> ItemIdx {
        let idx_vec = v.into_iter().map(|x| self.insert_item(x)).collect::<Vec<ItemIdx>>();
        self.items_fork.get_idx_or_insert_head(Seq(idx_vec))
    }
}

impl<T> HasInsertItem<&T> for TraceData 
where TraceData: HasInsertItem<T>,
      T : Clone {
    fn insert_item(&mut self, r : &T) -> ItemIdx {
        self.insert_item(r.clone())
    }
}

impl<T> HasInsertItem<&[T]> for TraceData 
where TraceData: HasInsertItem<T>,
      T : Clone {
    fn insert_item(&mut self, v : &[T]) -> ItemIdx {
        let idx_vec = v.into_iter().map(|x| self.insert_item(x.clone())).collect::<Vec<ItemIdx>>();
        self.items_fork.get_idx_or_insert_head(Seq(idx_vec))
    }
}
