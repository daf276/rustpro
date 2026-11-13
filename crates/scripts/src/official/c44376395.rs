// ASHLAN U1000
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 44376395;

// In the Extra Deck, Link Summoned from Sangans (Fiend) and Cyber Falcons (Machine), with Ritual Monsters in the hand and Deck, and
// Machine Angel Ritual for a Ritual Summon, with Sangan on the opponent's field to return.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (26202165, MZONE),
    (26202165, MZONE),
    (30655537, MZONE),
    (30655537, MZONE),
    (77235086, HAND),
    (36849933, HAND),
    (39996157, HAND),
    (30208479, DECK),
    (30208479, DECK),
    (69123138, DECK),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        aux::check_different_property_binary(duel, sg, &|c, duel| c.get_race_full(duel, Some(lc), sumtype, tp))
            && aux::check_different_property_binary(duel, sg, &|c, duel| u64::from(c.get_attribute_full(duel, Some(lc), sumtype, tp)))
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_PIERCE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_target_card(duel, |duel, e, c| {
        c.is_ritual_monster(duel) && e.get_handler(duel).get_linked_group(duel).is_contains(c)
    });
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 0));
    e2.set_cost(duel, thcost);
    e2.set_target(duel, stock::operation_info_target::<CATEGORY_TOHAND, LOCATION_DECK>);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, rthcon);
    e3.set_target(duel, rthtg);
    e3.set_operation(duel, stock::first_target_to_hand);
    c.register_effect(duel, e3);
}

fn thcostfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_ritual_monster(duel)
        && !c.is_public(duel)
        && duel.is_existing_matching_card(
            |dc, duel| thfilter(dc, duel, c.get_race(duel), c.get_attribute(duel)),
            tp,
            LOCATION_DECK,
            0,
            1,
            None,
        )
}

fn thfilter(c: CardId, duel: &Duel, race: u64, attribute: u32) -> bool {
    c.is_ritual_monster(duel) && c.is_race_except(duel, race) && c.is_attribute_except(duel, attribute) && c.is_able_to_hand(duel)
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| thcostfilter(c, duel, tp), tp, LOCATION_HAND, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let g = duel.select_matching_card(tp, |c, duel| thcostfilter(c, duel, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
    let Some(rc) = g.get_first() else { return Ok(false) };
    duel.confirm_cards(1 - tp, rc)?;
    duel.shuffle_hand(tp)?;
    e.set_label_object_group(duel, Some(rc.into()));
    rc.create_effect_relation(duel, e);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(rc) = e.get_label_object_group(duel).and_then(|g| g.get_first()) else {
        return Ok(());
    };
    if !rc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let (race, attribute) = (rc.get_race(duel), rc.get_attribute(duel));
    let g = duel.select_matching_card(tp, |c, duel| thfilter(c, duel, race, attribute), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}

fn rthconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_ritual_summoned(duel) && c.is_summon_player(duel, tp)
}

fn rthcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.unwrap_or_default()
        .is_exists(duel, |c, duel| rthconfilter(c, duel, tp), 1, None)
}

fn rthtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_faceup(duel) && chkc.is_able_to_hand(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::faceup_filter(CardId::is_able_to_hand), tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(tp, aux::faceup_filter(CardId::is_able_to_hand), tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, tp, 0);
    Ok(false)
}
