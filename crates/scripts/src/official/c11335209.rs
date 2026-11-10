// Duamutef, Blessing of Horus
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 11335209;

pub const LISTED_NAMES: &[u32] = &[CARD_KING_SARCOPHAGUS];

// In the Graveyard with King's Sarcophagus on the field to return, a copy and Imsety, Glory of Horus, on the field, and
// Raigeki and Dark Hole in the opponent's hand to destroy a card of ours and draw.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (ID, MZONE),
    (84941194, MZONE),
    (CARD_KING_SARCOPHAGUS, SZONE),
    (12580477, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
    (7892180, DECK),
    (7892180, DECK),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, spcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(
        duel,
        Value::Card(|duel, _, c| {
            let horus = aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_HORUS]));
            duel.get_matching_group_count(horus, c.get_controler(duel), LOCATION_MZONE, 0, None) as i32 * 1200
        }),
    );
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_DRAW);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e4.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e4.set_code(duel, EVENT_LEAVE_FIELD);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, (ID, 1));
    e4.set_condition(duel, drwcon);
    e4.set_target(duel, drwtg);
    e4.set_operation(duel, drwop);
    c.register_effect(duel, e4);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    unported!(duel, !c.get_card_effect(duel, EFFECT_NECRO_VALLEY).is_empty());
    let tp = c.get_controler(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_code(duel, CARD_KING_SARCOPHAGUS)),
            tp,
            LOCATION_ONFIELD,
            0,
            1,
            None,
        )
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_previous_location(duel, LOCATION_ONFIELD)
        && c.is_previous_controler(duel, tp)
        && c.is_reason(duel, REASON_EFFECT)
        && c.get_reason_player(duel) == 1 - tp
}

fn drwcon(duel: &Duel, a: &Args) -> bool {
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| cfilter(c, duel, a.tp), 1, None))
}

fn class_count(duel: &Duel, tp: u8) -> usize {
    duel.get_matching_group(CardId::is_faceup, tp, LOCATION_MMZONE, 0, None)
        .get_class_count(duel, |c, duel| i64::from(c.get_code(duel)))
}

fn drwtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let ct = class_count(duel, tp);
    if chk == 0 {
        return Ok(ct > 0 && duel.is_player_can_draw(tp, ct));
    }
    duel.set_target_player(tp);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, ct as i32);
    Ok(false)
}

fn drwop(duel: &mut Duel, a: &Args) -> Script<()> {
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let ct = class_count(duel, a.tp);
    if ct > 0 {
        duel.draw(p, ct as u32, REASON_EFFECT)?;
    }
    Ok(())
}
