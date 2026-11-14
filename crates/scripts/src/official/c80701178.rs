// Impcantation Talismandra
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 80701178;
const CHALISLIME: u32 = 65877963;
const CANDOLL: u32 = 53303460;

pub const LISTED_SERIES: &[u16] = &[SET_IMPCANTATION];
// In the hand with Impcantation Chalislime (Ritual Monster) to reveal and search, and Impcantation Candoll in the Deck to
// summon; also in the Deck for Candoll's summon, with Impcantation Inception (Ritual Spell) in the hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (CHALISLIME, HAND),
    (CHALISLIME, DECK),
    (CANDOLL, DECK),
    (CANDOLL, HAND),
    (86758915, HAND),
    (ID, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    register_summon_effects::<ID, { TYPE_MONSTER | TYPE_RITUAL }>(duel, c);
}

// The effects Talismandra and Candoll share, `RITUAL` being the Ritual card type each reveals and searches.
pub fn register_summon_effects<const ID: u32, const RITUAL: u32>(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, spcost::<RITUAL>);
    e1.set_target(duel, sptg::<ID>);
    e1.set_operation(duel, spop::<ID>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_TRIGGER_O + EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_DECK));
    e2.set_target(duel, thtg::<RITUAL>);
    e2.set_operation(duel, thop::<RITUAL>);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_target_range(duel, 1, 0);
    e3.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| c.is_location(duel, LOCATION_EXTRA));
    c.register_effect(duel, e3);
    aux::add_continuous_lizard_check(duel, c, LOCATION_MZONE, |_, _, _| true);
}

fn filter<const ID: u32>(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_IMPCANTATION]) && !c.is_code(duel, ID) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn costfilter<const RITUAL: u32>(c: CardId, duel: &Duel) -> bool {
    c.is_composite_type(duel, RITUAL) && !c.is_public(duel)
}

fn spcost<const RITUAL: u32>(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(costfilter::<RITUAL>, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let g = duel.select_matching_card(tp, costfilter::<RITUAL>, tp, LOCATION_HAND, 0, 1, 1, None)?;
    duel.confirm_cards(1 - tp, g)?;
    duel.shuffle_hand(tp)?;
    Ok(false)
}

fn sptg<const ID: u32>(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let c = e.get_handler(duel);
        return Ok(duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.get_location_count(tp, LOCATION_MZONE) > 1
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && duel.is_existing_matching_card(|c, duel| filter::<ID>(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, Some(c.into())));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, tp, (LOCATION_HAND | LOCATION_DECK) as i32);
    Ok(false)
}

fn spop<const ID: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e)
        || duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some()
        || duel.get_location_count(tp, LOCATION_MZONE) < 2
    {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let mut g = duel.select_matching_card(
        tp,
        |c, duel| filter::<ID>(c, duel, e, tp),
        tp,
        LOCATION_DECK,
        0,
        1,
        1,
        Some(c.into()),
    )?;
    if !g.is_empty() {
        g.insert(c);
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn thfilter<const RITUAL: u32>(c: CardId, duel: &Duel) -> bool {
    c.is_composite_type(duel, RITUAL) && c.is_able_to_hand(duel)
}

fn thtg<const RITUAL: u32>(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter::<RITUAL>, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop<const RITUAL: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter::<RITUAL>, LOCATION_DECK)
}
