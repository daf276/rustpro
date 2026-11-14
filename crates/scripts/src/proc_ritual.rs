// Port of proc_ritual.lua; customoperation, specificmatfilter, requirementfunc and self have no ported caller.
use std::marker::PhantomData;

use engine::{Args, CardId, CardSet, Duel, EffectId, Script, Value};

use crate::constant::*;
use crate::utility::aux;

pub struct Ritual;

// Whether sg may be the materials, and whether no larger group may either.
pub type ForcedSelection = fn(&Duel, EffectId, u8, CardSet, CardId) -> (bool, bool);

// The named parameters of Ritual.CreateProc, Target and Operation besides lvtype; nil ones keep the defaults below.
pub trait RitualParams {
    const DESC: u64 = 1171;
    const LOCATION: Option<u32> = None;
    const SUMPOS: u32 = POS_FACEUP;
    const FORCEDSELECTION: Option<ForcedSelection> = None;

    fn filter(_c: CardId, _duel: &Duel) -> bool {
        true
    }

    // filter, for one that reads the handler the Lua closure captures.
    fn filter_with_effect(c: CardId, duel: &Duel, _e: EffectId) -> bool {
        Self::filter(c, duel)
    }

    // lv, a level or a function of the Ritual Monster; nil is its level.
    fn lv(c: CardId, duel: &Duel) -> u32 {
        c.get_level(duel)
    }

    fn extrafil(_duel: &Duel, _e: EffectId, _tp: u8) -> Option<CardSet> {
        None
    }

    fn extraop(duel: &mut Duel, mat: CardSet, _e: EffectId, _tp: u8, _tc: CardId) -> Script<()> {
        duel.release_ritual_material(mat)
    }

    fn matfilter(_c: CardId, _duel: &Duel, _e: EffectId, _tp: u8) -> bool {
        true
    }

    fn stage2(_duel: &mut Duel, _mat: CardSet, _e: EffectId, _tp: u8, _tc: CardId) -> Script<()> {
        Ok(())
    }

    fn extratg(_duel: &mut Duel, _a: &Args, _chk: i32) -> Script<bool> {
        Ok(true)
    }
}

// The arguments of Ritual.AddProcCode after the type: lv, desc and the codes of the Ritual Monsters.
pub trait RitualCode {
    const LV: Option<u32> = None;
    const DESC: Option<u64> = None;
    const CODES: &'static [u32];
}

struct Code<P>(PhantomData<P>);

impl<P: RitualCode> RitualParams for Code<P> {
    const DESC: u64 = match P::DESC {
        Some(desc) => desc,
        None => 1171,
    };

    fn filter(c: CardId, duel: &Duel) -> bool {
        P::CODES.iter().any(|&code| c.is_code(duel, code))
    }

    fn lv(c: CardId, duel: &Duel) -> u32 {
        P::LV.unwrap_or_else(|| c.get_level(duel))
    }
}

// The condition of Ritual.AddWholeLevelTribute, condition(c, e) for the Ritual Monster c.
pub trait WholeLevelTribute {
    fn condition(c: CardId, duel: &Duel, e: EffectId) -> bool;
}

fn location<P: RitualParams>(duel: &Duel) -> u32 {
    P::LOCATION.unwrap_or(match duel.is_duel_type(DUEL_EXTRA_DECK_RITUAL) {
        true => LOCATION_EXTRA,
        false => LOCATION_HAND,
    })
}

impl Ritual {
    pub fn add_whole_level_tribute<T: WholeLevelTribute>(duel: &mut Duel, c: CardId) -> EffectId {
        let e = duel.create_effect(c);
        e.set_type(duel, EFFECT_TYPE_SINGLE);
        e.set_code(duel, EFFECT_RITUAL_LEVEL);
        e.set_value(duel, Value::Card(Ritual::whole_level_tribute_value::<T>));
        c.register_effect(duel, e);
        e
    }

    fn whole_level_tribute_value<T: WholeLevelTribute>(duel: &Duel, e: EffectId, c: CardId) -> i32 {
        let lv = e.get_handler(duel).get_level(duel);
        if !T::condition(c, duel, e) {
            return lv as i32;
        }
        let clv = duel.get_ritual_summoning_level().unwrap_or_else(|| c.get_level(duel));
        ((lv << 16) | clv) as i32
    }

