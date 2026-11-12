// Gunkan Suship Shirauo-class Carrier
use super::c75215744::add_material_effects;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 21293424;
const CARD_SUSHIP_SHIRAUO: u32 = 78362751;
pub const LISTED_NAMES: &[u32] = &[CARD_SUSHIP_SHARI, CARD_SUSHIP_SHIRAUO];

// In the Extra Deck, Xyz Summoned with Gunkan Suship Shari and Shirauo, with A Legendary Ocean as the Field Spell for its field effects.
// The Gunkan Spell/Trap its search needs are not ported, so only the draw reaches a resolution.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (CARD_SUSHIP_SHARI, MZONE),
    (CARD_SUSHIP_SHARI, MZONE),
    (CARD_SUSHIP_SHIRAUO, MZONE),
    (CARD_SUSHIP_SHIRAUO, MZONE),
    (295517, HAND),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(0)];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_FIELD);
    e0.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e0.set_range(duel, LOCATION_MZONE);
    e0.set_target_range(duel, LOCATION_MZONE, 0);
    e0.set_condition(duel, contcon);
    e0.set_target_card(duel, conttg);
    e0.set_value(duel, Value::EffectPlayer(aux::indoval));
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_condition(duel, contcon);
    e1.set_target_card(duel, conttg);
    e1.set_value(duel, Value::Card(|duel, _, c| c.get_base_defense(duel)));
    c.register_effect(duel, e1);
    add_material_effects::<CARD_SUSHIP_SHIRAUO>(duel, c, ID, regtg, regop);
}

fn contcon(duel: &Duel, _: &Args) -> bool {
    duel.is_existing_matching_card(CardId::is_faceup, 0, LOCATION_FZONE, LOCATION_FZONE, 1, None)
}

fn conttg(duel: &Duel, _: EffectId, c: CardId) -> bool {
    c.is_set_card(duel, &[SET_GUNKAN]) && c.is_summon_location(duel, LOCATION_EXTRA)
}

fn sfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GUNKAN]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn regtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let effs = e.get_label(duel);
    if chk == 0 {
        return Ok((effs & 1 > 0 && duel.is_player_can_draw(tp, 1))
            || (effs & 2 > 0 && duel.is_existing_matching_card(sfilter, tp, LOCATION_DECK, 0, 1, None)));
    }
    let mut cat = 0;
    if effs & 1 > 0 {
        cat = CATEGORY_DRAW;
        duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    }
    if effs & 2 > 0 {
        cat |= CATEGORY_TOHAND | CATEGORY_SEARCH;
        duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    }
    e.set_category(duel, cat);
    Ok(false)
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let effs = e.get_label(duel);
    if effs & 1 > 0 {
        duel.draw(tp, 1, REASON_EFFECT)?;
    }
    if effs & 2 > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g = duel.select_matching_card(tp, sfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, g)?;
        }
    }
    Ok(())
}
