// Dragon Buster Destruction Sword
use engine::constants::MSG_EQUIP;

use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 76218313;

pub const LISTED_NAMES: &[u32] = &[CARD_BUSTER_BLADER];

// Buster Blader on the field to equip to, and a second copy of this card in hand.
pub const SUPPORT: &[(u32, u16)] = &[(CARD_BUSTER_BLADER, MZONE), (CARD_BUSTER_BLADER, MZONE), (ID, HAND), (ID, HAND)];
pub const CHECKS: &[Check] = &[Check::Message(MSG_EQUIP), Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_category(duel, CATEGORY_EQUIP);
    e1.set_range(duel, LOCATION_HAND | LOCATION_MZONE);
    e1.set_target(duel, eqtg);
    e1.set_operation(duel, eqop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target_range(duel, 0, 1);
    e2.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| c.is_location(duel, LOCATION_EXTRA));
    e2.set_condition(duel, equipped);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_count_limit(duel, 1, ID);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_condition(duel, equipped);
    e3.set_target(duel, stock::special_summon_self_target);
    e3.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e3);
    let e4 = aux::create_continuous_lizard_check(duel, c, LOCATION_SZONE, |_, _, _| true);
    e4.set_target_range(duel, 0, 0xff);
    e4.set_condition(duel, equipped);
    c.register_effect(duel, e4);
}

fn equipped(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_equip_target(duel).is_some()
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_code(duel, CARD_BUSTER_BLADER)
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0 && duel.is_existing_target(filter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    duel.select_target(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    let c = a.e.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_EQUIP, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || c.is_location(duel, LOCATION_MZONE) && c.is_facedown(duel) {
        return Ok(());
    }
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if duel.get_location_count(tp, LOCATION_SZONE) <= 0
        || tc.get_controler(duel) != tp
        || tc.is_facedown(duel)
        || !tc.is_relate_to_effect(duel, e)
    {
        duel.send_to_grave(c, REASON_EFFECT)?;
        return Ok(());
    }
    duel.equip(tp, c, tc)?;
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_EQUIP_LIMIT);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    e1.set_value(duel, Value::Card(eqlimit));
    e1.set_label_object_group(duel, Some(tc.into()));
    c.register_effect(duel, e1);
    Ok(())
}

fn eqlimit(duel: &Duel, e: EffectId, c: CardId) -> i32 {
    i32::from(e.get_label_object_group(duel).and_then(|g| g.get_first()) == Some(c))
}
