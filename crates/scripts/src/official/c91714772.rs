// Galaxy Defense Robot Orbital 7
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 91714772;

pub const LISTED_SERIES: &[u16] = &[SET_PHOTON, SET_GALAXY, SET_GALAXY_EYES];
// In the hand and Graveyard, with Photon and Galaxy monsters in the Deck (Galaxy Serpent, Galaxy Soldier, Photon Delta Wing) and
// a Galaxy-Eyes monster to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (11066358, DECK),
    (46659709, DECK),
    (47051709, DECK),
    (47051709, DECK),
    (11066358, DECK),
    (93717133, HAND),
    (93717133, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND | LOCATION_MZONE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_cost(duel, Cost::self_tribute);
    e1.set_target(duel, thtgtg);
    e1.set_operation(duel, thtgop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon);
    e2.set_target(duel, stock::special_summon_self_target);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn thtgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_PHOTON, SET_GALAXY]) && c.is_monster(duel) && (c.is_able_to_hand(duel) || c.is_able_to_grave(duel))
}

fn rescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    (
        sg.is_exists(duel, |c, duel| c.is_set_card(duel, &[SET_PHOTON]), 1, None)
            && sg.is_exists(duel, |c, duel| c.is_set_card(duel, &[SET_GALAXY]), 1, None)
            && sg.is_exists(duel, CardId::is_able_to_hand, 1, None)
            && sg.is_exists(duel, CardId::is_able_to_grave, 1, None),
        false,
    )
}

fn thtgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let g = duel.get_matching_group(thtgfilter, tp, LOCATION_DECK, 0, None);
        return Ok(g.len() >= 2 && aux::select_unselect_group_check(duel, g, 2, 2, &rescon));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thtgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(thtgfilter, tp, LOCATION_DECK, 0, None);
    let sg = aux::select_unselect_group(duel, g, 2, 2, &rescon, tp, aux::stringid(ID, 2))?;
    if sg.len() == 2 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let hg = sg.filter_select(duel, tp, CardId::is_able_to_hand, 1, 1, None)?;
        duel.send_to_hand(hg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, hg)?;
        duel.send_to_grave(sg.difference(hg), REASON_EFFECT)?;
    }
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 3));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_attribute_except(duel, ATTRIBUTE_LIGHT));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}

fn spconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_GALAXY_EYES]) && c.is_controler(duel, tp) && c.is_faceup(duel)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| spconfilter(c, duel, tp), 1, None))
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3300);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e1.set_value(duel, LOCATION_REMOVED as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        c.register_effect_full(duel, e1, true);
    }
    Ok(())
}
