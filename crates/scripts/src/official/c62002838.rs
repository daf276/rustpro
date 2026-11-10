// Dark End Evaporation Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 62002838;

struct Params;

impl FusionSummonParams for Params {}

// In the hand and on the field, with Accel Synchro Stardust Dragon (Level 8 Dragon) in the Extra Deck to banish, Flame
// Swordsman to Fusion Summon from Flame Manipulator and Masaki, and Attack Position Sangans to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (30983281, EXTRA),
    (45231177, EXTRA),
    (34460851, MZONE),
    (44287299, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(45231177),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, hspcost);
    e1.set_target(duel, hsptg);
    e1.set_operation(duel, hspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, efftg);
    e2.set_operation(duel, effop);
    c.register_effect(duel, e2);
}

fn hspcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level(duel, &[8]) && c.is_race(duel, RACE_DRAGON) && c.is_able_to_remove_as_cost(duel)
}

fn hspcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(hspcostfilter, tp, LOCATION_EXTRA, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, hspcostfilter, tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn hsptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn hspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| !c.is_race(duel, RACE_DRAGON));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(e.get_label(duel) == 2 && chkc.is_location(duel, LOCATION_MZONE) && chkc.is_attack_pos(duel) && chkc != c);
    }
    Fusion::new_summon_eff_closure(duel, e, false);
    let b1 = Fusion::summon_eff_tg::<Params>(duel, a, 0, None)?;
    let b2 = c.is_attack_above(duel, 500)
        && c.is_defense_above(duel, 500)
        && duel.is_existing_target(CardId::is_attack_pos, tp, LOCATION_MZONE, LOCATION_MZONE, 1, Some(c.into()));
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 3)), (b2, aux::stringid(ID, 4))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        e.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
        e.set_property(duel, 0);
        Fusion::summon_eff_tg::<Params>(duel, a, 1, None)?;
    } else if op == Some(2) {
        e.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE + CATEGORY_DESTROY);
        e.set_property(duel, EFFECT_FLAG_CARD_TARGET);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let g = duel.select_target(tp, CardId::is_attack_pos, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, Some(c.into()))?;
        duel.set_operation_info(0, CATEGORY_ATKCHANGE, Some(c.into()), 1, tp, -500);
        duel.set_operation_info(0, CATEGORY_DEFCHANGE, Some(c.into()), 1, tp, -500);
        duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let op = e.get_label(duel);
    if op == 1 {
        Fusion::new_summon_eff_closure(duel, e, true);
        Fusion::summon_eff_op::<Params>(duel, a)?;
    } else if op == 2 {
        let c = e.get_handler(duel);
        if !(c.is_relate_to_effect(duel, e) && c.is_faceup(duel) && c.is_attack_above(duel, 500) && c.is_defense_above(duel, 500)) {
            return Ok(());
        }
        if c.update_attack(duel, -500, None, None, None) == -500 && c.update_defense(duel, -500, None, None, None) == -500 {
            let tc = duel.get_first_target();
            if let Some(tc) = tc
                && tc.is_relate_to_effect(duel, e)
            {
                duel.destroy(tc, REASON_EFFECT)?;
            }
        }
    }
    Ok(())
}
