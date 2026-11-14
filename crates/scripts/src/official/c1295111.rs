// Salamangreat Sanctuary
use super::c74652966::{reinclinkcon, reinclinkop, reinclinktg};
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 1295111;

pub const LISTED_SERIES: &[u16] = &[SET_SALAMANGREAT];
// In the hand with Salamangreat Raging Phoenix on the field and in the Extra Deck, for the Reincarnation Link Summon, and a Link Monster to gain
// the Life Points of.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (57134592, MZONE),
    (57134592, EXTRA),
    (57134592, EXTRA),
    (7925734, MZONE),
    (7925734, MZONE),
    (60303245, MZONE),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(57134592), Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_IGNORE_IMMUNE);
    e2.set_code(duel, EFFECT_SPSUMMON_PROC);
    e2.set_range(duel, LOCATION_EXTRA);
    e2.set_condition(duel, reinclinkcon::<ID>);
    e2.set_target(duel, reinclinktg);
    e2.set_operation(duel, reinclinkop::<ID>);
    e2.set_value(duel, SUMMON_TYPE_LINK as i32);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_GRANT);
    e3.set_range(duel, LOCATION_FZONE);
    e3.set_target_range(duel, LOCATION_EXTRA, 0);
    e3.set_target_card(duel, |duel, _, c| {
        c.is_set_card(duel, &[SET_SALAMANGREAT]) && c.is_link_monster(duel)
    });
    e3.set_label_object(duel, Some(e2));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_RECOVER);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e4.set_code(duel, EVENT_PRE_DAMAGE_CALCULATE);
    e4.set_range(duel, LOCATION_FZONE);
    e4.set_count_limit(duel, 1, ID);
    e4.set_condition(duel, |duel, a| duel.get_battle_monster(a.tp).0.is_some());
    e4.set_cost(duel, Cost::pay_lp::<1000>);
    e4.set_target(duel, atktg);
    e4.set_operation(duel, atkop);
    c.register_effect(duel, e4);
}

fn atkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_link_monster(duel) && !c.is_attack(duel, &[0])
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && atkfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(atkfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATKDEF);
    let g = duel.select_target(tp, atkfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    unported!(duel, g.is_none());
    let rec = g.and_then(|g| g.get_first()).map_or(0, |tc| tc.get_base_attack(duel));
    duel.set_target_param(rec as u32);
    duel.set_operation_info(0, CATEGORY_RECOVER, None, 0, tp, rec);
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && tc.is_faceup(duel)
        && !tc.is_attack(duel, &[0])
    {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_value(duel, 0);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
        if !tc.is_attack(duel, &[0]) {
            return Ok(());
        }
        duel.recover(tp, tc.get_base_attack(duel) as u32, REASON_EFFECT)?;
    }
    Ok(())
}
