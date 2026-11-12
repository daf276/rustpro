// Gouki Dragon Ogre
use super::c37675907::Procedure;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 69453825;

// A copy on the field and two in the Extra Deck, Synchro Summoned with Gouki Machine Suprex (a Level 4 Tuner) and Doron (Level 2),
// with Gouki Face Turn in the Deck to search, Gouki Guts in the Graveyard to add back and Raigeki in the opponent's hand.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (29099860, MZONE),
    (756652, MZONE),
    (26285557, DECK),
    (26285557, DECK),
    (7540107, GRAVE),
    (7540107, MZONE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_target(duel, deckthtg);
    e1.set_operation(duel, deckthop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, gythtg);
    e2.set_operation(duel, gythop);
    c.register_effect(duel, e2);
}

fn deckthfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GOUKI]) && c.is_spell(duel) && c.is_able_to_hand(duel)
}

fn deckthtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, deckthfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn deckthop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, deckthfilter, LOCATION_DECK)
}

fn gythfilter(c: CardId, duel: &Duel, atk: i32) -> bool {
    c.is_set_card(duel, &[SET_GOUKI]) && c.is_monster(duel) && c.get_attack(duel) < atk && c.is_able_to_hand(duel)
}

fn earthfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_attribute(duel, ATTRIBUTE_EARTH)
        && c.is_faceup(duel)
        && duel.is_existing_matching_card(|g, duel| gythfilter(g, duel, c.get_attack(duel)), tp, LOCATION_GRAVE, 0, 1, None)
}

fn gythtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && earthfilter(chkc, duel, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| earthfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, |c, duel| earthfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn gythop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let atk = tc.get_attack(duel);
        let g = duel.select_matching_card(tp, |c, duel| gythfilter(c, duel, atk), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.hint_selection(g);
            duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        }
        tc.register_flag_effect_full(
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
        e1.set_code(duel, EFFECT_ONLY_ATTACK_MONSTER);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_target_range(duel, 0, LOCATION_MZONE);
        e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_controler(duel, a.e.get_owner_player(duel)));
        e1.set_value(
            duel,
            Value::Card(|duel, e, c| i32::from(e.get_label_object_group(duel).is_some_and(|g| g.contains(c)))),
        );
        e1.set_label_object_group(duel, Some(tc.into()));
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect_full(duel, e1, true);
    }
    Ok(())
}
