// Gem-Knight Quartz
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND};

const ID: u32 = 35622739;

pub const LISTED_NAMES: &[u32] = &[ID];
pub const LISTED_SERIES: &[u16] = &[SET_FUSION, SET_GEM_KNIGHT];

// Future Fusion in the Deck to Set; Gem-Knight Ruby in the Extra Deck, Fusion Summoned by Polymerization with this card and
// Gem-Knight Garnet, which it then adds back from the GY.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (77565204, DECK),
    (77565204, DECK),
    (76614340, EXTRA),
    (76614340, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (91731841, HAND),
    (91731841, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| duel.get_field_group_count(a.tp, 0, LOCATION_MZONE) > 0);
    e1.set_cost(duel, Cost::self_discard);
    e1.set_target(duel, settg);
    e1.set_operation(duel, setop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_BE_MATERIAL);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, thcon);
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_continuous_spell(duel) && c.is_set_card(duel, &[SET_FUSION]) && c.is_ssetable(duel)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.is_existing_matching_card(setfilter, a.tp, LOCATION_DECK, 0, 1, None))
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_matching_card(tp, setfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.sset(tp, g)?;
    }
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    e1.set_target_card(duel, |duel, _, c| {
        c.is_location(duel, LOCATION_EXTRA) && !c.is_set_card(duel, &[SET_GEM_KNIGHT])
    });
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| !c.is_original_set_card(duel, &[SET_GEM_KNIGHT]));
    Ok(())
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    a.r & REASON_FUSION == REASON_FUSION && c.is_location(duel, LOCATION_GRAVE | LOCATION_REMOVED) && c.is_faceup(duel)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GEM_KNIGHT]) && c.is_monster(duel) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_GRAVE, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_GRAVE)
}
