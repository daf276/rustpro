// World of Spirits
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 5414777;

pub const LISTED_NAMES: &[u32] = &[CARD_ANCIENT_FAIRY_DRAGON];
// In hand with Ancient Fairy Life Dragon on the field; the search finds nothing, since only Extra Deck monsters mention the dragon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (43321985, MZONE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_POSITION);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_condition(duel, lightdragonsynchrocon);
    e2.set_target(duel, postg);
    e2.set_operation(duel, posop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_PHASE + PHASE_END);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, lightdragonsynchrocon);
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.lists_code(duel, CARD_ANCIENT_FAIRY_DRAGON) && c.is_able_to_hand(duel)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None) && duel.select_yes_no(tp, aux::stringid(ID, 3))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, g)?;
        }
    }
    Ok(())
}

fn lightdragonsynchroconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level_above(duel, 7)
        && c.is_attribute(duel, ATTRIBUTE_LIGHT)
        && c.is_race(duel, RACE_DRAGON)
        && c.is_type(duel, TYPE_SYNCHRO)
        && c.is_faceup(duel)
}

fn lightdragonsynchrocon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(lightdragonsynchroconfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn attack_position(c: CardId, duel: &Duel) -> bool {
    c.is_position(duel, POS_ATTACK)
}

fn postg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let g = a.eg.unwrap_or_default().filter(duel, attack_position, None);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_target_card(g);
    duel.set_operation_info(0, CATEGORY_POSITION, Some(g), g.len() as u8, a.tp, 0);
    Ok(false)
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_target_cards(a.e).unwrap_or_default().filter(duel, attack_position, None);
    if !g.is_empty() {
        duel.change_position_full(
            g,
            [POS_FACEUP_DEFENSE, POS_FACEDOWN_DEFENSE, POS_FACEUP_DEFENSE, POS_FACEUP_DEFENSE],
            false,
        )?;
    }
    Ok(())
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let g = duel.get_matching_group(attack_position, duel.get_turn_player(), LOCATION_MZONE, 0, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, a.tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, _: &Args) -> Script<()> {
    let g = duel.get_matching_group(attack_position, duel.get_turn_player(), LOCATION_MZONE, 0, None);
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
