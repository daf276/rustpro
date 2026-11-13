// Jinzo - Energy Shocker
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 40275470;

pub const LISTED_NAMES: &[u32] = &[CARD_JINZO, CARD_DARK_TIME_WIZARD];

// In the hand to Normal Summon and in the Monster Zone, with the opponent's Traps and a monster that mentions Dark Time Wizard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (4881365, MZONE),
    (4206964, SZONE | OPPONENT),
    (4206964, SZONE | OPPONENT),
    (4206964, SZONE | OPPONENT),
    (10045474, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_CHANGE_CODE);
    e1.set_range(duel, LOCATION_MZONE | LOCATION_GRAVE);
    e1.set_value(duel, CARD_JINZO as i32);
    c.register_effect(duel, e1);
    let e2a = duel.create_effect(c);
    e2a.set_description(duel, aux::stringid(ID, 0));
    e2a.set_category(duel, CATEGORY_DESTROY + CATEGORY_ATKCHANGE);
    e2a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2a.set_property(duel, EFFECT_FLAG_DELAY);
    e2a.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2a.set_count_limit(duel, 1, ID);
    e2a.set_target(duel, destg);
    e2a.set_operation(duel, desop);
    c.register_effect(duel, e2a);
    let e2b = e2a.clone_effect(duel);
    e2b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2b);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_range(duel, 0, 1);
    e3.set_condition(duel, cannotactcon);
    e3.set_value(duel, Value::EffectPlayer(|duel, _, re, _| re.is_trap_effect(duel)));
    c.register_effect(duel, e3);
}

fn desfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_trap(duel) && c.is_faceup(duel)) || (c.is_spell_trap(duel) && c.is_facedown(duel))
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(desfilter, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    let g = duel.get_matching_group(aux::faceup_filter(|c, duel| c.is_trap(duel)), tp, 0, LOCATION_ONFIELD, None);
    if !g.is_empty() {
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, tp, 0);
    }
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, None, 1, 1 - tp, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(desfilter, tp, 0, LOCATION_ONFIELD, None);
    if g.is_empty() {
        return Ok(());
    }
    let fdg = g.filter(duel, |c, duel| c.is_facedown(duel), None);
    if !fdg.is_empty() {
        duel.confirm_cards(tp, fdg)?;
    }
    let traps = g.filter(duel, |c, duel| c.is_trap(duel), None);
    if traps.is_empty() {
        return Ok(());
    }
    let c = e.get_handler(duel);
    let destroy_count = duel.destroy(traps, REASON_EFFECT)?;
    if destroy_count > 0 && c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
        duel.break_effect()?;
        c.update_attack(duel, 300 * destroy_count, None, None, None);
    }
    Ok(())
}

fn cannotactconfilter(c: CardId, duel: &Duel) -> bool {
    c.lists_code(duel, CARD_DARK_TIME_WIZARD) && c.is_monster(duel) && c.is_faceup(duel)
}

fn cannotactcon(duel: &Duel, a: &Args) -> bool {
    let e = a.e;
    duel.is_existing_matching_card(
        cannotactconfilter,
        e.get_handler_player(duel),
        LOCATION_MZONE | LOCATION_GRAVE,
        0,
        1,
        Some(e.get_handler(duel).into()),
    )
}
