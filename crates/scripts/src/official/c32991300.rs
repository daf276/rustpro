// Fiendsmith's Agnumday
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 32991300;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(3);

    fn specialchk(duel: &Duel, g: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        g.is_exists(duel, |c, duel| matfilter(c, duel, lc, sumtype, tp), 1, None)
    }
}

// In the Extra Deck, Link Summoned with Evil HERO Adusted Gold (LIGHT Fiend), with LIGHT Fiends in the Graveyard to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (13650422, MZONE),
    (13650422, MZONE),
    (13650422, MZONE),
    (13650422, GRAVE),
    (28803166, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn matfilter(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_attribute_full(duel, ATTRIBUTE_LIGHT, Some(lc), sumtype, tp) && c.is_race_full(duel, RACE_FIEND, Some(lc), sumtype, tp)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_attribute(duel, ATTRIBUTE_LIGHT)
        && !c.is_link_monster(duel)
        && c.is_race(duel, RACE_FIEND)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp));
    }
    let filter = |c, duel: &Duel| spfilter(c, duel, e, tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_target(filter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, filter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_EQUIP, Some(e.get_handler(duel).into()), 1, tp, 0);
    Ok(false)
}

fn atkvalfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster_card(duel) && c.is_original_type(duel, TYPE_LINK) && c.is_faceup(duel)
}

fn atkval(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    let equipped = c.get_equip_group(duel).filter(duel, atkvalfilter, None);
    600 * equipped.get_sum(duel, |c, duel| i64::from(c.get_link(duel))) as i32
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e)
        && duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && c.is_relate_to_effect(duel, e)
        && c.is_controler(duel, tp)
        && duel.equip(tp, c, tc)?
    {
        c.register_flag_effect_full(
            duel,
            ID,
            RESET_EVENT | RESETS_STANDARD,
            EFFECT_FLAG_CLIENT_HINT,
            1,
            0,
            aux::stringid(ID, 1),
        );
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_EQUIP);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, Value::Card(atkval));
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_EQUIP);
        e2.set_code(duel, EFFECT_PIERCE);
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e2);
        let e3 = duel.create_effect(c);
        e3.set_type(duel, EFFECT_TYPE_SINGLE);
        e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e3.set_code(duel, EFFECT_EQUIP_LIMIT);
        e3.set_value(duel, Value::Card(super::c38423248::equip_limit));
        e3.set_label_object_group(duel, Some(tc.into()));
        e3.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e3);
    }
    Ok(())
}
