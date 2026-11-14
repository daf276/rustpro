// Junk Mail
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 39520293;

pub const LISTED_SERIES: &[u16] = &[SET_WARRIOR, SET_SYNCHRON, SET_STARDUST];
// A copy on the field and in the Graveyard, Junk Warrior on the field, and Swordsman of Landstar (Level 3) as the other
// material of Armory Arm and Toxic Boxfish (Level 4) in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, GRAVE),
    (60800381, MZONE),
    (3573512, MZONE),
    (3573512, MZONE),
    (29071332, EXTRA),
    (48882106, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::MaterialFrom(MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_code(duel, EVENT_BE_MATERIAL);
    e1.set_condition(duel, |_, a| a.r & REASON_SYNCHRO > 0);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, spcond);
    e2.set_target(duel, stock::special_summon_self_target);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(rc) = c.get_reason_card(duel) else { return Ok(()) };
    let e1 = duel.create_effect(c);
    e1.set_description(duel, 3000);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_value(duel, 1);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    rc.register_effect_full(duel, e1, true);
    Ok(())
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_SYNCHRO) && c.is_original_set_card(duel, &[SET_WARRIOR, SET_SYNCHRON, SET_STARDUST])
}

fn spcond(duel: &Duel, a: &Args) -> bool {
    aux::exccon(duel, a) && duel.is_existing_matching_card(cfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3300);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e1.set_value(duel, LOCATION_REMOVED as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        c.register_effect_full(duel, e1, true);
    }
    Ok(())
}
