// Port of proc_link.lua: Link.AddProcedure with its condition, target and operation. Extra materials
// (EFFECT_EXTRA_MATERIAL, whose filters the target collects with Link.CheckRecursive2) and EFFECT_FORMUD_SKIPPER come
// with the cards that grant them, as does `spcon`.
use engine::{Args, CardId, CardSet, Duel, Script};

use crate::constant::*;
use crate::utility::{CardUtility, aux};

pub struct Link;

// The parameters of Link.AddProcedure (f, min, max, specialchk): `MAX` None is the monster's Link Rating, a nil filter
// passes every card and a nil specialchk every group.
pub trait LinkProcedure {
    const MIN: usize;
    const MAX: Option<usize> = None;

    fn f(_c: CardId, _duel: &Duel, _lc: CardId, _sumtype: u64, _tp: u8) -> bool {
        true
    }

    fn specialchk(_duel: &Duel, _sg: CardSet, _lc: CardId, _sumtype: u64, _tp: u8) -> bool {
        true
    }
}

const LINK_MATERIAL: u64 = SUMMON_TYPE_LINK as u64 | MATERIAL_LINK;

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

    // A card outside `og` would be an extra material; without extra materials it is never valid.
    fn check_recursive<P: LinkProcedure>(
        duel: &Duel,
        c: CardId,
        tp: u8,
        sg: CardSet,
        mg: CardSet,
        lc: CardId,
        minc: usize,
        maxc: usize,
        og: CardSet,
    ) -> bool {
        if sg.len() > maxc {
            return false;
        }
        let sg = sg.union(c.into());
        if !og.is_contains(c) {
            return false;
        }
        Link::check_goal::<P>(duel, tp, sg, lc, minc)
            || (sg.len() < maxc
                && mg.is_exists(
                    duel,
                    |tc, duel| Link::check_recursive::<P>(duel, tc, tp, sg, mg, lc, minc, maxc, og),
                    1,
                    Some(sg),
                ))
    }

    fn check_goal<P: LinkProcedure>(duel: &Duel, tp: u8, sg: CardSet, lc: CardId, minc: usize) -> bool {
        sg.len() >= minc
            && sg.check_with_sum_equal(duel, Link::get_link_count, lc.get_link(duel), sg.len(), sg.len())
            && P::specialchk(duel, sg, lc, LINK_MATERIAL, tp)
            && duel.get_location_count_from_ex(tp, tp, Some(sg), lc) > 0
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
        aux::get_extra_materials(duel, tp, mustg.union(mg), c, SUMMON_TYPE_LINK);
        Some((mg, mustg, min, max))
    }

    fn condition<P: LinkProcedure>(duel: &Duel, a: &Args) -> bool {
        let Some(c) = a.c else { return true };
        if c.is_type(duel, TYPE_PENDULUM) && c.is_faceup(duel) {
            return false;
        }
        let tp = c.get_controler(duel);
        let Some((mg, mustg, min, max)) = Link::materials::<P>(duel, a, c, tp) else {
            return false;
        };
        if mustg.is_exists(duel, |mc, duel| !Link::condition_filter::<P>(mc, duel, c, tp), 1, None) || mustg.len() > max {
            return false;
        }
        let mg_tg = mg;
        if !(mg_tg.includes(mustg) && mustg.len() <= max) {
            return false;
        }
        let recursive = |sg: CardSet| move |tc, duel: &Duel| Link::check_recursive::<P>(duel, tc, tp, sg, mg_tg, c, min, max, mg);
        if mustg.len() == max {
            mustg.is_exists(duel, recursive(CardSet::default()), 1, None)
        } else {
            mg_tg.is_exists(duel, recursive(mustg), 1, Some(mustg))
        }
    }

    fn target<P: LinkProcedure>(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
        let (e, tp) = (a.e, a.tp);
        let Some(c) = a.c else { return Ok(false) };
        let Some((mg, mustg, min, max)) = Link::materials::<P>(duel, a, c, tp) else {
            return Ok(false);
        };
        let mut sg = mustg;
        let mg_tg = mg;
        while sg.len() < max {
            let cg = mg_tg.filter(
                duel,
                |tc, duel| Link::check_recursive::<P>(duel, tc, tp, sg, mg_tg, c, min, max, mg),
                Some(sg),
            );
            if cg.is_empty() {
                break;
            }
            let finish = sg.len() >= min && sg.len() <= max && Link::check_goal::<P>(duel, tp, sg, c, min);
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
            return Ok(false);
        }
        e.set_label_object_group(duel, Some(sg));
        Ok(true)
    }

    fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
        let Some(c) = a.c else { return Ok(()) };
        let g = a.e.get_label_object_group(duel).unwrap_or_default();
        unported!(
            duel,
            g.is_exists(duel, |tc, duel| tc.is_has_effect(duel, EFFECT_FORMUD_SKIPPER).is_some(), 1, None)
        );
        c.set_material(duel, Some(g));
        duel.send_to_grave(g, REASON_MATERIAL + REASON_LINK)?;
        Ok(())
    }
}
