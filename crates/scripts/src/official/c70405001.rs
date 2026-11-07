// Black Luster Soldier - Soldier of Light and Darkness
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 70405001;

pub const LISTED_NAMES: &[u32] = &[CARD_LIGHT_AND_DARKNESS_RITUAL];

// In the hand, Ritual Summoned by Light and Darkness Ritual with two monsters in the hand, and weak Normal Monsters for it to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (CARD_LIGHT_AND_DARKNESS_RITUAL, HAND),
    (76792184, HAND),
    (30208479, HAND),
    (15025844, MZONE | OPPONENT),
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
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, Value::Effect(immval));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_REMOVE);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_count_limit(duel, 1, (ID, 0));
    e3.set_target(duel, bantg);
    e3.set_operation(duel, banop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_ATKCHANGE);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_code(duel, EVENT_BATTLE_DESTROYING);
    e4.set_count_limit(duel, 1, (ID, 1));
    e4.set_condition(duel, atkcon);
    e4.set_operation(duel, atkop);
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

fn bantg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let removable = |c: CardId, duel: &Duel| c.is_able_to_remove(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_on_field(duel) && removable(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(removable, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, removable, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, tp, 0);
    Ok(false)
}

fn banop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.remove(tc, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_relate_to_battle(duel) && c.is_status(duel, STATUS_OPPO_BATTLE) && duel.check_mut(|duel| c.can_chain_attack(duel))
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_battle(duel) && c.update_attack(duel, 1500, None, None, None) > 0 {
        duel.chain_attack();
    }
    Ok(())
}
