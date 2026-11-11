// Mathmech Nabla
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE};

const ID: u32 = 53577438;

// On the field to Tribute itself, with Mathmech Sigma and Diameter in the Deck to Special Summon, and Sigma, Leotron and Geomathmech
// Magma (Synchro Summoned into the Extra Monster Zone) for the second effect.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (27182739, DECK),
    (17946349, DECK),
    (27182739, MZONE),
    (27182739, MZONE),
    (47226949, MZONE),
    (47226949, MZONE),
    (15248594, EXTRA),
    (15248594, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, _| duel.is_able_to_enter_bp() || duel.is_battle_phase());
    e2.set_target(duel, atktg);
    e2.set_operation(duel, atkop);
    c.register_effect(duel, e2);
}

fn costfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_race(duel, RACE_CYBERSE) && duel.get_mzone_count(tp, Some(c.into())) > 0 && (c.is_controler(duel, tp) || c.is_faceup(duel))
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_MATHMECH]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, |c, duel| costfilter(c, duel, tp), (1, 1), false, None, None));
    }
    let sg = duel.select_release_group_cost(tp, |c, duel| costfilter(c, duel, tp), (1, 1), false, None, None)?;
    duel.release(sg, REASON_COST)?;
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(
        duel,
        a,
        chk,
        |c, duel| spfilter(c, duel, a.e, a.tp),
        LOCATION_DECK,
        CATEGORY_SPECIAL_SUMMON,
    )
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK, POS_FACEUP)
}

fn atkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACE_CYBERSE) && c.is_has_effect(duel, EFFECT_EXTRA_ATTACK).is_none()
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_EMZONE) && chkc.is_controler(duel, tp) && atkfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(atkfilter, tp, LOCATION_EMZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, atkfilter, tp, LOCATION_EMZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_EXTRA_ATTACK_MONSTER);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        e1.set_value(duel, 1);
        tc.register_effect(duel, e1);
        tc.register_flag_effect_full(
            duel,
            0,
            RESETS_STANDARD_PHASE_END,
            EFFECT_FLAG_CLIENT_HINT,
            1,
            0,
            aux::stringid(ID, 2),
        );
    }
    Ok(())
}
