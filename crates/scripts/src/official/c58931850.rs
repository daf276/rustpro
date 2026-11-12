// Dragon Master Lords
use engine::constants::MSG_DAMAGE;

use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 58931850;

// Copies in the Extra Deck, Xyz Summoned with two Level 12 monsters (Dragon Master Magia), and 25 cards in the opponent's Graveyard
// for the Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (12381100, MZONE),
    (12381100, MZONE),
    (12381100, MZONE),
    (12381100, MZONE),
    (756652, GRAVE | OPPONENT),
    (756652, GRAVE | OPPONENT),
    (756652, GRAVE | OPPONENT),
    (756652, GRAVE | OPPONENT),
    (756652, GRAVE | OPPONENT),
    (756652, GRAVE | OPPONENT),
    (756652, GRAVE | OPPONENT),
    (756652, GRAVE | OPPONENT),
    (756652, GRAVE | OPPONENT),
    (756652, GRAVE | OPPONENT),
    (756652, GRAVE | OPPONENT),
    (756652, GRAVE | OPPONENT),
    (756652, GRAVE | OPPONENT),
    (756652, GRAVE | OPPONENT),
    (756652, GRAVE | OPPONENT),
    (756652, GRAVE | OPPONENT),
    (756652, GRAVE | OPPONENT),
    (756652, GRAVE | OPPONENT),
    (756652, GRAVE | OPPONENT),
    (756652, GRAVE | OPPONENT),
    (756652, GRAVE | OPPONENT),
    (756652, GRAVE | OPPONENT),
    (756652, GRAVE | OPPONENT),
    (756652, GRAVE | OPPONENT),
    (756652, GRAVE | OPPONENT),
];
// The piercing needs 25 cards in the player's own Graveyard and an attack on a Defense Position monster.
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::Message(MSG_DAMAGE)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(12);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_xyz_summoned(duel));
    e1.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e2.set_value(duel, Value::EffectPlayer(aux::indoval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_PIERCE);
    e3.set_condition(duel, |duel, a| {
        duel.get_field_group_count(a.e.get_handler_player(duel), LOCATION_GRAVE, 0) >= 25
    });
    e3.set_value(duel, DOUBLE_DAMAGE as i32);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 0));
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, 0);
    e4.set_condition(duel, |duel, a| duel.get_field_group_count(a.tp, 0, LOCATION_GRAVE) >= 25);
    e4.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e4.set_target(duel, sptg);
    e4.set_operation(duel, spop);
    c.register_effect(duel, e4);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    if !c.is_can_be_special_summoned(duel, e, 0, tp, false, false) {
        return false;
    }
    if c.is_location(duel, LOCATION_EXTRA) {
        duel.get_location_count_from_ex(tp, tp, None, c) > 0
    } else {
        duel.get_location_count(tp, LOCATION_MZONE) > 0
    }
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let location = LOCATION_DECK | LOCATION_EXTRA | LOCATION_GRAVE;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, location, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, location as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK | LOCATION_EXTRA | LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
