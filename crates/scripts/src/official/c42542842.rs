// Dakki, the Graceful Mayakashi
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 42542842;

// In the Graveyard with Tsukahagi and Yuki-Musume (Mayakashi) on the field to Link Summon Yuki-Onna, the Ice Mayakashi.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (5325155, MZONE),
    (72700231, MZONE),
    (72700231, MZONE),
    (72700231, MZONE),
    (66870733, EXTRA),
    (66870733, EXTRA),
    (66870733, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.set_unique_on_field(duel, 1, 0, |c, duel| c.is_code(duel, ID), LOCATION_ONFIELD);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_condition(duel, spcon);
    e1.set_cost(duel, spcost::<ID>);
    e1.set_target(duel, stock::special_summon_self_target_player_0);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    duel.add_custom_activity_counter(ID, ACTIVITY_SPSUMMON, CounterFilter::Card(counterfilter));
    aux::add_continuous_lizard_check(duel, c, LOCATION_MZONE, lizfilter);
}

pub fn counterfilter(duel: &Duel, c: CardId) -> bool {
    !c.is_summon_location(duel, LOCATION_EXTRA) || c.is_set_card(duel, &[SET_MAYAKASHI])
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_MAYAKASHI]) && c.is_summon_location(duel, LOCATION_EXTRA) && c.is_controler(duel, tp)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| cfilter(c, duel, a.tp), 1, None))
}

pub fn spcost<const CODE: u32>(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_custom_activity_count(CODE, tp, ACTIVITY_SPSUMMON) == 0);
    }
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| {
        c.is_location(duel, LOCATION_EXTRA) && !c.is_set_card(duel, &[SET_MAYAKASHI])
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(CODE, 1)));
    Ok(false)
}

pub fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.special_summon(c, 0, a.tp, a.tp, true, false, POS_FACEUP)?;
    }
    Ok(())
}

pub fn lizfilter(duel: &Duel, _: EffectId, c: CardId) -> bool {
    !c.is_original_set_card(duel, &[SET_MAYAKASHI])
}
