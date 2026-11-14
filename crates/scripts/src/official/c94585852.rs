// Pandemonium
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 94585852;

pub const LISTED_SERIES: &[u16] = &[SET_ARCHFIEND];
// In the hand, with Summoned Skull on both sides for Raigeki to destroy and Archfiend Soldier and Marmot of Nefariousness (lower
// Levels) in the Deck. The LP cost replacement needs an "Archfiend" monster paying LP in the Standby Phase, which none ported does.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (70781052, MZONE),
    (70781052, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
    (49881766, DECK),
    (75889523, DECK),
    (75889523, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_code(duel, EFFECT_LPCOST_REPLACE);
    e2.set_condition(duel, lrcon);
    c.register_effect(duel, e2);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e4.set_range(duel, LOCATION_FZONE);
    e4.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_EVENT_PLAYER);
    e4.set_code(duel, EVENT_CUSTOM + ID);
    e4.set_target(duel, target);
    e4.set_operation(duel, operation);
    c.register_effect(duel, e4);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_TO_GRAVE);
        ge1.set_operation(duel, regop);
        duel.register_effect(ge1, 0);
    });
}

fn regfilter(c: CardId, duel: &Duel) -> bool {
    c.is_reason(duel, REASON_DESTROY)
        && !c.is_reason(duel, REASON_BATTLE)
        && c.is_set_card(duel, &[SET_ARCHFIEND])
        && c.has_level(duel)
        && (c.is_previous_location(duel, LOCATION_MZONE) || c.is_monster(duel))
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (mut lv1, mut lv2) = (0, 0);
    for tc in a.eg.unwrap_or_default() {
        if regfilter(tc, duel) {
            let level = if tc.is_controler(duel, 0) { &mut lv1 } else { &mut lv2 };
            *level = (*level).max(tc.get_level(duel));
        }
    }
    for (player, level) in [(0, lv1), (1, lv2)] {
        if level > 0 {
            duel.raise_event(CardSet::default(), EVENT_CUSTOM + ID, a.re, a.r, a.rp, player, level)?;
        }
    }
    Ok(())
}

fn lrcon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    let rc = re.get_handler(duel);
    duel.is_phase(PHASE_STANDBY) && rc.is_set_card(duel, &[SET_ARCHFIEND]) && rc.is_monster(duel)
}

fn filter(c: CardId, duel: &Duel, lv: u32) -> bool {
    c.get_level(duel) < lv && c.is_set_card(duel, &[SET_ARCHFIEND]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (tp, ev) = (a.tp, a.ev);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| filter(c, duel, ev), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, ev) = (a.tp, a.ev);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, |c, duel| filter(c, duel, ev), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
