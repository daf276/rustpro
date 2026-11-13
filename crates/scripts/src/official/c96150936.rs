// Madolche Fresh Sistart
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};
use engine::constants::MSG_SELECT_EFFECTYN;

const ID: u32 = 96150936;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_MADOLCHE], Some(lc), sumtype, tp)
    }
}

// Madolche Chouxvalier and Puddingcess to Link Summon and to send back, and the opponent's Raigeki and Dark Hole to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (75363626, MZONE),
    (75363626, MZONE),
    (75363626, MZONE),
    (74641045, GRAVE),
    (74641045, GRAVE),
    (12580477, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::Message(MSG_SELECT_EFFECTYN)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EFFECT_DESTROY_REPLACE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target(duel, desreptg);
    e1.set_operation(duel, desrepop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_ONFIELD, 0);
    e2.set_condition(duel, indescon);
    e2.set_target_card(duel, indestg);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e3.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    c.register_effect(duel, e3);
}

fn desrepfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MADOLCHE]) && c.is_monster(duel) && c.is_able_to_deck(duel) && aux::nvfilter(c, duel)
}

fn desreptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(!c.is_reason(duel, REASON_RULE) && duel.is_existing_matching_card(desrepfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.select_effect_yes_no_full(tp, c, 96)
}

fn desrepop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_matching_card(tp, desrepfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT | REASON_REPLACE)?;
    Ok(())
}

fn linkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_MADOLCHE]) && c.is_monster(duel)
}

fn indescon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_linked_group(duel).is_exists(duel, linkfilter, 1, None)
}

fn indestg(duel: &Duel, _: EffectId, c: CardId) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_MADOLCHE]) && c.is_spell_trap(duel)
}
