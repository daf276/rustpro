// Saffira, Dragon Queen of the Voiceless Voice
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 51296484;

struct Params;

impl RitualParams for Params {
    const DESC: u64 = aux::stringid(ID, 1);

    fn filter(rit_c: CardId, duel: &Duel) -> bool {
        rit_c.is_attribute(duel, ATTRIBUTE_LIGHT) && rit_c.is_race(duel, RACE_WARRIOR | RACE_DRAGON)
    }
}

// Prayers of the Voiceless Voice in the Deck to send, with Skull Guardian, Protector of the Voiceless Voice in the Deck to
// add, and a copy in the Graveyard to Ritual Summon Sauravis, the Ancient and Ascended by Tributing Lo.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (52472775, DECK),
    (52472775, DECK),
    (10774240, DECK),
    (4810828, HAND),
    (25801745, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::self_discard);
    e1.set_target(duel, tgtg);
    e1.set_operation(duel, tgop);
    c.register_effect(duel, e1);
    let e2 = Ritual::create_proc::<Params, RITPROC_GREATER>(duel, c);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    c.register_effect(duel, e2);
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_ritual_spell(duel) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_ritual_monster(duel)
        && c.is_attribute(duel, ATTRIBUTE_LIGHT)
        && c.is_race(duel, RACE_WARRIOR | RACE_DRAGON)
        && c.is_able_to_hand(duel)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, tgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    let locations = LOCATION_DECK | LOCATION_GRAVE;
    if !g.is_empty()
        && duel.send_to_grave(g, REASON_EFFECT)? > 0
        && g.get_first().is_some_and(|tc| tc.is_location(duel, LOCATION_GRAVE))
        && duel.is_existing_matching_card(aux::necro_valley_filter(thfilter), tp, locations, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let sg = duel.select_matching_card(tp, aux::necro_valley_filter(thfilter), tp, locations, 0, 1, 1, None)?;
        duel.break_effect()?;
        duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, sg)?;
    }
    Ok(())
}
