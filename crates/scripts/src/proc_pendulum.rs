// Port of proc_pendulum.lua; no card passes a pendulum_location, a reset or a pendulum_flag.
use engine::{Args, CardId, CardSet, ConditionFn, Duel, EffectId, GroupOperationFn, Script};

use crate::chain::DuelChain;
use crate::constant::*;
use crate::utility::{CardUtility, DuelUtility, aux};

pub struct Pendulum;

// What a card passes to CreateAdditionalPendulumSummonEffect and CreateHarmonicOscillationEffect.
pub trait ExtraPendulum {
    const DESC: u64;
    const OWNER: u32;

    fn filter(c: CardId, duel: &Duel) -> bool;
}

impl Pendulum {
    pub fn add_procedure(duel: &mut Duel, c: CardId) {
        Pendulum::add_procedure_full(duel, c, true, None);
    }

    // `reg` registers the activation; `desc` None takes 1163.
    pub fn add_procedure_full(duel: &mut Duel, c: CardId, reg: bool, desc: Option<u64>) {
        let property = EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_CANNOT_DISABLE;
        let e1 = Pendulum::summon_effect(duel, c, desc.unwrap_or(1163), property, Pendulum::condition, Pendulum::operation);
        c.register_effect(duel, e1);
        if reg {
            let e2 = duel.create_effect(c);
            e2.set_description(duel, 1160);
            e2.set_type(duel, EFFECT_TYPE_ACTIVATE);
            e2.set_code(duel, EVENT_FREE_CHAIN);
            e2.set_range(duel, LOCATION_HAND);
            c.register_effect(duel, e2);
        }
    }

    // Pendulum.CreateAdditionalPendulumSummonEffect with the hand and the Extra Deck, left unregistered.
    pub fn create_additional_pendulum_summon_effect<P: ExtraPendulum>(duel: &mut Duel, handler: CardId) -> EffectId {
        let property = EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_CANNOT_DISABLE;
        let (condition, operation) = (Pendulum::additional_condition::<P>, Pendulum::additional_operation::<P>);
        let e1 = Pendulum::summon_effect(duel, handler, P::DESC, property, condition, operation);
        e1.set_reset_full(duel, RESET_PHASE | PHASE_END, 1);
        e1
    }

    // The first effect of Pendulum.CreateHarmonicOscillationEffect, flag CARD_ZEFRAATH; no caller registers the second.
    pub fn create_harmonic_oscillation_effect<P: ExtraPendulum>(duel: &mut Duel, handler: CardId) -> EffectId {
        let property = EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_BOTH_SIDE;
        let (condition, operation) = (Pendulum::harmonic_condition::<P>, Pendulum::harmonic_operation::<P>);
        let e1 = Pendulum::summon_effect(duel, handler, P::DESC, property, condition, operation);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        e1
    }

    // Pendulum.GrantAdditionalPendulumSummon with the hand and the Extra Deck and the default reset, the owner as the flag.
    pub fn player_can_gain_additional_pendulum_summon(duel: &Duel, player: u8, effect_flag: u32) -> bool {
        !(duel.has_flag_effect(player, CARD_ZEFRAATH) || duel.has_flag_effect(player, effect_flag))
            && duel.is_turn_player(player)
            && !duel.is_phase(PHASE_END)
    }

