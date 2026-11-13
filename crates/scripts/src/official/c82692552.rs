// Coralianektes the Shimmering Shining Dragon
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 82692552;
const REVEALED_CARDS: u8 = 0;

pub const LISTED_NAMES: &[u32] = &[CARD_UMI];

// On the field, in the hand and the Deck, with Compulsory Evacuation Device and Sangan with Dark Hole to add it to the hand, and
// Hyosube in the hand as the other WATER monster.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, HAND),
    (ID, DECK),
    (ID, DECK),
    (ID, DECK),
    (94192409, SZONE),
    (94192409, SZONE),
    (94192409, SZONE | OPPONENT),
    (94192409, SZONE | OPPONENT),
    (26202165, MZONE),
    (53129443, HAND),
    (2118022, HAND),
    (2118022, HAND),
    (CARD_UMI, HAND),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_TO_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| !a.e.get_handler(duel).is_reason(duel, REASON_DRAW));
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e2.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_condition(duel, |duel, a| {
        let tp = a.e.get_handler_player(duel);
        duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_code(duel, CARD_UMI)),
            tp,
            LOCATION_ONFIELD,
            0,
            1,
            None,
        ) || duel.is_environment_full(CARD_UMI, tp, LOCATION_FZONE | LOCATION_ONFIELD)
    });
    e2.set_target_card(duel, |duel, _, c| c.is_attribute(duel, ATTRIBUTE_WATER) && c.is_link_monster(duel));
    e2.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e2);
}

fn spcostfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_attribute(duel, ATTRIBUTE_WATER) && !c.is_public(duel) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(!c.is_public(duel)
            && duel.is_existing_matching_card(|c, duel| spcostfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let sc = duel.select_matching_card(
        tp,
        |c, duel| spcostfilter(c, duel, e, tp),
        tp,
        LOCATION_HAND,
        0,
        1,
        1,
        Some(c.into()),
    )?;
    let mut g = CardSet::from(c);
    g.merge(sc);
    duel.confirm_cards(1 - tp, g)?;
    duel.shuffle_hand(tp)?;
    if let Some(sc) = sc.get_first() {
        sc.create_effect_relation(duel, e);
    }
    e.set_chain_data_group(duel, REVEALED_CARDS, g);
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let c = e.get_handler(duel);
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) >= 2
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none());
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, e.get_chain_data_group(duel, REVEALED_CARDS), 2, tp, 0);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_relate_to_effect(duel, e) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = e.get_chain_data_group(duel, REVEALED_CARDS).unwrap_or_default();
    if duel.get_location_count(tp, LOCATION_MZONE) >= 2
        && g.filter_count(duel, |c, duel| spfilter(c, duel, e, tp), None) == 2
        && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
    {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_attribute_except(duel, ATTRIBUTE_WATER));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}
