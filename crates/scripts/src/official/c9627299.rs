// Ichiki Sayori-Hime
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 9627299;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand with a 800 ATK monster (Mystical Elf) on the field to Normal Summon from the hand, and Gradius in the Deck to
// search.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (15025844, MZONE), (10992251, DECK), (10992251, DECK)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.add_must_be_special_summoned(duel);
    Spirit::add_procedure(duel, c, &[EVENT_SUMMON_SUCCESS, EVENT_FLIP]);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, nscon);
    e1.set_target(duel, nstg);
    e1.set_operation(duel, nsop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn nsconfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_attack(duel, &[800]) || c.is_defense(duel, &[800])) && c.is_faceup(duel)
}

fn nscon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(nsconfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn nstg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_summonable(duel, true, None));
    }
    duel.set_operation_info(0, CATEGORY_SUMMON, Some(c.into()), 1, a.tp, 0);
    Ok(false)
}

fn nsop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.summon(a.tp, c, true, None)?;
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level(duel, &[4])
        && c.is_attribute(duel, ATTRIBUTE_LIGHT | ATTRIBUTE_DARK)
        && (c.is_attack(duel, &[800]) || c.is_defense(duel, &[800]))
        && c.is_able_to_hand(duel)
        && !c.is_code(duel, ID)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, a.tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