    pub fn grant_additional_pendulum_summon<P: ExtraPendulum>(duel: &mut Duel, handler: CardId, tp: u8, hint_description: u64) {
        if !Pendulum::player_can_gain_additional_pendulum_summon(duel, tp, P::OWNER) {
            return;
        }
        aux::register_client_hint_full(duel, handler, 0, tp, 1, 0, Some(hint_description), RESET_PHASE | PHASE_END, 1);
        duel.register_flag_effect(tp, P::OWNER, RESET_PHASE | PHASE_END, 0, 1);
        let extra_pendulum_effect = Pendulum::create_additional_pendulum_summon_effect::<P>(duel, handler);
        let e1 = duel.create_effect(handler);
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_GRANT);
        e1.set_target_range(duel, LOCATION_SZONE, 0);
        e1.set_target_card(duel, |duel, _, c| c.is_location(duel, LOCATION_PZONE));
        e1.set_label_object(duel, Some(extra_pendulum_effect));
        e1.set_reset_full(duel, RESET_PHASE | PHASE_END, 1);
        duel.register_effect(e1, tp);
        let harmonic_effect = Pendulum::create_harmonic_oscillation_effect::<P>(duel, handler);
        let e2 = e1.clone_effect(duel);
        e2.set_target_range(duel, 0, LOCATION_SZONE);
        e2.set_label_object(duel, Some(harmonic_effect));
        duel.register_effect(e2, tp);
    }

    fn summon_effect(
        duel: &mut Duel,
        c: CardId,
        desc: u64,
        property: u32,
        condition: ConditionFn,
        operation: GroupOperationFn,
    ) -> EffectId {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_description(duel, desc);
        e1.set_code(duel, EFFECT_SPSUMMON_PROC_G);
        e1.set_property(duel, property);
        e1.set_range(duel, LOCATION_PZONE);
        e1.set_condition(duel, condition);
        e1.set_group_operation(duel, operation);
        e1.set_value(duel, SUMMON_TYPE_PENDULUM as i32);
        e1
    }

    fn select(duel: &mut Duel, tp: u8, g: CardSet, sg: &mut CardSet, counts: [&mut i32; 3]) -> Script<bool> {
        let [hand, extra, usable] = counts;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let cancelable = duel.is_summon_cancelable();
        let Some(tc) = g.select_unselect(duel, Some(*sg), tp, !sg.is_empty(), cancelable, 1, 1)? else {
            return Ok(false);
        };
        let step = if sg.is_contains(tc) {
            sg.remove(tc);
            1
        } else {
            sg.insert(tc);
            -1
        };
        if tc.is_location(duel, LOCATION_HAND) {
            *hand += step;
        } else {
            *extra += step;
        }
        *usable += step;
        Ok(true)
    }

    // The idle command asks with only e and c, so `inchain` is nil. Pendulum::filter orders the scales.
    fn additional_condition<P: ExtraPendulum>(duel: &Duel, a: &Args) -> bool {
        let Some(c) = a.c else { return true };
        let tp = c.get_controler(duel);
        if duel.has_flag_effect(tp, CARD_ZEFRAATH) {
            return false;
        }
        let available_zones_count =
            duel.get_location_count(tp, LOCATION_MZONE) + duel.get_location_count_from_ex(tp, tp, None, None::<CardId>);
        if available_zones_count == 0 {
            return false;
        }
        let Some(rpz) = duel.get_field_card(tp, LOCATION_PZONE, 1).filter(|&rpz| rpz != c) else {
            return false;
        };
        let (lscale, rscale) = (c.get_left_scale(duel), rpz.get_right_scale(duel));
        let filter = |pc, duel: &Duel| Pendulum::filter(pc, duel, a.e, tp, lscale, rscale, false) && P::filter(pc, duel);
        duel.is_existing_matching_card(filter, tp, LOCATION_HAND | LOCATION_EXTRA, 0, 1, None)
    }

    // inchain is nil here as well.
    fn additional_operation<P: ExtraPendulum>(duel: &mut Duel, a: &Args, sg: &mut CardSet) -> Script<()> {
        let (e, tp) = (a.e, a.tp);
        let Some(c) = a.c else { return Ok(()) };
        let Some(rpz) = duel.get_field_card(tp, LOCATION_PZONE, 1) else {
            return Ok(());
        };
        let (lscale, rscale) = (c.get_left_scale(duel), rpz.get_right_scale(duel));
        let mut mmz_count = duel.get_location_count(tp, LOCATION_MZONE);
        let mut emz_count = duel.get_location_count_from_ex(tp, tp, None, None::<CardId>);
        let mut usable_mzone_count = duel.get_usable_mzone_count(tp);
        unported!(duel, duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some());
        let filter = |pc, duel: &Duel| Pendulum::filter(pc, duel, e, tp, lscale, rscale, false) && P::filter(pc, duel);
        let available = duel.get_matching_group(filter, tp, LOCATION_HAND | LOCATION_EXTRA, 0, None);
        let count = |duel: &Duel, location: u32| available.filter_count(duel, |pc, duel| pc.is_location(duel, location), None) as i32;
        mmz_count = mmz_count.min(count(duel, LOCATION_HAND));
        emz_count = emz_count.min(count(duel, LOCATION_EXTRA));
        emz_count = emz_count.min(aux::check_summon_gate(duel, tp).unwrap_or(emz_count));
        while usable_mzone_count != 0 {
            let location = (if mmz_count > 0 { LOCATION_HAND } else { 0 }) + if emz_count > 0 { LOCATION_EXTRA } else { 0 };
            let remaining = available.filter(duel, |pc, duel| pc.is_location(duel, location), Some(*sg));
            let counts = [&mut mmz_count, &mut emz_count, &mut usable_mzone_count];
            if remaining.is_empty() || !Pendulum::select(duel, tp, remaining, sg, counts)? {
                break;
            }
        }
        if !sg.is_empty() {
            duel.register_flag_effect(tp, CARD_ZEFRAATH, RESET_PHASE | PHASE_END, 0, 1);
            duel.hint(HINT_CARD, 0, u64::from(P::OWNER));
            duel.hint_selection(c);
            duel.hint_selection(rpz);
        }
        Ok(())
    }

    fn harmonic_condition<P: ExtraPendulum>(duel: &Duel, a: &Args) -> bool {
        let Some(c) = a.c else { return true };
        let tp = a.e.get_owner_player(duel);
        if duel.get_location_count_from_ex(tp, tp, None, None::<CardId>) <= 0 || duel.has_flag_effect(tp, CARD_ZEFRAATH) {
            return false;
        }
        let Some(rpz) = duel.get_field_card(1 - tp, LOCATION_PZONE, 1) else {
            return false;
        };
        let label = |c: CardId| c.get_flag_effect_label(duel, CARD_HARMONIC_OSCILLATION);
        if !(c.has_flag_effect(duel, CARD_HARMONIC_OSCILLATION) && rpz.has_flag_effect(duel, CARD_HARMONIC_OSCILLATION))
            || label(c) != label(rpz)
        {
            return false;
        }
        let (lscale, rscale) = (c.get_left_scale(duel), rpz.get_right_scale(duel));
        let filter = |pc, duel: &Duel| Pendulum::filter(pc, duel, a.e, tp, lscale, rscale, false) && P::filter(pc, duel);
        duel.is_existing_matching_card(filter, tp, LOCATION_EXTRA, 0, 1, None)
    }

    fn harmonic_operation<P: ExtraPendulum>(duel: &mut Duel, a: &Args, sg: &mut CardSet) -> Script<()> {
        let e = a.e;
        let tp = e.get_owner_player(duel);
        let Some(c) = a.c else { return Ok(()) };
        let Some(rpz) = duel.get_field_card(1 - tp, LOCATION_PZONE, 1) else {
            return Ok(());
        };
        let (lscale, rscale) = (c.get_left_scale(duel), rpz.get_right_scale(duel));
        unported!(duel, duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some());
        let ft = duel.get_location_count_from_ex(tp, tp, None, None::<CardId>);
        let ft = ft.min(aux::check_summon_gate(duel, tp).unwrap_or(ft));
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let filter = |pc, duel: &Duel| Pendulum::filter(pc, duel, e, tp, lscale, rscale, false) && P::filter(pc, duel);
        let min = if duel.is_summon_cancelable() { 0 } else { 1 };
        sg.merge(duel.select_matching_card(tp, filter, tp, LOCATION_EXTRA, 0, min, ft.max(0) as u8, None)?);
        if !sg.is_empty() {
            duel.register_flag_effect(tp, CARD_ZEFRAATH, RESET_PHASE | PHASE_END, 0, 1);
            duel.hint(HINT_CARD, 0, u64::from(P::OWNER));
            duel.hint(HINT_CARD, 0, u64::from(CARD_HARMONIC_OSCILLATION));
            duel.hint_selection(c);
            duel.hint_selection(rpz);
        }
        Ok(())
    }

    pub fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8, lscale: u32, rscale: u32, lvchk: bool) -> bool {
        let (lscale, rscale) = if lscale > rscale { (rscale, lscale) } else { (lscale, rscale) };
        let lv = crate::official::pendulum_level(c.get_original_code(duel)).unwrap_or_else(|| c.get_level(duel));
        (c.is_location(duel, LOCATION_HAND) || (c.is_faceup(duel) && c.is_type(duel, TYPE_PENDULUM)))
            && (lvchk || (lv > lscale && lv < rscale) || c.is_has_effect(duel, 511004423).is_some())
            && c.is_can_be_special_summoned(duel, e, SUMMON_TYPE_PENDULUM, tp, false, false)
            && !c.is_forbidden(duel)
    }

    // Both scales ignore the range with 511007000.
    fn lvchk(duel: &Duel, c: CardId, rpz: CardId) -> bool {
        c.is_has_effect(duel, 511007000).is_some() && rpz.is_has_effect(duel, 511007000).is_some()
    }

    // The idle command asks with only e and c, so `inchain` is nil.
    fn condition(duel: &Duel, a: &Args) -> bool {
        let e = a.e;
        let Some(c) = a.c else { return true };
        let tp = c.get_controler(duel);
        let Some(rpz) = duel.get_field_card(tp, LOCATION_PZONE, 1).filter(|&rpz| rpz != c) else {
            return false;
        };
        if duel.get_flag_effect(tp, 10000000) > 0 {
            return false;
        }
        let lscale = c.get_left_scale(duel);
        let rscale = rpz.get_right_scale(duel);
        let mut loc = 0;
        if duel.get_location_count(tp, LOCATION_MZONE) > 0 {
            loc += LOCATION_HAND;
        }
        if duel.get_location_count_from_ex(tp, tp, None, TYPE_PENDULUM) > 0 {
            loc += LOCATION_EXTRA;
        }
        if loc == 0 {
            return false;
        }
        let g = duel.get_field_group(tp, loc, 0);
        let lvchk = Pendulum::lvchk(duel, c, rpz);
        g.is_exists(duel, |pc, duel| Pendulum::filter(pc, duel, e, tp, lscale, rscale, lvchk), 1, None)
    }

    // inchain is false: no ported card Pendulum Summons during a chain.
    fn operation(duel: &mut Duel, a: &Args, sg: &mut CardSet) -> Script<()> {
        let (e, tp) = (a.e, a.tp);
        let Some(c) = a.c else { return Ok(()) };
        let Some(rpz) = duel.get_field_card(tp, LOCATION_PZONE, 1) else {
            return Ok(());
        };
        let lscale = c.get_left_scale(duel);
        let rscale = rpz.get_right_scale(duel);
        let mut ft1 = duel.get_location_count(tp, LOCATION_MZONE);
        let mut ft2 = duel.get_location_count_from_ex(tp, tp, None, TYPE_PENDULUM);
        let mut ft = duel.get_usable_mzone_count(tp);
        unported!(duel, duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some());
        let location = |ft1: i32, ft2: i32| (if ft1 > 0 { LOCATION_HAND } else { 0 }) + if ft2 > 0 { LOCATION_EXTRA } else { 0 };
        let lvchk = Pendulum::lvchk(duel, c, rpz);
        let tg = duel.get_matching_group(
            |pc, duel| Pendulum::filter(pc, duel, e, tp, lscale, rscale, lvchk),
            tp,
            location(ft1, ft2),
            0,
            None,
        );
        let count = |duel: &Duel, location: u32| tg.filter_count(duel, |pc, duel| pc.is_location(duel, location), None) as i32;
        ft1 = ft1.min(count(duel, LOCATION_HAND));
        ft2 = ft2.min(count(duel, LOCATION_EXTRA));
        ft2 = ft2.min(aux::check_summon_gate(duel, tp).unwrap_or(ft2));
        loop {
            let loc = location(ft1, ft2);
            let g = tg.filter(duel, |pc, duel| pc.is_location(duel, loc), Some(*sg));
            if g.is_empty() || ft == 0 || !Pendulum::select(duel, tp, g, sg, [&mut ft1, &mut ft2, &mut ft])? {
                break;
            }
        }
        if !sg.is_empty() {
            duel.register_flag_effect(tp, 10000000, RESET_PHASE + PHASE_END + RESET_SELF_TURN, 0, 1);
            duel.hint_selection(c);
            duel.hint_selection(rpz);
        }
        Ok(())
    }
}
