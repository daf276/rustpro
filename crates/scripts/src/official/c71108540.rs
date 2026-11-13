// Invincible Demise Lord
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 71108540;
const CARD_SLASH_DRAW: u32 = 71344451;

pub const LISTED_NAMES: &[u32] = &[ID, CARD_SLASH_DRAW];

// In the hand, the Graveyard and on the field, with Slash Draw in the Deck to reveal and Elfnote Lucina on the opponent's field to
// destroy by battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (ID, MZONE),
    (CARD_SLASH_DRAW, DECK),
    (CARD_SLASH_DRAW, DECK),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_PHASE + PHASE_END);
    e1.set_range(duel, LOCATION_HAND | LOCATION_GRAVE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, |duel, _| duel.has_flag_effect(0, ID));
    e1.set_target(duel, stock::special_summon_self_target);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, efftg);
    e2.set_operation(duel, effop);
    c.register_effect(duel, e2);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_BATTLE_DESTROYED);
        ge1.set_operation(duel, |duel, a| {
            duel.register_flag_effect(0, ID, RESET_PHASE | PHASE_END, 0, 1);
            if a.eg
                .is_some_and(|eg| eg.is_exists(duel, |c, duel| c.is_previous_code_on_field(duel, &[ID]), 1, None))
            {
                duel.register_flag_effect(1, ID, RESET_PHASE | PHASE_END, 0, 1);
            }
            Ok(())
        });
        duel.register_effect(ge1, 0);
    });
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.special_summon_step(c, 0, tp, tp, false, false, POS_FACEUP)? && duel.has_flag_effect(1, ID) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_BASE_ATTACK);
        e1.set_value(duel, 3000);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_description(duel, 3001);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
        e2.set_value(duel, 1);
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e2);
    }
    duel.special_summon_complete()?;
    Ok(())
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let deck_count = duel.get_field_group_count(tp, LOCATION_DECK, 0);
        return Ok(deck_count > 1
            && deck_count > duel.get_field_group_count(tp, 0, LOCATION_ONFIELD)
            && duel.is_existing_matching_card(|c, duel| c.is_code(duel, CARD_SLASH_DRAW), tp, LOCATION_DECK, 0, 1, None));
    }
    Ok(false)
}

fn rescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    let res = sg.is_exists(duel, |c, duel| c.is_code(duel, CARD_SLASH_DRAW), 1, None);
    (res, !res)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !duel.is_existing_matching_card(|c, duel| c.is_code(duel, CARD_SLASH_DRAW), tp, LOCATION_DECK, 0, 1, None) {
        return Ok(());
    }
    let reveal_count = duel.get_field_group_count(tp, 0, LOCATION_ONFIELD) + 1;
    let g = duel.get_field_group(tp, LOCATION_DECK, 0);
    if g.len() < reveal_count {
        return Ok(());
    }
    let sg = aux::select_unselect_group(duel, g, reveal_count, reveal_count, &rescon, tp, HINTMSG_CONFIRM)?;
    if sg.len() == reveal_count {
        duel.confirm_cards(1 - tp, sg)?;
        duel.shuffle_deck(tp)?;
        duel.move_to_deck_top(sg)?;
        if reveal_count > 1 {
            duel.sort_decktop(tp, tp, reveal_count as u16)?;
        }
    }
    Ok(())
}
