// Port of proc_link.lua; EFFECT_FORMUD_SKIPPER comes with the card that grants it.
use engine::{Args, CardId, CardSet, Duel, EffectId, ExtraMaterialCall, Script, Value};

use crate::constant::*;
use crate::proc_xyz::must_group;
use crate::utility::aux::ExtraMaterials;
use crate::utility::{CardUtility, aux};

pub struct Link;

// The parameters of Link.AddProcedure: `MAX` None is the Link Rating, a nil filter, specialchk or spcon passes.
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

// What the recursive checks share: mg is all materials (mg+tg), og those that are not extra materials.
struct Search<'a> {
    tp: u8,
    mg: CardSet,
    og: CardSet,
    lc: CardId,
    minc: usize,
    maxc: usize,
    emt: &'a ExtraMaterials,
}

impl<'a> Search<'a> {
    fn new(tp: u8, mg: CardSet, og: CardSet, lc: CardId, minc: usize, maxc: usize, emt: &'a ExtraMaterials) -> Self {
        Self {
            tp,
            mg,
            og,
            lc,
            minc,
            maxc,
            emt,
        }
    }
}

// Writes only when extra material operations can run (W:P Fancy Ball's calls values that register flags).
enum SearchDuel<'a> {
    Read(&'a Duel),
    Write(&'a mut Duel),
}

impl SearchDuel<'_> {
    fn read(&self) -> &Duel {
        match self {
            SearchDuel::Read(duel) => duel,
            SearchDuel::Write(duel) => duel,
        }
    }

    // Only extra material operations write, and a search that only reads has none.
    fn write(&mut self) -> Option<&mut Duel> {
        match self {
            SearchDuel::Read(duel) => {
                unported!(duel, true);
                None
            }
            SearchDuel::Write(duel) => Some(duel),
        }
    }
}

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

    // c joins sg if the used extra materials' filters take it and, unless it is in og, an extra material does (into filt).
    fn joins(duel: &mut SearchDuel, c: CardId, sg: CardSet, s: &Search, filt: &mut Vec<usize>) -> bool {
        let passes = |&index: &usize| {
            s.emt
                .get(index)
                .is_none_or(|&ex| (duel.write()).is_some_and(|duel| aux::extra_material_filter(duel, ex, c, s.tp, sg, s.mg, s.lc, 1)))
        };
        filt.iter().all(passes)
            && (s.og.is_contains(c)
                || (duel.write()).is_some_and(|duel| aux::check_valid_extra(duel, c, s.tp, sg, s.mg, s.lc, s.emt, filt)))
    }

    fn check_recursive<P: LinkProcedure>(duel: &mut SearchDuel, c: CardId, sg: CardSet, s: &Search, filt: &mut Vec<usize>) -> bool {
        if sg.len() > s.maxc {
            return false;
        }
        let sg = sg.union(c.into());
        if !Link::joins(duel, c, sg, s, filt) {
            return false;
        }
        let filt = filt.clone();
        Link::check_goal::<P>(duel, sg, s, &filt) || (sg.len() < s.maxc && Link::any_recursive::<P>(duel, s.mg, sg, s, filt))
    }

    // mg:IsExists(Link.CheckRecursive, 1, sg, ..., {table.unpack(filt)}): one copy of filt, shared by all the calls.
    fn any_recursive<P: LinkProcedure>(duel: &mut SearchDuel, mg: CardSet, sg: CardSet, s: &Search, mut filt: Vec<usize>) -> bool {
        mg.difference(sg)
            .iter()
            .any(|tc| Link::check_recursive::<P>(duel, tc, sg, s, &mut filt))
    }

    // Link.CheckRecursive2 with secondg as mg: walks sg2 into sg, collecting the extra materials it uses in filt.
    fn check_recursive2<P: LinkProcedure>(
        duel: &mut SearchDuel,
        c: CardId,
        sg: CardSet,
        sg2: CardSet,
        s: &Search,
        filt: &mut Vec<usize>,
    ) -> bool {
        if sg.len() > s.maxc {
            return false;
        }
        let sg = sg.union(c.into());
        if !Link::joins(duel, c, sg, s, filt) {
            return false;
        }
        match sg2.difference(sg).first() {
            None if !s.mg.is_empty() => Link::any_recursive::<P>(duel, s.mg, sg, s, filt.clone()),
            None => Link::check_goal::<P>(duel, sg, s, filt),
            Some(next) => Link::check_recursive2::<P>(duel, next, sg, sg2, s, filt),
        }
    }

    fn check_goal<P: LinkProcedure>(duel: &mut SearchDuel, sg: CardSet, s: &Search, filt: &[usize]) -> bool {
        let none = CardSet::default();
        let taken = |&index: &usize| {
            s.emt.get(index).is_none_or(|&ex| {
                sg.iter()
                    .any(|c| (duel.write()).is_some_and(|duel| aux::extra_material_filter(duel, ex, c, s.tp, sg, none, s.lc, 1)))
            })
        };
        if !filt.iter().all(taken) {
            return false;
        }
        let duel = duel.read();
        sg.len() >= s.minc
            && sg.check_with_sum_equal(duel, Link::get_link_count, s.lc.get_link(duel), sg.len(), sg.len())
            && P::specialchk(duel, sg, s.lc, LINK_MATERIAL, s.tp)
            && duel.get_location_count_from_ex(s.tp, s.tp, Some(sg), s.lc) > 0
    }

    // The materials the condition and the target start from, None if the forced counts are out of range.
    fn materials<P: LinkProcedure>(duel: &Duel, a: &Args, c: CardId, tp: u8) -> Option<(CardSet, CardSet, usize, usize)> {
        let (minc, maxc) = (P::MIN, Link::max::<P>(duel, c));
        let g = a
            .materials
            .unwrap_or_else(|| duel.get_matching_group(|c, duel| c.is_faceup(duel), tp, LOCATION_MZONE, 0, None));
        let mg = g.filter(duel, |mc, duel| Link::condition_filter::<P>(mc, duel, c, tp), None);
        let mustg = must_group(duel, tp, Some(g), c, mg, REASON_LINK, a.must);
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
        // Not in the Lua: the copy of the duel that lets the Lua register effects is made only when an effect may.
        if !duel.may_have_effect(EFFECT_EXTRA_MATERIAL) {
            let emt = ExtraMaterials::new();
            return Link::search::<P>(&mut SearchDuel::Read(duel), &Search::new(tp, mg, mg, c, min, max, &emt), mustg);
        }
        duel.check_mut(|duel| {
            let (emt, tg) = aux::get_extra_materials(duel, tp, mustg.union(mg), c, SUMMON_TYPE_LINK);
            let tg = tg.filter(duel, |mc, duel| Link::condition_filter::<P>(mc, duel, c, tp), None);
            let s = Search::new(tp, mg.union(tg), mg, c, min, max, &emt);
            let res = Link::search::<P>(&mut SearchDuel::Write(duel), &s, mustg);
            aux::delete_extra_material_groups(duel, &emt);
            res
        })
    }

    fn search<P: LinkProcedure>(duel: &mut SearchDuel, s: &Search, mustg: CardSet) -> bool {
        if !(s.mg.includes(mustg) && mustg.len() <= s.maxc) {
            return false;
        }
        let (candidates, sg) = if mustg.len() == s.maxc {
            (mustg, CardSet::default())
        } else {
            (s.mg.difference(mustg), mustg)
        };
        candidates
            .iter()
            .any(|tc| Link::check_recursive::<P>(duel, tc, sg, s, &mut Vec::new()))
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
        let s = Search::new(tp, mg_tg, mg, c, min, max, &emt);
        let filters = |duel: &mut Duel, sg: CardSet| {
            let mut filters = Vec::new();
            if let Some(first) = sg.first() {
                Link::check_recursive2::<P>(&mut SearchDuel::Write(duel), first, CardSet::default(), sg, &s, &mut filters);
            }
            filters
        };
        while sg.len() < max {
            let filt = filters(duel, sg);
            let mut shared = filt.clone();
            let cg: CardSet = (mg_tg.difference(sg).iter())
                .filter(|&tc| Link::check_recursive::<P>(&mut SearchDuel::Write(duel), tc, sg, &s, &mut shared))
                .collect();
            if cg.is_empty() {
                break;
            }
            let finish = sg.len() >= min && sg.len() <= max && Link::check_goal::<P>(&mut SearchDuel::Write(duel), sg, &s, &filt);
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
