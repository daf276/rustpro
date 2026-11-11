// X-Saber Palomuro
use crate::prelude::*;
use crate::testing::{GRAVE, MZONE, OPPONENT};

const ID: u32 = 96099959;

// X-Saber Anu Piranha on the field and strong monsters on the opponent's side to destroy them in battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (23115241, MZONE),
    (23115241, MZONE),
    (23115241, MZONE),
    (39674352, MZONE | OPPONENT),
    (32626733, MZONE | OPPONENT),
    (56649609, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_range(duel, LOCATION_GRAVE);
    e1.set_code(duel, EVENT_BATTLE_DESTROYED);
    e1.set_condition(duel, condition);
    e1.set_cost(duel, Cost::pay_lp::<500>);
    e1.set_target(duel, stock::special_summon_self_target_player_0);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel, ec: CardId, tp: u8) -> bool {
    c != ec
        && c.is_set_card(duel, &[SET_SABER])
        && c.is_location(duel, LOCATION_GRAVE)
        && c.is_previous_controler(duel, tp)
        && c.is_reason(duel, REASON_BATTLE)
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    a.eg.unwrap_or_default()
        .is_exists(duel, |tc, duel| filter(tc, duel, c, tp), 1, None)
}
