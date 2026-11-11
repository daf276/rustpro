// Nobledragon Magician
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, PZONE};

const ID: u32 = 88935103;
const STARGAZER_MAGICIAN: u32 = 94415058;
const ODD_EYES_METEORBURST_DRAGON: u32 = 80696379;

// The scale beside Stargazer Magician keeps it from self-destructing; Odd-Eyes Meteorburst Dragon (Level 7) is the target.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (STARGAZER_MAGICIAN, PZONE),
    (ID, HAND),
    (ID, GRAVE),
    (ODD_EYES_METEORBURST_DRAGON, MZONE),
    (ODD_EYES_METEORBURST_DRAGON, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_SELF_DESTROY);
    e2.set_range(duel, LOCATION_PZONE);
    e2.set_condition(duel, descon);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_CANNOT_BE_SYNCHRO_MATERIAL);
    e3.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e3.set_value(duel, Value::Card(|duel, _, c| (!c.is_race(duel, RACE_DRAGON)).into()));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE);
    e4.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
    e4.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e4.set_value(duel, LOCATION_DECKBOT as i32);
    e4.set_condition(duel, rdcon);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_type(duel, EFFECT_TYPE_FIELD);
    e5.set_property(
        duel,
        EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_SET_AVAILABLE + EFFECT_FLAG_IGNORE_RANGE,
    );
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_code(duel, EFFECT_MATERIAL_CHECK);
    e5.set_value(duel, Value::MaterialCheck(valcheck));
    e5.set_label_object(duel, Some(e4));
    c.register_effect(duel, e5);
    let e6 = duel.create_effect(c);
    e6.set_description(duel, aux::stringid(ID, 1));
    e6.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e6.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e6.set_type(duel, EFFECT_TYPE_IGNITION);
    e6.set_range(duel, LOCATION_HAND | LOCATION_GRAVE);
    e6.set_target(duel, sptg);
    e6.set_operation(duel, spop);
    c.register_effect(duel, e6);
}

fn descon(duel: &Duel, a: &Args) -> bool {
    !duel.is_existing_matching_card(
        |c, duel| c.is_set_card(duel, &[SET_MAGICIAN]),
        a.e.get_handler_player(duel),
        LOCATION_PZONE,
        0,
        1,
        Some(a.e.get_handler(duel).into()),
    )
}

fn rdcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_reason(duel, REASON_MATERIAL) && c.is_reason(duel, REASON_SYNCHRO) && a.e.get_label(duel) == 1
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.get_level(duel) >= 7 && c.is_set_card(duel, &[SET_ODD_EYES])
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && cfilter(chkc, duel));
    }
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.is_existing_target(cfilter, tp, LOCATION_MZONE, 0, 1, None)
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, cfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_facedown(duel) || !tc.is_relate_to_effect(duel, e) || tc.is_immune_to_effect(duel, e) || tc.get_level(duel) < 4 {
        return Ok(());
    }
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_LEVEL);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    e1.set_value(duel, -3);
    tc.register_effect(duel, e1);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    Ok(())
}

fn sfilter(c: CardId, duel: &Duel) -> bool {
    !c.is_set_card(duel, &[SET_ODD_EYES])
}

fn valcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let label = i64::from(c.get_material(duel).is_exists(duel, sfilter, 1, Some(e.get_handler(duel).into())));
    if let Some(e4) = e.get_label_object(duel) {
        e4.set_label(duel, label);
    }
}
