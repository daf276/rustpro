// Eternal Soul
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, SZONE};

const ID: u32 = 48680970;
const THOUSAND_KNIVES: u32 = 63391643;
const DARK_MAGIC_ATTACK: u32 = 2314238;

pub const LISTED_NAMES: &[u32] = &[CARD_DARK_MAGICIAN, THOUSAND_KNIVES, DARK_MAGIC_ATTACK];

// Set, with Dark Magician on the field, in the hand and in the Graveyard, and its two Spells to add from the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, HAND),
    (CARD_DARK_MAGICIAN, MZONE),
    (CARD_DARK_MAGICIAN, HAND),
    (CARD_DARK_MAGICIAN, GRAVE),
    (THOUSAND_KNIVES, DECK),
    (DARK_MAGIC_ATTACK, DECK),
];

pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_END_PHASE);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_target_card(duel, |duel, _, c| c.is_code(duel, CARD_DARK_MAGICIAN));
    e2.set_value(
        duel,
        Value::Effect(|duel, e, re| re.get_owner_player(duel) != e.get_handler_player(duel)),
    );
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_LEAVE_FIELD);
    e3.set_condition(duel, descon);
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_type(duel, EFFECT_TYPE_QUICK_O);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_code(duel, EVENT_FREE_CHAIN);
    e4.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e4.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e4.set_count_limit(duel, 1, ID);
    e4.set_target(duel, thtg);
    e4.set_operation(duel, thop);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 1));
    e5.set_type(duel, EFFECT_TYPE_QUICK_O);
    e5.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e5.set_range(duel, LOCATION_SZONE);
    e5.set_code(duel, EVENT_FREE_CHAIN);
    e5.set_hint_timing(duel, 0, TIMING_END_PHASE);
    e5.set_count_limit(duel, 1, ID);
    e5.set_target(duel, sptg);
    e5.set_operation(duel, spop);
    c.register_effect(duel, e5);
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_position(duel, POS_FACEUP) && !c.is_location(duel, LOCATION_DECK)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let g = duel.get_matching_group(aux::TRUE, a.tp, LOCATION_MZONE, 0, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(aux::TRUE, a.tp, LOCATION_MZONE, 0, None);
    duel.destroy(g, REASON_EFFECT)?;
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_code(duel, DARK_MAGIC_ATTACK) || c.is_code(duel, THOUSAND_KNIVES)) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.hint(HINT_OPSELECTED, 1 - tp, a.e.get_description(duel));
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !a.e.get_handler(duel).is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, CARD_DARK_MAGICIAN) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let filter = |c, duel: &Duel| spfilter(c, duel, e, tp);
        return Ok(
            duel.is_existing_matching_card(filter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, None)
                && duel.get_location_count(tp, LOCATION_MZONE) > 0,
        );
    }
    duel.hint(HINT_OPSELECTED, 1 - tp, e.get_description(duel));
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
