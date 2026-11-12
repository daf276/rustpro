// Gouki Machine Suprex
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, OPPONENT};

const ID: u32 = 29099860;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the hand with Gouki Guts to Special Summon alongside it, one in the Deck to search, and the opponent's Dark Hole to destroy them.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (7540107, HAND),
    (7540107, HAND),
    (7540107, DECK),
    (53129443, HAND | OPPONENT),
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
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_ONFIELD));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn spcostfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_GOUKI]) && !c.is_public(duel) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(!c.is_public(duel)
            && duel.is_existing_matching_card(|sc, duel| spcostfilter(sc, duel, e, tp), tp, LOCATION_HAND, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let g = duel.select_matching_card(
        tp,
        |sc, duel| spcostfilter(sc, duel, e, tp),
        tp,
        LOCATION_HAND,
        0,
        1,
        1,
        Some(c.into()),
    )?;
    let rc = g.get_first();
    unported!(duel, rc.is_none());
    let Some(rc) = rc else { return Ok(false) };
    e.set_label_object_group(duel, Some(rc.into()));
    rc.create_effect_relation(duel, e);
    duel.confirm_cards(1 - tp, CardSet::from_iter([c, rc]))?;
    duel.shuffle_hand(tp)?;
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.get_location_count(tp, LOCATION_MZONE) >= 2
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(summoned(duel, e)), 2, tp, 0);
    Ok(false)
}

fn summoned(duel: &Duel, e: EffectId) -> CardSet {
    let rc = e.get_label_object_group(duel).and_then(|g| g.get_first());
    [Some(e.get_handler(duel)), rc].into_iter().flatten().collect()
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_relate_to_effect(duel, e) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) < 2 || duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        return Ok(());
    }
    let sg = summoned(duel, e);
    if sg.filter_count(duel, |sc, duel| spfilter(sc, duel, e, tp), None) == 2
        && duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)? > 0
    {
        for sc in sg {
            sc.register_flag_effect_full(
                duel,
                ID,
                RESET_EVENT | RESETS_STANDARD,
                EFFECT_FLAG_CLIENT_HINT,
                1,
                0,
                aux::stringid(ID, 2),
            );
            let e1 = duel.create_effect(e.get_handler(duel));
            e1.set_type(duel, EFFECT_TYPE_FIELD);
            e1.set_property(
                duel,
                EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE,
            );
            e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
            e1.set_range(duel, LOCATION_MZONE);
            e1.set_absolute_range(duel, tp, 1, 0);
            e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_controler(duel, a.e.get_owner_player(duel)));
            e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| !c.is_set_card(duel, &[SET_GOUKI]));
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            sc.register_effect_full(duel, e1, true);
        }
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GOUKI]) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}
