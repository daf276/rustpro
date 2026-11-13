// Galaxy-Eyes Antimatter Dragon
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 92517928;

pub const LISTED_NAMES: &[u32] = &[ID];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(9);
    const CT: usize = 2;
    const ALTERF: Option<AlterFilter> = Some(altfilter);
    const DESC: u64 = aux::stringid(ID, 0);

    fn op_check(duel: &Duel, _: EffectId, tp: u8, _: CardId) -> bool {
        duel.get_flag_effect(tp, ID) == 0
    }

    fn op(duel: &mut Duel, _: EffectId, tp: u8, _: CardId) -> Script<bool> {
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, EFFECT_FLAG_OATH, 1);
        Ok(true)
    }
}

// On the field with Xyz Import to attach the opponent's monsters (Sangan, a Fiend, for the Deck's Fiends), and in the Extra Deck
// with Level 9 monsters on the field to Xyz Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (14602126, HAND),
    (14602126, HAND),
    (16195942, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (12057781, DECK),
    (12057781, DECK),
    (12163590, MZONE),
    (12197223, MZONE),
    (53927851, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1)), Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CHANGE_BATTLE_DAMAGE);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).get_overlay_count(duel) > 0);
    e1.set_value(duel, Value::BattleDamage(aux::change_battle_damage::<1, HALF_DAMAGE>));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, _| duel.is_able_to_enter_bp());
    e2.set_cost(duel, |duel, a, chk| {
        Cost::detach_from_self_op::<1, 1>(duel, a, chk, |duel, e, og| {
            let detached_type = og.get_first().filter(|oc| oc.is_monster(duel)).map_or(0, |oc| oc.get_race(duel));
            e.set_label(duel, detached_type as i64);
        })
    });
    e2.set_target(duel, efftg);
    e2.set_operation(duel, effop);
    c.register_effect(duel, e2);
}

fn altfilter(c: CardId, duel: &Duel, _: u8, _: CardId) -> bool {
    c.is_xyz_monster(duel) && c.get_overlay_count(duel) >= 3 && c.is_faceup(duel)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(e.get_handler(duel).is_has_effect(duel, EFFECT_EXTRA_ATTACK_MONSTER).is_none());
    }
    let detached_type = e.get_label(duel);
    e.set_label(duel, 0);
    duel.set_target_param(detached_type as u32);
    if detached_type > 0 {
        duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    }
    Ok(false)
}

fn tgfilter(c: CardId, duel: &Duel, detached_type: u64) -> bool {
    c.is_race(duel, detached_type) && c.is_able_to_grave(duel)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) || c.is_has_effect(duel, EFFECT_EXTRA_ATTACK_MONSTER).is_some() {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, 3202);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_EXTRA_ATTACK_MONSTER);
    e1.set_value(duel, 1);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    c.register_effect(duel, e1);
    let detached_type = u64::from(duel.get_chain_info::<u32>(0, CHAININFO_TARGET_PARAM));
    if detached_type > 0
        && duel.is_existing_matching_card(|tc, duel| tgfilter(tc, duel, detached_type), tp, LOCATION_DECK, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
        let g = duel.select_matching_card(tp, |tc, duel| tgfilter(tc, duel, detached_type), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.break_effect()?;
            duel.send_to_grave(g, REASON_EFFECT)?;
        }
    }
    Ok(())
}
