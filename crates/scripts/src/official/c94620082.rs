// Salamangreat Foxy
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 94620082;

pub const LISTED_SERIES: &[u16] = &[SET_SALAMANGREAT];

// Copies in the hand to Normal Summon and discard, in the Deck to excavate and in the GY, with Salamangreat Gazelle in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (ID, DECK),
    (ID, DECK),
    (ID, GRAVE),
    (26889158, DECK),
    (26889158, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, _| {
        duel.is_existing_matching_card(
            aux::faceup_filter(CardId::is_spell_trap),
            0,
            LOCATION_ONFIELD,
            LOCATION_ONFIELD,
            1,
            None,
        )
    });
    e2.set_cost(duel, spcost);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_field_group_count(tp, LOCATION_DECK, 0) > 2);
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SALAMANGREAT]) && c.is_able_to_hand(duel)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_field_group_count(tp, LOCATION_DECK, 0) < 3 {
        return Ok(());
    }
    duel.confirm_decktop(tp, 3)?;
    let g = duel.get_decktop_group(tp, 3).filter(duel, thfilter, None);
    if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let sg = g.select(duel, tp, 1, 1, None)?;
        duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, sg)?;
    }
    duel.shuffle_deck(tp)
}

fn spcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SALAMANGREAT]) && c.is_discardable(duel)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(spcostfilter, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    duel.discard_hand(tp, spcostfilter, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, None, 1, PLAYER_EITHER, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let filter = aux::faceup_filter(CardId::is_spell_trap);
    if c.is_relate_to_effect(duel, e)
        && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && duel.is_existing_matching_card(&filter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 3))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let g = duel.select_matching_card(tp, &filter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
        if !g.is_empty() {
            duel.hint_selection(g);
            duel.break_effect()?;
            duel.destroy(g, REASON_EFFECT)?;
        }
    }
    Ok(())
}
