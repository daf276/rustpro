// Gold Pride - Captain Carrie
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 96305350;

// In the hand and on the field, with Chariot Carrie in the Extra Deck (Xyz Summoned with Level 3 Chamberlain of the Six Samurai) to boost,
// Gold Pride - Leon in the Graveyard to banish, and the opponent's Raigeki to send it to the Graveyard. No Gold Pride Trap is ported, so
// the search is not reached.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (65892585, EXTRA),
    (65892585, EXTRA),
    (44430454, MZONE),
    (44430454, MZONE),
    (23512906, GRAVE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| duel.get_lp(a.tp) < duel.get_lp(1 - a.tp));
    e1.set_target(duel, stock::special_summon_self_target);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_REMOVE + CATEGORY_ATKCHANGE);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e4.set_code(duel, EVENT_TO_GRAVE);
    e4.set_count_limit(duel, 1, (ID, 2));
    e4.set_target(duel, atktg);
    e4.set_operation(duel, atkop);
    c.register_effect(duel, e4);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GOLD_PRIDE]) && c.is_trap(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn rmvfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GOLD_PRIDE]) && c.is_able_to_remove(duel) && aux::sp_elim_filter(c, duel, true, false)
}

fn atkfilter(c: CardId, duel: &Duel, g: CardSet) -> bool {
    c.is_set_card(duel, &[SET_GOLD_PRIDE])
        && c.is_summon_location(duel, LOCATION_EXTRA)
        && c.is_faceup(duel)
        && (!g.is_contains(c) || g.len() > 1)
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(rmvfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, None);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && atkfilter(chkc, duel, g));
    }
    if chk == 0 {
        return Ok(!g.is_empty() && duel.is_existing_target(|c, duel| atkfilter(c, duel, g), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATKDEF);
    let tc = duel.select_target(tp, |c, duel| atkfilter(c, duel, g), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, tc, 1, 0, 500);
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tc = duel.get_first_target();
    let g = duel.get_matching_group(rmvfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, tc.map(CardSet::from));
    if !g.is_empty() {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
        let rg = g.select(duel, tp, 1, 3, None)?;
        let ct = duel.remove(rg, POS_FACEUP, REASON_EFFECT)?;
        if ct > 0
            && let Some(tc) = tc
            && tc.is_relate_to_effect(duel, e)
            && tc.is_faceup(duel)
        {
            tc.update_attack(duel, ct * 500, Some(RESET_EVENT | RESETS_STANDARD), Some(e.get_handler(duel)), None);
        }
    }
    Ok(())
}
