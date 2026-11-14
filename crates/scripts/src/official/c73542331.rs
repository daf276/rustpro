// Kashtira Shangri-Ira
use engine::constants::{MSG_FIELD_DISABLED, MSG_SELECT_EFFECTYN};

use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 73542331;

pub const LISTED_SERIES: &[u16] = &[SET_KASHTIRA];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(7);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

// On the field and in the Extra Deck for an Xyz Summon from Kashtira Fenrir and Unicorn, whose face-down banishes of the
// opponent's cards trigger the zone effect; Kashtira monsters in the Deck to Special Summon; Xyz Import to attach a material of the
// opponent's Sangan and Raigeki to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (32909498, MZONE),
    (32909498, MZONE),
    (68304193, MZONE),
    (68304193, MZONE),
    (32909498, DECK),
    (68304193, DECK),
    (13597785, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
    (14602126, HAND),
    (26202165, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
    (14602126, HAND),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::Message(MSG_FIELD_DISABLED),
    Check::Message(MSG_SELECT_EFFECTYN),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_PHASE + PHASE_STANDBY);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_REMOVE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, zcon);
    e2.set_target(duel, ztg);
    e2.set_operation(duel, zop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_code(duel, EFFECT_DESTROY_REPLACE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target(duel, reptg);
    c.register_effect(duel, e3);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_KASHTIRA]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK, POS_FACEUP)
}

fn zfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_facedown(duel) && c.is_controler(duel, 1 - tp) && c.is_previous_controler(duel, 1 - tp)
}

fn zcon(duel: &Duel, a: &Args) -> bool {
    a.eg.unwrap_or_default().is_exists(duel, |c, duel| zfilter(c, duel, a.tp), 1, None)
}

fn ztg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let count = |p, location| duel.get_location_count_full(p, location, PLAYER_NONE, 0, 0xff);
        return Ok(
            count(tp, LOCATION_MZONE) + count(tp, LOCATION_SZONE) + count(1 - tp, LOCATION_MZONE) + count(1 - tp, LOCATION_SZONE) > 0,
        );
    }
    let dis = duel
        .select_disable_field(tp, 1, LOCATION_ONFIELD as u16, LOCATION_ONFIELD as u16, 0)?
        .unwrap_or(0);
    duel.hint(HINT_ZONE, tp, u64::from(dis));
    duel.set_target_param(dis);
    Ok(false)
}

fn zop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
        let mut dis: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
        if !c.is_controler(duel, tp) {
            dis = if dis > 4096 { dis >> 16 } else { dis << 16 };
        }
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_DISABLE_FIELD);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_label(duel, i64::from(dis));
        e1.set_returning_operation(duel, |duel, a| Ok(a.e.get_label(duel) as u32));
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(!c.is_reason(duel, REASON_REPLACE) && c.check_remove_overlay_card(duel, tp, 1, REASON_EFFECT));
    }
    Ok(duel.select_effect_yes_no_full(tp, c, 96)? && c.remove_overlay_card(duel, tp, 1, 1, REASON_EFFECT)? > 0)
}
