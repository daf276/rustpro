// Azamina Aphes
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, SZONE};

const ID: u32 = 39491690;

pub const LISTED_SERIES: &[u16] = &[SET_WHITE_FOREST];

// An Azamina Fusion Monster in the GY to count, and Elzette and Astellar of the White Forest, whose costs send it from the hand or
// field, with Silvy of the White Forest in the Deck for their effects.
pub const SUPPORT: &[(u32, u16)] = &[
    (73391962, GRAVE),
    (61980241, HAND),
    (61980241, HAND),
    (25592142, MZONE),
    (98385955, DECK),
    (98385955, DECK),
    (ID, SZONE),
    (ID, SZONE),
    (ID, HAND),
    (ID, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_LEAVE_GRAVE + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        a.e.get_handler(duel).is_reason(duel, REASON_COST) && a.re.is_some_and(|re| re.is_activated(duel) && re.is_monster_effect(duel))
    });
    e2.set_target(duel, settg);
    e2.set_operation(duel, setop);
    c.register_effect(duel, e2);
}

fn thctfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_FUSION | TYPE_SYNCHRO) && c.is_race(duel, RACE_FIEND | RACE_ILLUSION | RACE_SPELLCASTER) && c.is_faceup(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_able_to_hand(duel) && chkc != c);
    }
    let ct = duel.get_matching_group_count(thctfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, None);
    if chk == 0 {
        return Ok(ct > 0 && duel.is_existing_target(CardId::is_able_to_hand, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(
        tp,
        CardId::is_able_to_hand,
        tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        1,
        ct as u8,
        Some(c.into()),
    )?;
    let count = g.map_or(0, |g| g.len());
    duel.set_operation_info(0, CATEGORY_TOHAND, g, count as u8, tp, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if !tg.is_empty() {
        duel.send_to_hand(tg, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_ssetable(duel));
    }
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(c.into()), 1, a.tp, 0);
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && c.is_ssetable(duel) {
        duel.sset(a.tp, c)?;
    }
    Ok(())
}
