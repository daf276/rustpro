// Exosister Sophia
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 5352328;

pub const LISTED_NAMES: &[u32] = &[79858629];
pub const LISTED_SERIES: &[u16] = &[SET_EXOSISTER];

// On the field with Exosister Irene, and the opponent's Monster Reborn to take a monster out of a Graveyard, for Exosister Karmael
// in the Extra Deck to be Xyz Summoned.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, HAND),
    (79858629, MZONE),
    (79858629, MZONE),
    (77675029, EXTRA),
    (77675029, EXTRA),
    (83764718, HAND | OPPONENT),
    (83764718, HAND | OPPONENT),
    (15023985, GRAVE),
    (15023985, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DRAW + CATEGORY_RECOVER);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, drcon);
    e1.set_target(duel, stock::draw_target::<1>);
    e1.set_operation(duel, drop::<79858629>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_LEAVE_GRAVE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |_, a| a.rp == 1 - a.tp);
    e2.set_target(duel, super::c16474916::sptg);
    e2.set_operation(duel, super::c16474916::spop);
    c.register_effect(duel, e2);
}

fn drcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(
        aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_EXOSISTER])),
        a.tp,
        LOCATION_MZONE,
        0,
        1,
        Some(a.e.get_handler(duel).into()),
    )
}

pub fn drop<const PARTNER: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    if duel.draw(p, d, REASON_EFFECT)? == d as i32
        && duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_code(duel, PARTNER)),
            tp,
            LOCATION_ONFIELD,
            0,
            1,
            None,
        )
    {
        duel.break_effect()?;
        duel.recover(tp, 800, REASON_EFFECT)?;
    }
    Ok(())
}
