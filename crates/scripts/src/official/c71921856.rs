// Number 79: Battlin' Boxer Nova Kaiser
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 71921856;

pub const LISTED_SERIES: &[u16] = &[SET_BATTLIN_BOXER];
pub const XYZ_NUMBER: u32 = 79;

// In the Extra Deck, Xyz Summoned with Level 4 Normal Monsters, Battlin' Boxers to attach and Special Summon, and Raigeki in the
// opponent's hand to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (549481, MZONE),
    (1184620, MZONE),
    (1784619, MZONE),
    (13313278, HAND),
    (13313278, GRAVE),
    (61679541, GRAVE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::ResolvesEffect(0)];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_value(duel, Value::Card(|duel, _, c| c.get_overlay_count(duel) as i32 * 100));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e3.set_condition_mut(duel, spcon);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_BATTLIN_BOXER]) && c.is_monster(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(a.e.get_handler(duel).is_type(duel, TYPE_XYZ)
            && duel.is_existing_matching_card(filter, a.tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, None));
    }
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, c) = (a.e, a.tp, a.e.get_handler(duel));
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_XMATERIAL);
    let g = duel.select_matching_card(
        tp,
        aux::necro_valley_filter(filter),
        tp,
        LOCATION_HAND | LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.overlay(c, g)?;
    }
    Ok(())
}

fn spcon(duel: &mut Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let ct = c.get_overlay_count(duel);
    a.e.set_label(duel, ct as i64);
    c.is_reason(duel, REASON_DESTROY)
        && c.get_reason_player(duel) != a.tp
        && c.is_previous_location(duel, LOCATION_MZONE)
        && c.is_previous_controler(duel, a.tp)
        && ct > 0
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 4) && c.is_set_card(duel, &[SET_BATTLIN_BOXER]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let filter = |c: CardId, duel: &Duel| spfilter(c, duel, e, tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_target(filter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    let mut ct = e.get_label(duel).min(i64::from(duel.get_location_count(tp, LOCATION_MZONE)));
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ct = 1;
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel
        .select_target(tp, filter, tp, LOCATION_GRAVE, 0, 1, ct as u8, None)?
        .unwrap_or_default();
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE);
    if ft <= 0 {
        return Ok(());
    }
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ft = 1;
    }
    if ft < g.len() as i32 {
        return Ok(());
    }
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
