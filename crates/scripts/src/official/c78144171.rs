// Dark Armed, the Dragon of Annihilation
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 78144171;

// In the Extra Deck, Xyz Summoned from Slot Machine (Level 7), with Slot Machine in the Graveyard to banish and the opponent's Sangan to destroy. The
// alternative Xyz Summon needs exactly five DARK monsters in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (3797883, MZONE),
    (3797883, MZONE),
    (3797883, MZONE),
    (3797883, MZONE),
    (3797883, GRAVE),
    (3797883, GRAVE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(7);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
    const ALTERF: Option<AlterFilter> = Some(ovfilter);
    const DESC: u64 = aux::stringid(ID, 0);

    fn op_check(duel: &Duel, _: EffectId, tp: u8, _: CardId) -> bool {
        duel.get_flag_effect(tp, ID) == 0
    }

    fn op(duel: &mut Duel, _: EffectId, tp: u8, _: CardId) -> Script<bool> {
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
        Ok(true)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
}

fn ovfilter(c: CardId, duel: &Duel, tp: u8, lc: CardId) -> bool {
    c.is_faceup(duel)
        && c.is_race_full(duel, RACE_DRAGON, Some(lc), SUMMON_TYPE_XYZ as u64, tp)
        && c.is_attribute_full(duel, ATTRIBUTE_DARK, Some(lc), SUMMON_TYPE_XYZ as u64, tp)
        && c.is_level_above(duel, 5)
        && duel.get_matching_group_count(|c, duel| c.is_attribute(duel, ATTRIBUTE_DARK), tp, LOCATION_GRAVE, 0, None) == 5
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_remove(duel) && aux::sp_elim_filter(c, duel, false, false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, None)
            && duel.is_existing_matching_card(rmfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && duel.destroy(tc, REASON_EFFECT)? > 0
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
        let g = duel.select_matching_card(tp, rmfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.break_effect()?;
            duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
        }
    }
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, 3206);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    c.register_effect(duel, e1);
    Ok(())
}
