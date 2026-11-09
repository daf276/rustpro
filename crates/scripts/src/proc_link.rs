// Port of proc_link.lua: Link.AddProcedure with its condition, target and operation. EFFECT_FORMUD_SKIPPER comes with
// the card that grants it.
use std::cell::RefCell;

use engine::{Args, CardId, CardSet, Duel, EffectId, ExtraMaterialCall, Script, Value};

use crate::constant::*;
use crate::utility::aux::ExtraMaterials;
use crate::utility::{CardUtility, aux};

pub struct Link;

// The parameters of Link.AddProcedure (f, min, max, specialchk, spcon): `MAX` None is the monster's Link Rating, a nil
// filter passes every card and a nil specialchk or spcon everything.
pub trait LinkProcedure {
    const MIN: usize;
    const MAX: Option<usize> = None;

    fn f(_c: CardId, _duel: &Duel, _lc: CardId, _sumtype: u64, _tp: u8) -> bool {
        true
    }

    fn specialchk(_duel: &Duel, _sg: CardSet, _lc: CardId, _sumtype: u64, _tp: u8) -> bool {
        true
    }

    fn spcon(_duel: &Duel, _e: EffectId, _tp: u8) -> bool {
        true
    }
}

const LINK_MATERIAL: u64 = SUMMON_TYPE_LINK as u64 | MATERIAL_LINK;

// What the recursive checks share: the player, all materials (mg+tg) and those that are not extra materials (og), the
// Link Monster, the material counts and the extra materials.
struct Search<'a> {
    tp: u8,
    mg: CardSet,
    og: CardSet,
    lc: CardId,
    minc: usize,
    maxc: usize,
    emt: &'a ExtraMaterials,
}

// The filt table of the recursive checks: the indices into emt of the extra materials the selection uses. The Lua passes
// one copy to all the calls of an IsExists or Filter, which add to it in turn.
type Filters = RefCell<Vec<usize>>;

impl Link {
    pub fn add_procedure<P: LinkProcedure>(duel: &mut Duel, c: CardId) {
        Link::add_procedure_full::<P>(duel, c, None);
    }

