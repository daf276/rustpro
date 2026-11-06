// Port of proc_pendulum.lua: Pendulum.AddProcedure, the activation in the Pendulum Zone and the Pendulum Summon.
// The additional Pendulum Summons and Harmonic Oscillation come with the cards that grant them.
use engine::{Args, CardId, CardSet, Duel, EffectId, Script};

use crate::constant::*;
use crate::utility::aux;

pub struct Pendulum;

impl Pendulum {
    pub fn add_procedure(duel: &mut Duel, c: CardId) {
        Pendulum::add_procedure_full(duel, c, true, None);
    }

    // `reg` registers the activation; `desc` None takes 1163.
    pub fn add_procedure_full(duel: &mut Duel, c: CardId, reg: bool, desc: Option<u64>) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_description(duel, desc.unwrap_or(1163));
        e1.set_code(duel, EFFECT_SPSUMMON_PROC_G);
        e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_range(duel, LOCATION_PZONE);
        e1.set_condition(duel, Pendulum::condition);
        e1.set_group_operation(duel, Pendulum::operation);
        e1.set_value(duel, SUMMON_TYPE_PENDULUM as i32);
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

    // No ported card sets pendulum_level.
    pub fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8, lscale: u32, rscale: u32, lvchk: bool) -> bool {
        let (lscale, rscale) = if lscale > rscale { (rscale, lscale) } else { (lscale, rscale) };
        let lv = c.get_level(duel);
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
        if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
            duel.reach_unported(concat!(file!(), ":", line!()));
        }
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
            if g.is_empty() || ft == 0 {
                break;
            }
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let cancelable = duel.is_summon_cancelable();
            let Some(tc) = g.select_unselect(duel, Some(*sg), tp, !sg.is_empty(), cancelable, 1, 1)? else {
                break;
            };
            let step = if sg.is_contains(tc) {
                sg.remove(tc);
                1
            } else {
                sg.insert(tc);
                -1
            };
            if tc.is_location(duel, LOCATION_HAND) {
                ft1 += step;
            } else {
                ft2 += step;
            }
            ft += step;
        }
        if !sg.is_empty() {
            duel.register_flag_effect(tp, 10000000, RESET_PHASE + PHASE_END + RESET_SELF_TURN, 0, 1);
            duel.hint_selection(c);
            duel.hint_selection(rpz);
        }
        Ok(())
    }
}
