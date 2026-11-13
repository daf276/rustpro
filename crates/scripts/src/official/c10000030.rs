// Magi Magi ☆ Magician Gal
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 10000030;

// Xyz Summoned with Level 6 Spellcasters (Trance the Magic Swordsman and Neo Aqua Madoor), with the opponent's monsters on the
// field and in the Graveyard to take or Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (14214060, MZONE),
    (14214060, MZONE),
    (49563947, MZONE),
    (49563947, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, xyz, sumtype, tp| c.is_race_full(duel, RACE_SPELLCASTER, Some(xyz), sumtype, tp));
    const LV: Option<u32> = Some(6);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, |duel, a, chk| {
        Cost::and(duel, a, chk, &[Cost::detach_from_self::<1, 1>, effcost])
    });
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
}

fn effcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_remove_as_cost, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, CardId::is_able_to_remove_as_cost, tp, LOCATION_HAND, 0, 1, 1, None)?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        if !chkc.is_controler(duel, 1 - tp) {
            return Ok(false);
        }
        match e.get_label(duel) {
            1 => return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler_can_be_changed(duel)),
            2 => return Ok(chkc.is_location(duel, LOCATION_GRAVE) && spfilter(chkc, duel, e, tp)),
            _ => {}
        }
    }
    let b1 = duel.is_existing_target(CardId::is_controler_can_be_changed, tp, 0, LOCATION_MZONE, 1, None);
    let b2 = duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, 0, LOCATION_GRAVE, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        e.set_category(duel, CATEGORY_CONTROL);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
        let g = duel.select_target(tp, CardId::is_controler_can_be_changed, tp, 0, LOCATION_MZONE, 1, 1, None)?;
        duel.set_operation_info(0, CATEGORY_CONTROL, g, 1, tp, 0);
    } else if op == Some(2) {
        e.set_category(duel, CATEGORY_SPECIAL_SUMMON);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, 0, LOCATION_GRAVE, 1, 1, None)?;
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target().filter(|tc| tc.is_relate_to_effect(duel, e)) else {
        return Ok(());
    };
    match e.get_label(duel) {
        1 => {
            duel.get_control_full(tc, tp, PHASE_END as u16, 1, 0xff, duel.get_reason_player())?;
        }
        2 => {
            duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
        }
        _ => {}
    }
    Ok(())
}
