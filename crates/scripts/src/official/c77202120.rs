// Assault Synchron
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 77202120;
const SNAKE_EYES_VENGEANCE_DRAGON: u32 = 79415624;

// In the hand and the Graveyard, with a Dragon Synchro Monster on the field to be Tributed and Special Summoned back.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (SNAKE_EYES_VENGEANCE_DRAGON, MZONE),
    (SNAKE_EYES_VENGEANCE_DRAGON, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, spselftg);
    e1.set_operation(duel, spselfop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property_full(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e2.set_code(duel, EVENT_LEAVE_FIELD);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn spselftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, tp, 700);
    Ok(false)
}

fn spselfop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    if duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0 {
        duel.break_effect()?;
        duel.damage(tp, 700, REASON_EFFECT)?;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_absolute_range(duel, tp, 1, 0);
        e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| {
            c.is_location(duel, LOCATION_EXTRA) && !c.is_type(duel, TYPE_SYNCHRO)
        });
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect_full(duel, e1, true);
        let e2 = aux::create_continuous_lizard_check(duel, c, LOCATION_MZONE, |duel, _, c| !c.is_original_type(duel, TYPE_SYNCHRO));
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect_full(duel, e2, true);
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_DRAGON)
        && c.is_type(duel, TYPE_SYNCHRO)
        && c.is_location(duel, LOCATION_GRAVE | LOCATION_REMOVED)
        && c.is_faceup(duel)
        && c.is_previous_controler(duel, tp)
        && c.is_previous_location(duel, LOCATION_MZONE)
        && c.is_previous_position(duel, POS_FACEUP)
        && c.get_previous_race_on_field(duel) & RACE_DRAGON > 0
        && c.get_previous_type_on_field(duel) & TYPE_SYNCHRO > 0
        && c.is_can_be_effect_target(duel, e)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && (c.is_reason(duel, REASON_RELEASE) || c.is_location(duel, LOCATION_REMOVED))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let eg = a.eg.unwrap_or_default();
    let filter = |c, duel: &Duel| spfilter(c, duel, e, tp);
    if let Some(chkc) = chkc {
        return Ok(eg.is_contains(chkc) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && !eg.is_contains(e.get_handler(duel))
            && eg.is_exists(duel, filter, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = eg.filter_select(duel, tp, filter, 1, 1, None)?;
    duel.set_target_card(g);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(g), 1, tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) {
        duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
