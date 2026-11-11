// Cyber Phoenix
use engine::constants::MSG_CHAIN_DISABLED;

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 3370104;

// Two on the field with Axe of Despair to equip them and Mystical Space Typhoon to chain to it, and strong monsters on the
// opponent's side to destroy them in battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (40619825, HAND),
    (40619825, HAND),
    (5318639, HAND | OPPONENT),
    (39674352, MZONE | OPPONENT),
    (32626733, MZONE | OPPONENT),
    (56649609, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Message(MSG_CHAIN_DISABLED), Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_DISABLE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_SZONE, LOCATION_SZONE);
    e1.set_condition(duel, discon);
    e1.set_target_card(duel, distg);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_CHAIN_SOLVING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_DRAW);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_code(duel, EVENT_BATTLE_DESTROYED);
    e3.set_condition(duel, condition);
    e3.set_target(duel, stock::draw_target::<1>);
    e3.set_operation(duel, stock::draw);
    c.register_effect(duel, e3);
    aux::double_snare_validity(duel, c, LOCATION_MZONE);
}

fn discon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_attack_pos(duel)
}

fn distg(duel: &Duel, e: EffectId, c: CardId) -> bool {
    c.get_card_target_count(duel) == 1
        && c.get_first_card_target(duel)
            .is_some_and(|tc| tc.is_controler(duel, e.get_handler_player(duel)) && tc.is_faceup(duel) && tc.is_race(duel, RACE_MACHINE))
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, ev) = (a.tp, a.ev as u8);
    let Some(re) = a.re else { return Ok(()) };
    if !a.e.get_handler(duel).is_attack_pos(duel) || re.is_monster_effect(duel) || !re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET) {
        return Ok(());
    }
    let Some(tc) = duel
        .get_chain_info::<Option<CardSet>>(ev, CHAININFO_TARGET_CARDS)
        .filter(|g| g.len() == 1)
        .and_then(|g| g.get_first())
    else {
        return Ok(());
    };
    if tc.is_controler(duel, tp) && tc.is_location(duel, LOCATION_MZONE) && tc.is_faceup(duel) && tc.is_race(duel, RACE_MACHINE) {
        duel.negate_effect(ev);
    }
    Ok(())
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_location(duel, LOCATION_GRAVE) && c.is_reason(duel, REASON_BATTLE) && c.is_previous_position(duel, POS_FACEUP)
}
