// Dragon Gate
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 9567495;

// In the Extra Deck, Xyz Summoned with Level 6 Normal Monsters (Luster Dragon #2, Sword Arm of Dragon).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (17658803, MZONE),
    (17658803, MZONE),
    (13069066, MZONE),
    (13069066, MZONE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(6);
    const CT: usize = 2;
    const MAXCT: usize = 2;
    const ALTERF: Option<AlterFilter> = Some(ovfilter);
    const DESC: u64 = aux::stringid(ID, 0);

    fn op_check(duel: &Duel, _: EffectId, tp: u8, _: CardId) -> bool {
        !duel.has_flag_effect(tp, ID)
    }

    fn op(duel: &mut Duel, _: EffectId, tp: u8, _: CardId) -> Script<bool> {
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, EFFECT_FLAG_OATH, 1);
        Ok(true)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_CANNOT_BE_XYZ_MATERIAL);
    e0.set_condition(duel, cannotxyzmatcon);
    e0.set_value(duel, 1);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, atktg);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
}

fn ovfilter(c: CardId, duel: &Duel, tp: u8, lc: CardId) -> bool {
    c.is_rank_below(duel, 3) && c.is_type_full(duel, TYPE_XYZ, Some(lc), SUMMON_TYPE_XYZ as u64, tp) && c.is_faceup(duel)
}

fn cannotxyzmatcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_status(duel, STATUS_SPSUMMON_TURN) && c.is_xyz_summoned(duel)
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.check_remove_overlay_card(duel, a.tp, 1, REASON_EFFECT));
    }
    duel.set_operation_info(0, CATEGORY_ATKCHANGE, Some(c.into()), 1, a.tp, 1000);
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_ATTACK_ALL);
    e1.set_value(duel, 1);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    c.register_effect(duel, e1);
    let overlay_count = c.get_overlay_count(duel) as u16;
    if c.remove_overlay_card(duel, tp, 1, overlay_count, REASON_EFFECT)? > 0 && c.is_faceup(duel) {
        let atk = 1000
            * duel
                .get_operated_group()
                .get_class_count(duel, |c, duel| i64::from(c.get_main_card_type(duel))) as i32;
        if c.update_attack(duel, atk, Some(RESETS_STANDARD_DISABLE_PHASE_END), None, None) == atk {
            let g = duel.get_matching_group(CardId::is_faceup, tp, 0, LOCATION_MZONE, None);
            for tc in g {
                tc.update_attack(duel, -atk, Some(RESETS_STANDARD_PHASE_END), Some(c), None);
            }
        }
    }
    Ok(())
}
