// D - Force
use crate::prelude::*;
use crate::testing::{DECK, GRAVE, HAND, MZONE};

const ID: u32 = 6186304;
const DESTINY_HERO_PLASMA: u32 = 83965310;

pub const LISTED_NAMES: &[u32] = &[DESTINY_HERO_PLASMA];
// In the hand, with Destiny HERO - Plasma in the Deck, the Graveyard and on the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (DESTINY_HERO_PLASMA, DECK),
    (DESTINY_HERO_PLASMA, GRAVE),
    (DESTINY_HERO_PLASMA, MZONE),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    e0.set_target(duel, target);
    e0.set_operation(duel, activate);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_DRAW);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_target_range(duel, 1, 0);
    e1.set_condition(duel, |duel, a| plasmacon(duel, a) && drawcon(duel, a));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EFFECT_DRAW_COUNT);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target_range(duel, 1, 0);
    e2.set_condition(duel, plasmacon);
    e2.set_value(duel, 0);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_target_range(duel, LOCATION_ONFIELD, 0);
    e3.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE + EFFECT_FLAG_SET_AVAILABLE);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e3.set_condition(duel, plasmacon);
    e3.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD);
    e4.set_code(duel, EFFECT_UPDATE_ATTACK);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_target_range(duel, LOCATION_MZONE, 0);
    e4.set_condition(duel, plasmacon);
    e4.set_target_card(duel, |duel, _, c| c.is_code(duel, DESTINY_HERO_PLASMA));
    e4.set_value(duel, Value::Card(val));
    c.register_effect(duel, e4);
    let e5 = e4.clone_effect(duel);
    e5.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e5.set_value(duel, Value::EffectPlayer(aux::indoval));
    c.register_effect(duel, e5);
    let e6 = e4.clone_effect(duel);
    e6.set_code(duel, EFFECT_EXTRA_ATTACK);
    e6.set_value(duel, 1);
    c.register_effect(duel, e6);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, DESTINY_HERO_PLASMA) && c.is_able_to_hand(duel)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !a.e.get_handler(duel).is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    let g = duel.get_matching_group(aux::necro_valley_filter(thfilter), tp, LOCATION_DECK | LOCATION_GRAVE, 0, None);
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 0))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let sg = g.select(duel, tp, 1, 1, None)?;
        duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, sg)?;
    }
    Ok(())
}

fn plasmacon(duel: &Duel, a: &Args) -> bool {
    let filter = |c: CardId, duel: &Duel| c.is_faceup(duel) && c.is_code(duel, DESTINY_HERO_PLASMA);
    duel.is_existing_matching_card(filter, a.e.get_handler_player(duel), LOCATION_ONFIELD, 0, 1, None)
}

fn drawcon(duel: &Duel, _: &Args) -> bool {
    duel.is_phase(PHASE_DRAW)
}

fn val(duel: &Duel, _: EffectId, _: CardId) -> i32 {
    duel.get_matching_group_count(CardId::is_monster, 0, LOCATION_GRAVE, LOCATION_GRAVE, None) as i32 * 100
}
