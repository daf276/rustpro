// Cosmic Tree Irmistil
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};
use engine::constants::MSG_RECOVER;

const ID: u32 = 74139959;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 3;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }
}

pub const LISTED_NAMES: &[u32] = &[ID];
// Sangans (Effect Monsters) to Link Summon with, and Elfnote Lucina on the opponent's side, whose Special Summons gain Life Points.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, MZONE),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Message(MSG_RECOVER), Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    c.set_unique_on_field(duel, 1, 0, |c, duel| c.is_code(duel, ID), LOCATION_ONFIELD);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| {
        duel.is_main_phase() && a.eg.unwrap_or_default().is_exists(duel, opponent_summon(a.tp), 1, None)
    });
    e1.set_operation(duel, lpgainop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP | TIMINGS_CHECK_MONSTER_E);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |duel, _| !(duel.is_phase(PHASE_DAMAGE) && duel.is_damage_calculated()));
    e2.set_cost(duel, atkcost);
    e2.set_operation(duel, atkop);
    c.register_effect(duel, e2);
}

fn opponent_summon(tp: u8) -> impl Fn(CardId, &Duel) -> bool {
    aux::faceup_filter(move |c, duel| c.is_summon_player(duel, 1 - tp))
}

fn lpgainop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let val =
        a.eg.unwrap_or_default()
            .filter(duel, opponent_summon(tp), None)
            .get_sum(duel, |c, duel| i64::from(c.get_attack(duel)));
    if !duel.is_chain_solving() {
        if val > 0 {
            duel.hint(HINT_CARD, 1 - tp, ID.into());
            duel.recover(tp, val as u32, REASON_EFFECT)?;
        }
    } else if let Some(eff) = e.get_label_object(duel).filter(|eff| !eff.is_deleted(duel)) {
        let label = eff.get_label(duel);
        eff.set_label(duel, label + val);
    } else {
        let c = e.get_handler(duel);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EVENT_CHAIN_SOLVED);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_operation(duel, chainsolvedop);
        e1.set_label(duel, val);
        e1.set_label_object(duel, Some(e));
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_CHAIN);
        c.register_effect(duel, e1);
        e.set_label_object(duel, Some(e1));
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e2.set_code(duel, EVENT_CHAIN_SOLVED);
        e2.set_operation(duel, |duel, a| {
            if let Some(e) = a.e.get_label_object(duel) {
                e.set_label_object(duel, None);
            }
            Ok(())
        });
        e2.set_label_object(duel, Some(e));
        e2.set_reset(duel, RESET_CHAIN);
        duel.register_effect(e2, tp);
    }
    Ok(())
}

fn chainsolvedop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let val = e.get_label(duel);
    if val > 0 {
        duel.hint(HINT_CARD, 1 - tp, ID.into());
        duel.recover(tp, val as u32, REASON_EFFECT)?;
    }
    let origin = e.get_label_object(duel);
    e.reset(duel);
    if let Some(origin) = origin {
        origin.set_label_object(duel, None);
    }
    Ok(())
}

fn atkcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.check_lp_cost(tp, 1000));
    }
    let cost_options: Vec<u64> = (1..=duel.get_lp(tp).min(3000) as u64 / 1000).map(|i| i * 1000).collect();
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 1));
    let lp_cost = duel.announce_number(tp, &cost_options)?;
    e.set_label(duel, lp_cost as i64);
    duel.pay_lp_cost(tp, lp_cost as u32)?;
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, e.get_label(duel) as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
    }
    Ok(())
}
