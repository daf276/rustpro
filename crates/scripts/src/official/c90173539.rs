// World Dino Wrestling
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 90173539;

// In the hand to activate and in the Graveyard to banish, with Dinowrestlers on the field and in the Deck, and more monsters of the
// opponent's than the player's.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (75366958, MZONE),
    (75366958, DECK),
    (75366958, DECK),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e2.set_code(duel, EFFECT_CANNOT_ATTACK);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e2.set_condition(duel, atkcon1);
    e2.set_target_card(duel, |duel, e, c| i64::from(c.get_field_id(duel)) != e.get_label(duel));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e3.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_operation(duel, checkop);
    e3.set_label_object(duel, Some(e2));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_code(duel, EFFECT_UPDATE_ATTACK);
    e4.set_range(duel, LOCATION_FZONE);
    e4.set_target_range(duel, LOCATION_MZONE, 0);
    e4.set_target_card(duel, atktg2);
    e4.set_condition(duel, |duel, _| {
        duel.is_phase(PHASE_DAMAGE_CAL) && duel.get_attack_target().is_some()
    });
    e4.set_value(duel, 200);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 1));
    e5.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e5.set_type(duel, EFFECT_TYPE_IGNITION);
    e5.set_range(duel, LOCATION_GRAVE);
    e5.set_count_limit(duel, 1, ID);
    e5.set_condition(duel, spcon);
    e5.set_cost(duel, Cost::self_banish);
    e5.set_target(duel, sptg);
    e5.set_operation(duel, spop);
    c.register_effect(duel, e5);
}

fn atkcon1(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_flag_effect(duel, ID) != 0
        && duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_DINOWRESTLER])),
            a.e.get_handler_player(duel),
            LOCATION_MZONE,
            0,
            1,
            None,
        )
}

fn checkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.get_flag_effect(duel, ID) != 0 {
        return Ok(());
    }
    let (Some(fid), Some(e2)) = (
        a.eg.and_then(|eg| eg.get_first()).map(|ac| ac.get_field_id(duel)),
        a.e.get_label_object(duel),
    ) else {
        return Ok(());
    };
    c.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
    e2.set_label(duel, fid.into());
    Ok(())
}

fn atktg2(duel: &Duel, e: EffectId, c: CardId) -> bool {
    c.is_set_card(duel, &[SET_DINOWRESTLER]) && duel.get_turn_player() == e.get_handler_player(duel) && duel.get_attacker() == Some(c)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    duel.get_field_group_count(tp, 0, LOCATION_MZONE) > duel.get_field_group_count(tp, LOCATION_MZONE, 0)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_DINOWRESTLER]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, e, tp), LOCATION_DECK)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, e, tp), LOCATION_DECK, POS_FACEUP)
}
