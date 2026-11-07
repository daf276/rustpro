// Magician of Dark Chaos - Black Chaos
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND};

const ID: u32 = 44001993;

pub const LISTED_NAMES: &[u32] = &[CARD_LIGHT_AND_DARKNESS_RITUAL];

// In the hand, Ritual Summoned by Light and Darkness Ritual with Stealth Angecradle, with a Spell in the GY to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (CARD_LIGHT_AND_DARKNESS_RITUAL, HAND),
    (64413994, HAND),
    (83764718, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Effect(immval));
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_type(duel, EFFECT_TYPE_FIELD);
    e2a.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e2a.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e2a.set_range(duel, LOCATION_MZONE);
    e2a.set_target_range(duel, LOCATION_ONFIELD, 0);
    e2a.set_target_card(duel, |duel, _, c| c.is_spell_trap(duel));
    e2a.set_value(duel, Value::EffectPlayer(aux::indoval));
    c.register_effect(duel, e2a);
    let e2b = e2a.clone_effect(duel);
    e2b.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2b.set_code(duel, EFFECT_CANNOT_REMOVE);
    e2b.set_target_range(duel, 0, 1);
    e2b.set_player_card_target(duel, |duel, e, c, _, r| {
        c.is_spell_trap(duel) && c.is_controler(duel, e.get_handler_player(duel)) && c.is_on_field(duel) && r == REASON_EFFECT
    });
    e2b.set_value(duel, 1);
    c.register_effect(duel, e2b);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_count_limit(duel, 1, (ID, 0));
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_REMOVE);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, (ID, 1));
    e4.set_target(duel, bantg);
    e4.set_operation(duel, banop);
    c.register_effect(duel, e4);
}

fn immval(duel: &Duel, e: EffectId, re: EffectId) -> bool {
    if !(re.is_activated(duel) && e.get_owner_player(duel) == 1 - re.get_owner_player(duel)) {
        return false;
    }
    if !re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET) {
        return true;
    }
    let tg = duel.get_chain_info::<Option<CardSet>>(0, CHAININFO_TARGET_CARDS);
    !tg.is_some_and(|tg| tg.is_contains(e.get_handler(duel)))
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}

fn removable(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_able_to_remove_full(duel, Some(tp), POS_FACEDOWN, REASON_EFFECT)
}

fn bantg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_on_field(duel) && removable(chkc, duel, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| removable(c, duel, tp), tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, |c, duel| removable(c, duel, tp), tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, tp, 0);
    Ok(false)
}

fn banop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.remove(tc, POS_FACEDOWN, REASON_EFFECT)?;
    }
    Ok(())
}
