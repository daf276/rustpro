// Relinquished
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};
use engine::constants::MSG_DAMAGE;

const ID: u32 = 64631466;
pub const LISTED_NAMES: &[u32] = &[41426869];

// On the field with the opponent's Wattkid and Thunder Kid to equip, which leave it weak enough to take battle damage.
// The damage effect is continuous and forms no chain link, so only the damage messages show it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (27324313, MZONE | OPPONENT),
    (27324313, MZONE | OPPONENT),
    (15510988, MZONE | OPPONENT),
    (15510988, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::Message(MSG_DAMAGE)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, eqcon::<ID>);
    e1.set_target(duel, eqtg);
    e1.set_operation(duel, eqop::<ID>);
    c.register_effect(duel, e1);
    aux::add_er_equip_limit(duel, c, Some(eqcon::<ID>), e1);
    add_stat_effects::<ID>(duel, c);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e4.set_code(duel, EVENT_DAMAGE);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_condition(duel, damcon);
    e4.set_operation(duel, damop);
    c.register_effect(duel, e4);
}

// The ATK and DEF effects Relinquished and Thousand-Eyes Restrict share, keyed by the card's own flag code.
pub fn add_stat_effects<const FLAG: u32>(duel: &mut Duel, c: CardId) {
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_SET_ATTACK);
    e2.set_condition(duel, adcon::<FLAG>);
    e2.set_value(duel, Value::Card(atkval::<FLAG>));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EFFECT_SET_DEFENSE);
    e3.set_condition(duel, adcon::<FLAG>);
    e3.set_value(duel, Value::Card(defval::<FLAG>));
    c.register_effect(duel, e3);
}

pub fn eqfilter<const FLAG: u32>(c: CardId, duel: &Duel) -> bool {
    c.get_flag_effect(duel, FLAG) != 0
}

pub fn eqcon<const FLAG: u32>(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel)
        .get_equip_group(duel)
        .filter(duel, eqfilter::<FLAG>, None)
        .is_empty()
}

pub fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && chkc.is_able_to_change_controler(duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_target(CardId::is_able_to_change_controler, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    let g = duel.select_target(tp, CardId::is_able_to_change_controler, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_EQUIP, g, 1, 0, 0);
    Ok(false)
}

pub fn equipop<const FLAG: u32>(duel: &mut Duel, c: CardId, e: EffectId, tp: u8, tc: CardId) -> Script<()> {
    if !c.equip_by_effect_and_limit_register(duel, e, tp, tc, Some(FLAG), None)? {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_EQUIP);
    e1.set_code(duel, EFFECT_DESTROY_SUBSTITUTE);
    e1.set_property(duel, EFFECT_FLAG_SET_AVAILABLE + EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    e1.set_value(duel, Value::Reason(|_, _, _, r, _| r & REASON_BATTLE != 0));
    tc.register_effect(duel, e1);
    Ok(())
}

pub fn eqop<const FLAG: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && tc.is_monster(duel)
        && tc.is_controler(duel, 1 - tp)
        && eqcon::<FLAG>(duel, a)
    {
        equipop::<FLAG>(duel, c, e, tp, tc)?;
    }
    Ok(())
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    !c.get_equip_group(duel).filter(duel, eqfilter::<ID>, None).is_empty()
        && a.ep == a.tp
        && a.r & REASON_BATTLE != 0
        && (duel.get_attacker() == Some(c) || duel.get_attack_target() == Some(c))
}

fn damop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.damage(1 - a.tp, a.ev, REASON_EFFECT)?;
    Ok(())
}

pub fn adcon<const FLAG: u32>(duel: &Duel, a: &Args) -> bool {
    !a.e.get_handler(duel)
        .get_equip_group(duel)
        .filter(duel, eqfilter::<FLAG>, None)
        .is_empty()
}

fn equipped_stat<const FLAG: u32>(duel: &Duel, e: EffectId, stat: fn(CardId, &Duel) -> i32) -> i32 {
    let Some(tc) = e
        .get_handler(duel)
        .get_equip_group(duel)
        .filter(duel, eqfilter::<FLAG>, None)
        .get_first()
    else {
        return 0;
    };
    let value = stat(tc, duel);
    if tc.is_facedown(duel) || tc.get_original_type(duel) & TYPE_MONSTER == 0 || value < 0 {
        0
    } else {
        value
    }
}

pub fn atkval<const FLAG: u32>(duel: &Duel, e: EffectId, _: CardId) -> i32 {
    equipped_stat::<FLAG>(duel, e, CardId::get_text_attack)
}

pub fn defval<const FLAG: u32>(duel: &Duel, e: EffectId, _: CardId) -> i32 {
    equipped_stat::<FLAG>(duel, e, CardId::get_text_defense)
}
