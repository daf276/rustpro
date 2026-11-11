// The Phantom Knights of Ragged Gloves
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, MZONE};

const ID: u32 = 63821877;

// Two copies as Xyz material for Break Sword and Malevolent Scythe, and Doomed Soleret in the Deck to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, GRAVE),
    (62709239, EXTRA),
    (78449284, EXTRA),
    (83566725, DECK),
];
// The ATK trigger chains as the Xyz Monster, so no check can count it for this card; MaterialFrom stands in.
pub const CHECKS: &[Check] = &[Check::MaterialFrom(MZONE), Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_property(duel, EFFECT_FLAG_EVENT_PLAYER);
    e1.set_code(duel, EVENT_BE_MATERIAL);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| {
        a.r == REASON_XYZ
            && a.e
                .get_handler(duel)
                .get_reason_card(duel)
                .is_some_and(|rc| rc.is_attribute(duel, ATTRIBUTE_DARK))
    });
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(rc) = c.get_reason_card(duel) else { return Ok(()) };
    let e1 = duel.create_effect(rc);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_xyz_summoned(duel));
    e1.set_operation(duel, atkop);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    rc.register_effect_full(duel, e1, true);
    if !rc.is_type(duel, TYPE_EFFECT) {
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_ADD_TYPE);
        e2.set_value(duel, TYPE_EFFECT as i32);
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        rc.register_effect_full(duel, e2, true);
    }
    Ok(())
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && c.is_faceup(duel) {
        c.update_attack(duel, 1000, None, None, None);
    }
    Ok(())
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_set_card(duel, &[SET_THE_PHANTOM_KNIGHTS]) || (c.is_set_card(duel, &[SET_PHANTOM_KNIGHTS]) && c.is_spell_trap(duel)))
        && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, tgfilter, LOCATION_DECK, CATEGORY_TOGRAVE)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::send_to_grave(duel, a, tgfilter, LOCATION_DECK)
}
