// Deep-Eyes White Dragon, the Blue Abyss
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 67886895;
const GYGROUP: u64 = (ID as u64) << 16;

pub const LISTED_NAMES: &[u32] = &[CARD_BLUEEYES_W_DRAGON];

// In the hand and Graveyard, with a Level 1 LIGHT Tuner in the Deck to search and Blue-Eyes White Dragons on the field and the
// opponent's Dark Hole to send them to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (47643326, DECK),
    (47643326, DECK),
    (CARD_BLUEEYES_W_DRAGON, MZONE),
    (CARD_BLUEEYES_W_DRAGON, MZONE),
    (CARD_BLUEEYES_W_DRAGON, HAND),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_cost(duel, Cost::self_discard);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_ATKCHANGE + CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_CUSTOM + ID);
    e2.set_range(duel, LOCATION_GRAVE | LOCATION_HAND);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    aux::global_check(duel, ID, |duel| {
        duel.set_script_group(GYGROUP, Some(CardSet::default()));
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_TO_GRAVE);
        ge1.set_operation(duel, regop);
        duel.register_effect(ge1, 0);
    });
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level(duel, &[1]) && c.is_attribute(duel, ATTRIBUTE_LIGHT) && c.is_type(duel, TYPE_TUNER) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn tgfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    ((c.is_set_card(duel, &[SET_BLUE_EYES]) && c.is_ritual_monster(duel)) || c.is_code(duel, CARD_BLUEEYES_W_DRAGON))
        && c.is_can_be_effect_target(duel, e)
        && c.is_location(duel, LOCATION_GRAVE)
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let tg = a.eg.unwrap_or_default().filter(duel, |c, duel| tgfilter(c, duel, e), None);
    if tg.is_empty() {
        return Ok(());
    }
    for tc in tg {
        tc.register_flag_effect(duel, ID, RESET_CHAIN, 0, 1);
    }
    let mut gygroup = duel.get_script_group(GYGROUP).unwrap_or_default();
    if duel.get_current_chain() == 0 {
        gygroup = CardSet::default();
    }
    gygroup.merge(tg);
    gygroup.remove_matching(duel, |c, duel| c.get_flag_effect(duel, ID) == 0, None);
    duel.set_script_group(GYGROUP, Some(gygroup));
    duel.raise_event(gygroup, EVENT_CUSTOM + ID, Some(e), 0, a.rp, a.ep, a.ev)
}

fn gygroup_targets(duel: &Duel, e: EffectId, tp: u8) -> CardSet {
    let gygroup = duel.get_script_group(GYGROUP).unwrap_or_default();
    gygroup
        .filter(duel, |c, duel| tgfilter(c, duel, e), None)
        .filter(duel, |c, duel| c.is_controler(duel, tp), None)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = gygroup_targets(duel, e, tp);
    if let Some(chkc) = chkc {
        return Ok(g.is_contains(chkc) && chkc.is_controler(duel, tp) && tgfilter(chkc, duel, e));
    }
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(!g.is_empty()
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    let tc = if g.len() > 1 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
        g.select(duel, tp, 1, 1, None)?.get_first()
    } else {
        g.get_first()
    };
    if let Some(tc) = tc {
        duel.set_target_card(tc);
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !(c.is_relate_to_effect(duel, e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0) {
        return Ok(());
    }
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        let atk = tc.get_attack(duel);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_value(duel, atk);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
    }
    let e2 = duel.create_effect(c);
    e2.set_description(duel, 3300);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
    e2.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
    e2.set_value(duel, LOCATION_REMOVED as i32);
    e2.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
    c.register_effect_full(duel, e2, true);
    Ok(())
}
