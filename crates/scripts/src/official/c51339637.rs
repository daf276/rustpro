// Salamangreat Roar
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, SZONE};

const ID: u32 = 51339637;

pub const LISTED_SERIES: &[u16] = &[SET_WINDWITCH];

// Set and in the GY, with Salamangreat Raging Phoenix on the field and in the Extra Deck and three Link Apple, FIRE Effect
// Monsters, to Link Summon it with a copy as material (a Reincarnation Summon).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (ID, GRAVE),
    (57134592, MZONE),
    (57134592, MZONE),
    (57134592, EXTRA),
    (57134592, EXTRA),
    (7925734, MZONE),
    (7925734, MZONE),
    (7925734, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    aux::enable_check_reincarnation(duel);
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, condition);
    e1.set_target(duel, stock::negate_destroy_target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, setcon);
    e2.set_target(duel, settg);
    e2.set_operation(duel, setop);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_SALAMANGREAT]) && c.is_link_monster(duel)
}

fn condition(duel: &Duel, a: &Args) -> bool {
    if !duel.is_existing_matching_card(cfilter, a.tp, LOCATION_MZONE, 0, 1, None) {
        return false;
    }
    let Some(re) = a.re else { return false };
    duel.is_chain_negatable(a.ev as u8) && (re.is_monster_effect(duel) || re.is_has_type(duel, EFFECT_TYPE_ACTIVATE))
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    if duel.negate_activation(a.ev as u8)
        && let Some(re) = a.re
        && re.get_handler(duel).is_relate_to_effect(duel, re)
    {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}

fn lkfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_SALAMANGREAT]) && c.is_link_monster(duel) && c.is_reincarnation_summoned(duel) && c.is_controler(duel, tp)
}

fn setcon(duel: &Duel, a: &Args) -> bool {
    a.eg.unwrap_or_default().is_exists(duel, |c, duel| lkfilter(c, duel, a.tp), 1, None)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    if chkc.is_some() {
        return Ok(false);
    }
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_ssetable(duel));
    }
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && c.is_ssetable(duel) {
        duel.sset(a.tp, c)?;
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3300);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        e1.set_value(duel, LOCATION_REMOVED as i32);
        c.register_effect(duel, e1);
    }
    Ok(())
}
