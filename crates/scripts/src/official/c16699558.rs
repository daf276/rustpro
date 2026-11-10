// Indigo-Eyes Silver Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 16699558;

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_race_full(duel, RACE_DRAGON, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(8);
    const CT: usize = 2;
}

// Two in the Extra Deck, Blue-Eyes White Dragons on the field to Xyz Summon with and one in the Graveyard to revive, Sangans of the opponent to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (CARD_BLUEEYES_W_DRAGON, MZONE),
    (CARD_BLUEEYES_W_DRAGON, MZONE),
    (CARD_BLUEEYES_W_DRAGON, MZONE),
    (CARD_BLUEEYES_W_DRAGON, MZONE),
    (CARD_BLUEEYES_W_DRAGON, GRAVE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_DIRECT_ATTACK);
    e1.set_condition(duel, |duel, a| {
        !a.e.get_handler(duel)
            .get_overlay_group(duel)
            .is_exists(duel, |c, duel| c.is_type(duel, TYPE_NORMAL), 1, None)
    });
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DISABLE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 0));
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_xyz_summoned(duel));
    e2.set_target(duel, negtg);
    e2.set_operation(duel, negop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_ATKCHANGE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_negatable, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    let g = duel.get_matching_group(CardId::is_negatable, tp, 0, LOCATION_ONFIELD, None);
    duel.set_operation_info(0, CATEGORY_DISABLE, Some(g), g.len() as u8, tp, 0);
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(CardId::is_negatable, a.tp, 0, LOCATION_ONFIELD, None);
    if g.is_empty() {
        return Ok(());
    }
    let c = a.e.get_handler(duel);
    for tc in g.iter() {
        tc.negate_effects(duel, c, 0, true);
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_type(duel, TYPE_NORMAL) && c.is_faceup(duel) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE | LOCATION_REMOVED) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(
                |c, duel| spfilter(c, duel, e, tp),
                tp,
                LOCATION_GRAVE | LOCATION_REMOVED,
                0,
                1,
                None,
            ));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(
        tp,
        |c, duel| spfilter(c, duel, e, tp),
        tp,
        LOCATION_GRAVE | LOCATION_REMOVED,
        0,
        1,
        1,
        None,
    )?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        tc.update_attack(duel, 1000, Some(RESET_EVENT | RESETS_STANDARD), Some(e.get_handler(duel)), None);
    }
    Ok(())
}
