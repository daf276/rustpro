// Magical Musketeer Calamity
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 68024506;

pub const LISTED_SERIES: &[u16] = &[SET_MAGICAL_MUSKET];

const DESPERADO: u32 = 20745268;

// On the field in several columns, with Magical Musket - Desperado in the hand to activate in one of them, and Magical Musketeer
// Kidbrave in the GY to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (DESPERADO, HAND),
    (DESPERADO, HAND),
    (DESPERADO, HAND),
    (DESPERADO, HAND),
    (5230799, GRAVE),
    (5230799, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_QP_ACT_IN_NTPHAND);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_HAND, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_MAGICAL_MUSKET]));
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_TRAP_ACT_IN_HAND);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, ID);
    e3.set_condition(duel, spcon);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    if duel.get_current_phase() & (PHASE_DAMAGE | PHASE_DAMAGE_CAL) > 0 || !re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        return false;
    }
    let p = duel.get_chain_info::<u8>(a.ev as u8, CHAININFO_TRIGGERING_CONTROLER);
    let seq = duel.get_chain_info::<u32>(a.ev as u8, CHAININFO_TRIGGERING_SEQUENCE);
    a.e.get_handler(duel).is_column(duel, seq, p, LOCATION_SZONE)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_MAGICAL_MUSKET])
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) {
        duel.special_summon(tc, 0, a.tp, a.tp, false, false, POS_FACEUP_DEFENSE)?;
    }
    Ok(())
}
