// World Chalice Guardragon Almarduke
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 95793022;
const MONK: u32 = 32519092;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(link),
        FusionMaterial::Filter(link),
        FusionMaterial::Filter(link),
    ];
}

fn link(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_type_full(duel, TYPE_LINK, Some(fc), sumtype, tp)
}

struct Contact;

impl ContactFusion for Contact {
    const SUMCON: Option<Value> = Some(Value::SpSummonCondition(splimit));
    const CANNOT_BE_LIZARD: bool = false;

    fn group(duel: &Duel, tp: u8) -> CardSet {
        duel.get_release_group(tp)
    }

    fn op(duel: &mut Duel, g: CardSet, _: u8, _: CardId) -> Script<()> {
        duel.release(g, REASON_COST | REASON_MATERIAL)?;
        Ok(())
    }
}

// Contact Summoned from Monks of the Tenyi on the field, which also attack into the opponent's and pay the cost.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (MONK, MZONE),
    (MONK, MZONE),
    (MONK, MZONE),
    (MONK, GRAVE),
    (MONK, GRAVE),
    (MONK, MZONE | OPPONENT),
    (MONK, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    Fusion::add_contact_proc::<Contact>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_ATTACK_ALL);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_DAMAGE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, descon);
    e2.set_cost(duel, descost);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn splimit(duel: &Duel, e: EffectId, _: Option<EffectId>, _: u8, st: u32, _: u8, _: u8) -> bool {
    st & SUMMON_TYPE_FUSION == SUMMON_TYPE_FUSION || e.get_handler(duel).get_location(duel) != LOCATION_EXTRA
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let Some(attacker) = duel.get_attacker() else { return false };
    let defender = attacker.get_battle_target(duel);
    let tc = if attacker == c {
        defender
    } else if defender == Some(c) {
        Some(attacker)
    } else {
        None
    };
    tc.is_some_and(|tc| tc.is_link_monster(duel) && tc.is_controler(duel, 1 - a.tp))
}

fn descfilter(c: CardId, duel: &Duel, lk: u32) -> bool {
    c.is_link_monster(duel) && c.get_link(duel) == lk && c.is_able_to_remove_as_cost(duel)
}

fn descost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let tc = a.e.get_handler(duel).get_battle_target(duel);
    if chk == 0 {
        return Ok(tc.is_some_and(|tc| {
            tc.is_link_monster(duel)
                && duel.is_existing_matching_card(
                    |c, duel| descfilter(c, duel, tc.get_link(duel)),
                    tp,
                    LOCATION_MZONE | LOCATION_GRAVE,
                    0,
                    1,
                    None,
                )
        }));
    }
    let lk = tc.map_or(0, |tc| tc.get_link(duel));
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(
        tp,
        |c, duel| descfilter(c, duel, lk),
        tp,
        LOCATION_MZONE | LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let tc = a.e.get_handler(duel).get_battle_target(duel);
    duel.set_operation_info(0, CATEGORY_DESTROY, tc.map(CardSet::from), 1, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !c.is_relate_to_battle(duel) {
        return Ok(());
    }
    let tc = c.get_battle_target(duel);
    unported!(duel, tc.is_none());
    if let Some(tc) = tc
        && tc.is_relate_to_battle(duel)
        && duel.destroy(tc, REASON_EFFECT)? != 0
    {
        duel.damage(1 - a.tp, tc.get_base_attack(duel) as u32, REASON_EFFECT)?;
    }
    Ok(())
}
