// The Phantom Knights of Malevolent Scythe
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE, REMOVED};

const ID: u32 = 78449284;

pub const LISTED_SERIES: &[u16] = &[SET_THE_PHANTOM_KNIGHTS, SET_XYZ_DRAGON, SET_RANK_UP_MAGIC];
// Doomed Soleret: two Level 3 DARK materials, the Deck monster to Special Summon and the banished one to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (83566725, MZONE),
    (83566725, MZONE),
    (83566725, DECK),
    (83566725, REMOVED),
];
// No ported card is an "Xyz Dragon" monster to Xyz Summon for the Rank-Up-Magic search.
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_attribute_full(duel, ATTRIBUTE_DARK, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(3);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, deckthcon);
    e2.set_target(duel, deckthtg);
    e2.set_operation(duel, deckthop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_target(duel, banthtg);
    e3.set_operation(duel, banthop);
    c.register_effect(duel, e3);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_THE_PHANTOM_KNIGHTS]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK, POS_FACEUP)
}

fn deckthconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_summon_player(duel, tp) && c.is_xyz_summoned(duel) && c.is_set_card(duel, &[SET_XYZ_DRAGON]) && c.is_faceup(duel)
}

fn deckthcon(duel: &Duel, a: &Args) -> bool {
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| deckthconfilter(c, duel, a.tp), 1, None))
}

fn deckthfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_RANK_UP_MAGIC]) && c.is_spell(duel) && c.is_able_to_hand(duel)
}

fn deckthtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, deckthfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn deckthop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, deckthfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() && duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? > 0 {
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}

fn banthfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_THE_PHANTOM_KNIGHTS]) && !c.is_xyz_monster(duel) && c.is_faceup(duel) && c.is_able_to_hand(duel)
}

fn banthtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, banthfilter, LOCATION_REMOVED, CATEGORY_TOHAND)
}

fn banthop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, banthfilter, tp, LOCATION_REMOVED, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