    pub fn create_proc<P: RitualParams, const LVTYPE: u32>(duel: &mut Duel, c: CardId) -> EffectId {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, P::DESC);
        e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
        e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
        e1.set_code(duel, EVENT_FREE_CHAIN);
        e1.set_target(duel, Ritual::target::<P, LVTYPE>);
        e1.set_operation(duel, Ritual::operation::<P, LVTYPE>);
        e1
    }

    pub fn add_proc<P: RitualParams, const LVTYPE: u32>(duel: &mut Duel, c: CardId) -> EffectId {
        let e1 = Ritual::create_proc::<P, LVTYPE>(duel, c);
        c.register_effect(duel, e1);
        e1
    }

    pub fn add_proc_greater<P: RitualParams>(duel: &mut Duel, c: CardId) -> EffectId {
        Ritual::add_proc::<P, RITPROC_GREATER>(duel, c)
    }

    pub fn add_proc_equal<P: RitualParams>(duel: &mut Duel, c: CardId) -> EffectId {
        Ritual::add_proc::<P, RITPROC_EQUAL>(duel, c)
    }

    // Ritual.AddProcGreaterCode; mt.fit_monster, which it sets, is the port's FIT_MONSTER (official/mod.rs).
    pub fn add_proc_greater_code<P: RitualCode>(duel: &mut Duel, c: CardId) -> EffectId {
        Ritual::add_proc::<Code<P>, RITPROC_GREATER>(duel, c)
    }

    pub fn add_proc_equal_code<P: RitualCode>(duel: &mut Duel, c: CardId) -> EffectId {
        Ritual::add_proc::<Code<P>, RITPROC_EQUAL>(duel, c)
    }

    // Auxiliary.RitualCheckAdditionalLevel.
    pub fn check_additional_level(c: CardId, duel: &Duel, rc: CardId) -> u32 {
        let raw_level = c.get_ritual_level(duel, rc);
        let (lv1, lv2) = (raw_level & 0xffff, raw_level >> 16);
        if lv2 > 0 { lv1.min(lv2) } else { lv1 }
    }

    // The materials Target and Operation start from, mg and mg2 after Ritual.CheckMatFilter.
    fn materials<P: RitualParams>(duel: &Duel, e: EffectId, tp: u8) -> (CardSet, CardSet) {
        let mut mg = duel.get_ritual_material(tp, true);
        let mut mg2 = P::extrafil(duel, e, tp).unwrap_or_default();
        let extra_eff_g = mg.filter(duel, |c, duel| c.is_has_effect(duel, EFFECT_EXTRA_RITUAL_MATERIAL).is_some(), None);
        let extra_release = |c: CardId, duel: &Duel| c.is_controler(duel, 1 - tp) && c.is_has_effect(duel, EFFECT_EXTRA_RELEASE).is_some();
        let extra_mat_g = mg.filter(duel, extra_release, None);
        unported!(duel, !extra_eff_g.is_empty() || !extra_mat_g.is_empty());
        mg.keep_matching(duel, |c, duel| P::matfilter(c, duel, e, tp), None);
        mg2.keep_matching(duel, |c, duel| P::matfilter(c, duel, e, tp), None);
        (mg, mg2)
    }

    // Ritual.Filter, with the forcedselection its caller merged.
    fn filter<P: RitualParams>(
        c: CardId,
        duel: &Duel,
        lvtype: u32,
        e: EffectId,
        tp: u8,
        m: CardSet,
        m2: CardSet,
        forcedselection: Option<ForcedSelection>,
    ) -> bool {
        if !(c.is_original_type(duel, TYPE_RITUAL) && c.is_original_type(duel, TYPE_MONSTER))
            || !P::filter_with_effect(c, duel, e)
            || !c.is_can_be_special_summoned_full(duel, e, SUMMON_TYPE_RITUAL, tp, false, true, P::SUMPOS, tp, 0xff)
        {
            return false;
        }
        let lv = P::lv(c, duel).max(1);
        duel.set_ritual_summoning_level(Some(lv));
        let mut mg = m.filter(duel, |mc, duel| mc.is_can_be_ritual_material(duel, Some(c)), Some(c.into()));
        mg.merge(m2.difference(c.into()));
        Ritual::keep_mat_filter(duel, &mut mg, c, tp);
        let check = Ritual::check(c, lv, forcedselection, lvtype, e, tp);
        let res = aux::select_unselect_group_check(duel, mg, 1, lv as usize, &check);
        duel.set_ritual_summoning_level(None);
        res
    }

    // mg:Match(c.mat_filter, c, tp), when the Ritual Monster c's script has a mat_filter.
    fn keep_mat_filter(duel: &Duel, mg: &mut CardSet, c: CardId, tp: u8) {
        if let Some(mat_filter) = crate::official::mat_filter(c.get_original_code(duel)) {
            mg.keep_matching(duel, |mc, duel| mat_filter(mc, duel, tp), Some(c.into()));
        }
    }

    fn sum_check(duel: &Duel, sg: CardSet, sc: CardId, lv: u32, lvtype: u32) -> bool {
        let level = |c: CardId, duel: &Duel| c.get_ritual_level(duel, sc);
        if lvtype == RITPROC_EQUAL {
            return sg.check_with_sum_equal(duel, level, lv, sg.len(), sg.len());
        }
        duel.set_selected_card(sg);
        sg.check_with_sum_greater(duel, level, lv)
    }

    // Ritual.Check, as the rescon of aux.SelectUnselectGroup.
    fn check(
        sc: CardId,
        lv: u32,
        forcedselection: Option<ForcedSelection>,
        lvtype: u32,
        e: EffectId,
        tp: u8,
    ) -> impl Fn(&Duel, CardSet, CardSet, Option<CardId>) -> (bool, bool) {
        move |duel, sg, _, c| {
            let level = |c: CardId, duel: &Duel| i64::from(Ritual::check_additional_level(c, duel, sc));
            let mut sum = sg.get_sum(duel, level);
            if lvtype != RITPROC_EQUAL {
                sum -= c.map_or(0, |c| level(c, duel));
            }
            if sum > i64::from(lv) {
                return (false, true);
            }
            let (mut res, mut stop) = (true, false);
            if let Some(forcedselection) = forcedselection {
                (res, stop) = forcedselection(duel, e, tp, sg, sc);
            }
            if res && !stop {
                let count = match sc.is_location(duel, LOCATION_EXTRA) {
                    true => duel.get_location_count_from_ex(tp, tp, Some(sg), sc),
                    false => duel.get_mzone_count_full(tp, Some(sg), tp, LOCATION_REASON_TOFIELD, 0xff),
                };
                res = Ritual::sum_check(duel, sg, sc, lv, lvtype) && count > 0;
            }
            (res, stop)
        }
    }

    fn finishcon(
        sc: CardId,
        lv: u32,
        forcedselection: Option<ForcedSelection>,
        lvtype: u32,
        e: EffectId,
        tp: u8,
    ) -> impl Fn(&Duel, CardSet, CardSet, Option<CardId>) -> (bool, bool) {
        move |duel, sg, _, _| {
            if forcedselection.is_some_and(|forcedselection| !forcedselection(duel, e, tp, sg, sc).0) {
                return (false, false);
            }
            (Ritual::sum_check(duel, sg, sc, lv, lvtype), false)
        }
    }

    pub fn target<P: RitualParams, const LVTYPE: u32>(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
        let (e, tp) = (a.e, a.tp);
        let location = location::<P>(duel);
        if chk == 0 {
            let (mg, mg2) = Ritual::materials::<P>(duel, e, tp);
            let filter = |c, duel: &Duel| Ritual::filter::<P>(c, duel, LVTYPE, e, tp, mg, mg2, P::FORCEDSELECTION);
            return Ok(duel.is_existing_matching_card(filter, tp, location, 0, 1, None));
        }
        P::extratg(duel, a, chk)?;
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, location as i32);
        Ok(false)
    }

    pub fn operation<P: RitualParams, const LVTYPE: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
        let (e, tp) = (a.e, a.tp);
        let location = location::<P>(duel);
        let (mut mg, mg2) = Ritual::materials::<P>(duel, e, tp);
        let func = P::FORCEDSELECTION;
        let ft = duel.get_location_count(tp, LOCATION_MZONE);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let filter = aux::necro_valley_filter(|c, duel| Ritual::filter::<P>(c, duel, LVTYPE, e, tp, mg, mg2, func));
        let Some(tc) = duel.select_matching_card(tp, filter, tp, location, 0, 1, 1, None)?.get_first() else {
            return Ok(());
        };
        let lv = P::lv(tc, duel).max(1);
        duel.set_ritual_summoning_level(Some(lv));
        mg.keep_matching(duel, |c, duel| c.is_can_be_ritual_material(duel, Some(tc)), Some(tc.into()));
        mg.merge(mg2.difference(tc.into()));
        Ritual::keep_mat_filter(duel, &mut mg, tc, tp);
        let mat = if ft > 0 && func.is_none() {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_RELEASE);
            let level = |c: CardId, duel: &Duel| c.get_ritual_level(duel, tc);
            match LVTYPE {
                RITPROC_EQUAL => mg.select_with_sum_equal(duel, tp, level, lv, 1, mg.len() as i32)?,
                _ => mg.select_with_sum_greater(duel, tp, level, lv)?,
            }
        } else {
            let check = Ritual::check(tc, lv, func, LVTYPE, e, tp);
            let finishcon = Ritual::finishcon(tc, lv, func, LVTYPE, e, tp);
            aux::select_unselect_group_full(duel, mg, 1, lv as usize, &check, tp, HINTMSG_RELEASE, Some(&finishcon), false)?
        };
        let extra_eff_g = mat.filter(duel, |c, duel| c.is_has_effect(duel, EFFECT_EXTRA_RITUAL_MATERIAL).is_some(), None);
        unported!(duel, !extra_eff_g.is_empty());
        tc.set_material(duel, Some(mat));
        P::extraop(duel, mat, e, tp, tc)?;
        duel.break_effect()?;
        duel.special_summon(tc, SUMMON_TYPE_RITUAL, tp, tp, false, true, P::SUMPOS)?;
        tc.complete_procedure(duel);
        if tc.is_facedown(duel) {
            duel.confirm_cards(1 - tp, tc)?;
        }
        P::stage2(duel, mat, e, tp, tc)?;
        duel.set_ritual_summoning_level(None);
        Ok(())
    }
}
