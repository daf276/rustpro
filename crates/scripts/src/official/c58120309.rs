// Starlight Road
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 58120309;

pub const LISTED_NAMES: &[u32] = &[CARD_STARDUST_DRAGON];
// Two monsters of ours (Psychic Kappa) for the opponent's Raigeki and Dark Hole to destroy, and Stardust Dragon to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (CARD_STARDUST_DRAGON, EXTRA),
    (7892180, MZONE),
    (7892180, MZONE),
    (12580477, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::SpecialSummonedCode(CARD_STARDUST_DRAGON),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DISABLE + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_controler(duel, tp) && c.is_on_field(duel)
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    if !duel.is_chain_negatable(a.ev as u8) {
        return false;
    }
    duel.get_operation_info(a.ev as u8, CATEGORY_DESTROY).is_some_and(|info| {
        info.op_cards.is_some_and(|tg| {
            let mine = tg.filter_count(duel, |c, duel| cfilter(c, duel, tp), None) as i32;
            i32::from(info.op_count) + mine - tg.len() as i32 > 1
        })
    })
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, 0, 0);
    if let Some(re) = a.re {
        let rc = re.get_handler(duel);
        if rc.is_destructable(duel) && rc.is_relate_to_effect(duel, re) {
            duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, 0, 0);
        }
    }
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, CARD_STARDUST_DRAGON)
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(re) = a.re else { return Ok(()) };
    if duel.negate_effect(a.ev as u8)
        && re.get_handler(duel).is_relate_to_effect(duel, re)
        && duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)? > 0
    {
        let sg = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_EXTRA, 0, None);
        if sg.is_empty() || !duel.select_yes_no(tp, aux::stringid(ID, 1))? {
            return Ok(());
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sc = sg.select(duel, tp, 1, 1, None)?;
        duel.break_effect()?;
        duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
