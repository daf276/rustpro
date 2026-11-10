// Eclipse Twins
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 45852939;

// On the field and in the Extra Deck, Xyz Summoned with two Island Turtles (Level 4), with Tornado Dragons in the Graveyard
// to Special Summon and Dark Hole of the opponent to send it there.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (4042268, MZONE),
    (4042268, MZONE),
    (6983839, GRAVE),
    (6983839, GRAVE),
    (53129443, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_FIELD);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_IGNORE_IMMUNE);
    e0.set_code(duel, EFFECT_XYZ_LEVEL);
    e0.set_range(duel, LOCATION_EXTRA);
    e0.set_target_range(duel, LOCATION_MZONE, 0);
    e0.set_target_card(duel, |duel, _, c| c.is_rank(duel, &[4]));
    e0.set_value(
        duel,
        Value::XyzLevel(|duel, e, _, rc| if rc == e.get_handler(duel) { 4 } else { 0 }),
    );
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, _| duel.is_able_to_enter_bp());
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, atktg);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    Ok(chk == 0 && c.get_effect_count(duel, EFFECT_EXTRA_ATTACK) == 0 && c.get_effect_count(duel, EFFECT_EXTRA_ATTACK_MONSTER) == 0)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3202);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_EXTRA_ATTACK_MONSTER);
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn xyzfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_type(duel, TYPE_XYZ) && c.is_rank_below(duel, 4) && c.is_can_be_effect_target(duel, e)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, sg: CardSet) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && sg.is_exists(
            duel,
            |mc, duel| mc.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT),
            1,
            Some(c.into()),
        )
}

fn rescon(duel: &Duel, sg: CardSet, e: EffectId, tp: u8) -> (bool, bool) {
    (sg.is_exists(duel, |c, duel| spfilter(c, duel, e, tp, sg), 1, None), false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let g = duel.get_matching_group(
        |c, duel| xyzfilter(c, duel, e),
        tp,
        LOCATION_GRAVE,
        0,
        Some(e.get_handler(duel).into()),
    );
    let rescon = |duel: &Duel, sg, _, _| rescon(duel, sg, e, tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && aux::select_unselect_group_check(duel, g, 2, 2, &rescon));
    }
    let tg = aux::select_unselect_group(duel, g, 2, 2, &rescon, tp, HINTMSG_TARGET)?;
    duel.set_target_card(tg);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(tg), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(tg), 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mut g = duel.get_target_cards(e).unwrap_or_default();
    if g.len() != 2 || duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let sc = g
        .filter_select(
            duel,
            tp,
            |c, duel| c.is_can_be_special_summoned(duel, e, 0, tp, false, false),
            1,
            1,
            None,
        )?
        .get_first();
    if let Some(sc) = sc
        && duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP)? > 0
    {
        g.remove(sc);
        duel.overlay(sc, g)?;
    }
    Ok(())
}
