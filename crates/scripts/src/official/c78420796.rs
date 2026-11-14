// Dinomorphia Frenzy
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND};

const ID: u32 = 78420796;

pub const LISTED_SERIES: &[u16] = &[SET_DINOMORPHIA];

struct Params;

impl FusionSummonParams for Params {
    fn fusfilter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_DINOMORPHIA])
    }

    fn matfilter(_: CardId, _: &Duel, _: EffectId, _: u8) -> bool {
        false
    }

    fn extrafil(duel: &Duel, e: EffectId, tp: u8, _: CardSet) -> (Option<CardSet>, Option<FusionCheck>) {
        let g = duel.get_matching_group(
            |c, duel| Fusion::is_monster_filter(c, duel, CardId::is_able_to_grave),
            tp,
            LOCATION_DECK | LOCATION_EXTRA,
            0,
            None,
        );
        (Some(g), Some((fcheck, e.get_handler(duel))))
    }

    fn extratg(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
        if chk == 0 {
            return Ok(true);
        }
        duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 2, a.tp, (LOCATION_DECK | LOCATION_EXTRA) as i32);
        Ok(false)
    }
}

// Set copies, Therizia in the Deck and Kentregina in the Extra Deck as materials for Stealthbergia or Rexterm.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (92133240, DECK),
    (92133240, DECK),
    (48832775, EXTRA),
    (48832775, EXTRA),
    (74936480, EXTRA),
    (92798873, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(1170), Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Fusion::create_summon_eff::<Params>(duel, c);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_hint_timing(duel, 0, TIMING_MAIN_END);
    e1.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp) && duel.is_main_phase());
    e1.set_cost(duel, cost);
    c.register_effect(duel, e1);
    register_no_damage::<ID>(duel, c);
}

// The GY effect Frenzy and Domain share.
pub fn register_no_damage<const CODE: u32>(duel: &mut Duel, c: CardId) {
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(CODE, 1));
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_condition(duel, |duel, a| a.rp == 1 - a.tp && duel.get_lp(a.tp) <= 2000);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_operation(duel, nodamop);
    c.register_effect(duel, e2);
}

fn fcheck(duel: &Duel, _: CardId, _: u8, sg: CardSet, _: CardId) -> bool {
    sg.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_DECK), None) <= 1
        && sg.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_EXTRA), None) <= 1
}

pub fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let lp = duel.get_lp(a.tp) / 2;
    duel.pay_lp_cost(a.tp, lp as u32)?;
    Ok(false)
}

fn nodamop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CHANGE_DAMAGE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 1, 0);
    e1.set_value(duel, Value::ChangeDamage(damval));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, a.tp);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_NO_EFFECT_DAMAGE);
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e2, a.tp);
    Ok(())
}

fn damval(duel: &mut Duel, e: EffectId, _: Option<EffectId>, val: u32, r: u32, rp: u8, _: Option<CardId>) -> u32 {
    if r & REASON_EFFECT != 0 && rp != e.get_owner_player(duel) {
        0
    } else {
        val
    }
}
