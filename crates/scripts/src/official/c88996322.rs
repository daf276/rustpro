// Gladiator Beast Vespasius
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 88996322;

pub const LISTED_SERIES: &[u16] = &[SET_GLADIATOR];
// In the hand while Gladiator Beast Hoplomus battles, then on the field after battle with Hoplomus in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (4253484, MZONE),
    (4253484, MZONE),
    (4253484, MZONE),
    (4253484, DECK),
    (4253484, DECK),
    (27125110, MZONE | OPPONENT),
    (27125110, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BATTLE_START);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, stock::special_summon_self_target_player_0);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).get_flag_effect(duel, ID) > 0);
    e2.set_value(duel, 500);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_PHASE | PHASE_BATTLE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).get_battled_group_count(duel) > 0);
    e3.set_cost(duel, Cost::self_to_deck);
    e3.set_target(duel, |duel, a, chk, _| {
        super::c57731460::sptg(duel, a, chk, |c, duel| filter(c, duel, a.e, a.tp))
    });
    e3.set_operation(duel, |duel, a| {
        super::c57731460::spop(duel, a, |c, duel| filter(c, duel, a.e, a.tp), 100, false, 0)
    });
    c.register_effect(duel, e3);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    let Some(mut at) = duel.get_attacker() else { return false };
    let Some(mut d) = at.get_battle_target(duel) else { return false };
    if at.is_controler(duel, 1 - tp) {
        (at, d) = (d, at);
    }
    at.is_controler(duel, tp)
        && at.is_faceup(duel)
        && at.is_set_card(duel, &[SET_GLADIATOR])
        && at.get_controler(duel) != d.get_controler(duel)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && duel.special_summon(c, 0, a.tp, a.tp, false, false, POS_FACEUP)? != 0 {
        let code = c.get_original_code(duel);
        c.register_flag_effect(duel, code, RESET_EVENT | RESETS_STANDARD_DISABLE, 0, 0);
    }
    Ok(())
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    !c.is_code(duel, ID) && c.is_set_card(duel, &[SET_GLADIATOR]) && c.is_can_be_special_summoned(duel, e, 100, tp, false, false)
}
