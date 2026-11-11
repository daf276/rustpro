// DoomZ Change
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 38007649;

pub const LISTED_NAMES: &[u32] = &[ID];
// In the hand and Set, with DoomZ monsters on the field, in the Deck, in the hand and in the Graveyard, and the opponent's
// Mystical Space Typhoon to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (5318639, HAND | OPPONENT),
    (5318639, HAND | OPPONENT),
    (54265980, MZONE),
    (28877382, DECK),
    (28877382, DECK),
    (54265980, HAND),
    (92472273, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |_, a| a.r & REASON_EFFECT > 0);
    e2.set_target(duel, thsptg);
    e2.set_operation(duel, thspop);
    c.register_effect(duel, e2);
}

const DESTROY_FROM: u32 = LOCATION_HAND | LOCATION_DECK | LOCATION_ONFIELD;

fn desfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DOOMZ]) && (c.is_faceup(duel) || !c.is_on_field(duel)) && !c.is_code(duel, ID)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, desfilter, DESTROY_FROM, CATEGORY_DESTROY)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(tp, desfilter, tp, DESTROY_FROM, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first() {
        if sc.is_on_field(duel) {
            duel.hint_selection(sc);
        }
        duel.destroy(sc, REASON_EFFECT)?;
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DOOMZ]) && c.is_able_to_hand(duel) && !c.is_code(duel, ID)
}

fn thsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_GRAVE as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_DOOMZ]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn thspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let Some(sc) = duel
        .select_matching_card(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    duel.hint_selection(sc);
    if duel.send_to_hand(sc, PLAYER_NONE, REASON_EFFECT)? > 0
        && sc.is_location(duel, LOCATION_HAND)
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
    {
        duel.shuffle_hand(tp)?;
        let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, None);
        if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let sg = g.select(duel, tp, 1, 1, None)?;
            duel.break_effect()?;
            duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