    // `desc` None takes 1174.
    pub fn add_procedure_full<P: LinkProcedure>(duel: &mut Duel, c: CardId, desc: Option<u64>) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_description(duel, desc.unwrap_or(1174));
        e1.set_code(duel, EFFECT_SPSUMMON_PROC);
        e1.set_property(
            duel,
            EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_IGNORE_IMMUNE,
        );
        e1.set_range(duel, LOCATION_EXTRA);
        e1.set_condition(duel, Link::condition::<P>);
        e1.set_target(duel, Link::target::<P>);
        e1.set_operation(duel, Link::operation);
        e1.set_value(duel, SUMMON_TYPE_LINK as i32);
        c.register_effect(duel, e1);
    }

    fn max<P: LinkProcedure>(duel: &Duel, c: CardId) -> usize {
        P::MAX.unwrap_or_else(|| c.get_link(duel) as usize)
    }

    fn condition_filter<P: LinkProcedure>(c: CardId, duel: &Duel, lc: CardId, tp: u8) -> bool {
        unported!(duel, c.is_has_effect(duel, EFFECT_FORMUD_SKIPPER).is_some());
        c.is_can_be_link_material(duel, Some(lc), Some(tp)) && P::f(c, duel, lc, LINK_MATERIAL, tp)
    }

    fn get_link_count(c: CardId, duel: &Duel) -> u32 {
        if c.is_link_monster(duel) && c.get_link(duel) > 1 {
            1 + 0x10000 * c.get_link(duel)
        } else {
            1
        }
    }

    // c joins sg if the filters of the extra materials already used take it and, unless it is in og, an extra material
    // entry does (added to filt).
    fn joins(duel: &Duel, c: CardId, sg: CardSet, s: &Search, filt: &Filters) -> bool {
        let passes = |&index: &usize| {
            s.emt
                .get(index)
                .is_none_or(|&ex| aux::extra_material_filter(duel, ex, c, s.tp, sg, s.mg, s.lc, 1))
        };
        let passed = filt.borrow().iter().all(passes);
        passed && (s.og.is_contains(c) || aux::check_valid_extra(duel, c, s.tp, sg, s.mg, s.lc, s.emt, &mut filt.borrow_mut()))
    }

    fn check_recursive<P: LinkProcedure>(duel: &Duel, c: CardId, sg: CardSet, s: &Search, filt: &Filters) -> bool {
        if sg.len() > s.maxc {
            return false;
        }
        let sg = sg.union(c.into());
        if !Link::joins(duel, c, sg, s, filt) {
            return false;
        }
        let filt = filt.borrow().clone();
        Link::check_goal::<P>(duel, sg, s, &filt) || (sg.len() < s.maxc && Link::any_recursive::<P>(duel, s.mg, sg, s, filt))
    }

    // mg:IsExists(Link.CheckRecursive, 1, sg, ..., {table.unpack(filt)}).
    fn any_recursive<P: LinkProcedure>(duel: &Duel, mg: CardSet, sg: CardSet, s: &Search, filt: Vec<usize>) -> bool {
        let filt = RefCell::new(filt);
        mg.is_exists(duel, |tc, duel| Link::check_recursive::<P>(duel, tc, sg, s, &filt), 1, Some(sg))
    }

    // Link.CheckRecursive2 with secondg as mg: walks the cards of sg2 into sg, collecting the extra materials they use
    // in filt.
    fn check_recursive2<P: LinkProcedure>(duel: &Duel, c: CardId, sg: CardSet, sg2: CardSet, s: &Search, filt: &Filters) -> bool {
        if sg.len() > s.maxc {
            return false;
        }
        let sg = sg.union(c.into());
        if !Link::joins(duel, c, sg, s, filt) {
            return false;
        }
        match sg2.difference(sg).first() {
            None if !s.mg.is_empty() => Link::any_recursive::<P>(duel, s.mg, sg, s, filt.borrow().clone()),
            None => Link::check_goal::<P>(duel, sg, s, &filt.borrow()),
            Some(next) => Link::check_recursive2::<P>(duel, next, sg, sg2, s, filt),
        }
    }

    fn check_goal<P: LinkProcedure>(duel: &Duel, sg: CardSet, s: &Search, filt: &[usize]) -> bool {
        let none = CardSet::default();
        let taken = |&index: &usize| {
            s.emt.get(index).is_none_or(|&ex| {
                sg.is_exists(
                    duel,
                    |c, duel| aux::extra_material_filter(duel, ex, c, s.tp, sg, none, s.lc, 1),
                    1,
                    None,
                )
            })
        };
        filt.iter().all(taken)
            && sg.len() >= s.minc
            && sg.check_with_sum_equal(duel, Link::get_link_count, s.lc.get_link(duel), sg.len(), sg.len())
            && P::specialchk(duel, sg, s.lc, LINK_MATERIAL, s.tp)
            && duel.get_location_count_from_ex(s.tp, s.tp, Some(sg), s.lc) > 0
    }

    // The materials both the condition and the target start from: (mg, mustg, min, max), None if the forced counts
    // are out of range.
    fn materials<P: LinkProcedure>(duel: &Duel, a: &Args, c: CardId, tp: u8) -> Option<(CardSet, CardSet, usize, usize)> {
        let (minc, maxc) = (P::MIN, Link::max::<P>(duel, c));
        let g = a
            .materials
            .unwrap_or_else(|| duel.get_matching_group(|c, duel| c.is_faceup(duel), tp, LOCATION_MZONE, 0, None));
        let mg = g.filter(duel, |mc, duel| Link::condition_filter::<P>(mc, duel, c, tp), None);
        let mut mustg = aux::get_must_be_material_group(duel, tp, Some(g), tp, Some(c), Some(mg), REASON_LINK);
        if let Some(must) = a.must {
            mustg.merge(must);
        }
        let (min, max) = (usize::from(a.min), usize::from(a.max));
        if (min != 0 && min < minc) || (max != 0 && max > maxc) {
            return None;
        }
        let min = if min != 0 { min } else { minc };
        let max = if max != 0 { max } else { maxc };
        Some((mg, mustg, min, max))
    }

    fn condition<P: LinkProcedure>(duel: &Duel, a: &Args) -> bool {
        let Some(c) = a.c else { return true };
        if c.is_type(duel, TYPE_PENDULUM) && c.is_faceup(duel) {
            return false;
        }
        let tp = c.get_controler(duel);
        if !P::spcon(duel, a.e, tp) {
            return false;
        }
        let Some((mg, mustg, min, max)) = Link::materials::<P>(duel, a, c, tp) else {
            return false;
        };
        if mustg.is_exists(duel, |mc, duel| !Link::condition_filter::<P>(mc, duel, c, tp), 1, None) || mustg.len() > max {
            return false;
        }
        let search = |duel: &Duel, emt: &ExtraMaterials, tg: CardSet| {
            let tg = tg.filter(duel, |mc, duel| Link::condition_filter::<P>(mc, duel, c, tp), None);
            let mg_tg = mg.union(tg);
            if !(mg_tg.includes(mustg) && mustg.len() <= max) {
                return false;
            }
            let s = Search {
                tp,
                mg: mg_tg,
                og: mg,
                lc: c,
                minc: min,
                maxc: max,
                emt,
            };
            let recursive = |sg: CardSet| move |tc, duel: &Duel| Link::check_recursive::<P>(duel, tc, sg, &s, &Filters::default());
            if mustg.len() == max {
                mustg.is_exists(duel, recursive(CardSet::default()), 1, None)
            } else {
                mg_tg.is_exists(duel, recursive(mustg), 1, Some(mustg))
            }
        };
        // Not in the Lua: the copy of the duel that lets the Lua register effects is made only when an effect may.
        if !duel.may_have_effect(EFFECT_EXTRA_MATERIAL) {
            return search(duel, &ExtraMaterials::new(), CardSet::default());
        }
        duel.check_mut(|duel| {
            let (emt, tg) = aux::get_extra_materials(duel, tp, mustg.union(mg), c, SUMMON_TYPE_LINK);
            let res = search(duel, &emt, tg);
            aux::delete_extra_material_groups(duel, &emt);
            res
        })
    }

    fn target<P: LinkProcedure>(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
        let (e, tp) = (a.e, a.tp);
        let Some(c) = a.c else { return Ok(false) };
        if !P::spcon(duel, e, tp) {
            return Ok(false);
        }
        let Some((mg, mustg, min, max)) = Link::materials::<P>(duel, a, c, tp) else {
            return Ok(false);
        };
        let (emt, tg) = aux::get_extra_materials(duel, tp, mustg.union(mg), c, SUMMON_TYPE_LINK);
        let tg = tg.filter(duel, |mc, duel| Link::condition_filter::<P>(mc, duel, c, tp), None);
        let mut sg = mustg;
        let mg_tg = mg.union(tg);
        let s = Search {
            tp,
            mg: mg_tg,
            og: mg,
            lc: c,
            minc: min,
            maxc: max,
            emt: &emt,
        };
        let filters = |duel: &Duel, sg: CardSet| {
            let filters = Filters::default();
            if let Some(first) = sg.first() {
                Link::check_recursive2::<P>(duel, first, CardSet::default(), sg, &s, &filters);
            }
            filters.into_inner()
        };
        while sg.len() < max {
            let filt = filters(duel, sg);
            let shared = RefCell::new(filt.clone());
            let cg = mg_tg.filter(duel, |tc, duel| Link::check_recursive::<P>(duel, tc, sg, &s, &shared), Some(sg));
            if cg.is_empty() {
                break;
            }
            let finish = sg.len() >= min && sg.len() <= max && Link::check_goal::<P>(duel, sg, &s, &filt);
            let cancel = duel.is_summon_cancelable() && sg.is_empty();
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_LMATERIAL);
            let Some(tc) = cg.select_unselect(duel, Some(sg), tp, finish, cancel, 1, 1)? else {
                break;
            };
            if mustg.is_empty() || !mustg.is_contains(tc) {
                if !sg.is_contains(tc) {
                    sg.insert(tc);
                } else {
                    sg.remove(tc);
                }
            }
        }
        if sg.is_empty() {
            aux::delete_extra_material_groups(duel, &emt);
            return Ok(false);
        }
        let filt = filters(duel, sg);
        e.set_label_object_group(duel, Some(sg));
        aux::keep_extra_materials(duel, e, &emt, &filt);
        Ok(true)
    }

    fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
        let Some(c) = a.c else { return Ok(()) };
        let (e, tp) = (a.e, a.tp);
        let g = e.get_label_object_group(duel).unwrap_or_default();
        let (emt, filt) = aux::kept_extra_materials(duel, e, tp);
        for &(eg, te) in filt.iter().filter_map(|&index| emt.get(index)) {
            if let Value::ExtraMaterial(value) = te.get_value(duel) {
                let sg = eg.intersection(g);
                value(
                    duel,
                    te,
                    ExtraMaterialCall::Use {
                        summon_type: SUMMON_TYPE_LINK,
                        sg,
                        sc: c,
                        tp,
                    },
                );
                if te.check_count_limit(duel, tp) {
                    te.use_count_limit(duel, tp);
                }
            }
        }
        unported!(
            duel,
            g.is_exists(duel, |tc, duel| tc.is_has_effect(duel, EFFECT_FORMUD_SKIPPER).is_some(), 1, None)
        );
        c.set_material(duel, Some(g));
        duel.send_to_grave(g, REASON_MATERIAL + REASON_LINK)?;
        aux::delete_extra_material_groups(duel, &emt);
        Ok(())
    }
}
