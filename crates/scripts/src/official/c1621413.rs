// Dark Requiem Xyz Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 1621413;

pub const LISTED_NAMES: &[u32] = &[16195942];

// On the field, with Dark Rebellion Xyz Dragon attached by Xyz Import, to halve Sangan's ATK or negate a monster effect.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (16195942, MZONE | OPPONENT),
    (16195942, MZONE | OPPONENT),
    (14602126, HAND),
    (14602126, HAND),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(5);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, |duel, a| has_rebellion(duel, a.e.get_handler(duel)));
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, super::c16195942::target);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_NEGATE + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, discon);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
}

fn has_rebellion(duel: &Duel, c: CardId) -> bool {
    c.get_overlay_group(duel)
        .is_exists(duel, |mc, duel| mc.is_code(duel, 16195942), 1, None)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e) && !tc.is_immune_to_effect(duel, e) {
        let atk = tc.get_base_attack(duel);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_reset(duel, RESET_EVENT + RESETS_STANDARD);
        e1.set_value(duel, 0);
        tc.register_effect(duel, e1);
        if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
            let e2 = duel.create_effect(c);
            e2.set_type(duel, EFFECT_TYPE_SINGLE);
            e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
            e2.set_code(duel, EFFECT_UPDATE_ATTACK);
            e2.set_reset(duel, RESET_EVENT + RESETS_STANDARD);
            e2.set_value(duel, atk);
            c.register_effect(duel, e2);
        }
    }
    Ok(())
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    !c.is_status(duel, STATUS_BATTLE_DESTROYED)
        && a.rp != a.tp
        && a.re.is_some_and(|re| re.is_monster_effect(duel))
        && duel.is_chain_negatable(a.ev as u8)
        && has_rebellion(duel, c)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    if let Some(re) = a.re {
        let rc = re.get_handler(duel);
        if rc.is_destructable(duel) && rc.is_relate_to_effect(duel, re) {
            duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, 0, 0);
        }
    }
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_type(duel, TYPE_XYZ) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(re) = a.re else { return Ok(()) };
    if duel.negate_activation(a.ev as u8)
        && re.get_handler(duel).is_relate_to_effect(duel, re)
        && duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)? > 0
    {
        if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
            return Ok(());
        }
        let g = duel.get_matching_group(
            aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp)),
            tp,
            LOCATION_GRAVE,
            0,
            None,
        );
        if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
            duel.break_effect()?;
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let sg = g.select(duel, tp, 1, 1, None)?;
            duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
