// Black Rose Dragon/Assault Mode
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 46985799;

pub const LISTED_NAMES: &[u32] = &[CARD_ASSAULT_MODE, CARD_BLACK_ROSE_DRAGON];
pub const ASSAULT_MODE: u32 = CARD_BLACK_ROSE_DRAGON;

// Black Rose Dragon is Synchro Summoned (Ash Blossom & Joyous Spring, Mystical Elf) and tributed to Assault Mode Activate for a copy
// from the Deck, which Raigeki from the opponent destroys; a copy in the hand and one on the field for the other effects.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (ID, DECK),
    (CARD_BLACK_ROSE_DRAGON, EXTRA),
    (CARD_BLACK_ROSE_DRAGON, EXTRA),
    (14558127, MZONE),
    (15025844, MZONE),
    (CARD_ASSAULT_MODE, SZONE),
    (12580477, HAND | OPPONENT),
    (13193642, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    c.add_must_be_special_summoned(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_DESTROYED);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TODECK + CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, (ID, 0));
    e2.set_cost(duel, Cost::self_reveal);
    e2.set_target(duel, tdatktg);
    e2.set_operation(duel, tdatkop);
    c.register_effect(duel, e2);
    let e3a = duel.create_effect(c);
    e3a.set_description(duel, aux::stringid(ID, 2));
    e3a.set_category(duel, CATEGORY_DESTROY);
    e3a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3a.set_property(duel, EFFECT_FLAG_DELAY);
    e3a.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3a.set_count_limit(duel, 1, (ID, 1));
    e3a.set_target(duel, destg);
    e3a.set_operation(duel, desop);
    c.register_effect(duel, e3a);
    let e3b = e3a.clone_effect(duel);
    e3b.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3b.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e3b.set_range(duel, LOCATION_MZONE);
    e3b.set_condition(duel, |duel, a| {
        let (c, tp) = (a.e.get_handler(duel), a.tp);
        let eg = a.eg.unwrap_or_default();
        !eg.is_contains(c) && eg.is_exists(duel, |c, duel| c.is_controler(duel, 1 - tp), 1, None)
    });
    c.register_effect(duel, e3b);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, CARD_BLACK_ROSE_DRAGON) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_GRAVE)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_GRAVE, POS_FACEUP)
}

fn tdatktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_deck(duel)
            && duel.is_existing_matching_card(CardId::has_non_zero_attack, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TODECK, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn tdatkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e)
        && duel.send_to_deck(c, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? > 0
        && c.is_location(duel, LOCATION_DECK)
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATKDEF);
        let g = duel.select_matching_card(tp, CardId::has_non_zero_attack, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
        let Some(sc) = g.first() else { return Ok(()) };
        duel.hint_selection(sc);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_value(duel, 0);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        sc.register_effect(duel, e1);
    }
    Ok(())
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_field_group(tp, LOCATION_ONFIELD, LOCATION_ONFIELD);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_field_group(a.tp, LOCATION_ONFIELD, LOCATION_ONFIELD);
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
