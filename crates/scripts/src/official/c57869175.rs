// Fortune Lady Past
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 57869175;

pub const LISTED_SERIES: &[u16] = &[SET_FORTUNE_LADY];
// Copies on the field, with Sealmaster Meisei, a Spellcaster, in the hand and Graveyard to banish.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (2468169, HAND), (2468169, HAND), (2468169, GRAVE)];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EFFECT_SET_ATTACK);
    e1.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_SET_DEFENSE);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, lvcon);
    e3.set_operation(duel, lvop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_REMOVE);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, ID);
    e4.set_target(duel, rmtg);
    e4.set_operation(duel, rmop);
    c.register_effect(duel, e4);
}

fn atkval(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    c.get_level(duel) as i32 * 200
}

fn lvcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    duel.is_turn_player(a.tp) && c.has_level(duel) && c.is_level_below(duel, 11)
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_facedown(duel) || !c.is_relate_to_effect(duel, a.e) || c.is_level_above(duel, 12) {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_LEVEL);
    e1.set_value(duel, 1);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
    c.register_effect(duel, e1);
    Ok(())
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_SPELLCASTER) && c.is_able_to_remove(duel) && (c.is_faceup(duel) || c.is_location(duel, LOCATION_HAND))
}

fn lvfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.has_level(duel)
        && c.is_set_card(duel, &[SET_FORTUNE_LADY])
        && duel.is_existing_matching_card(rmfilter, tp, LOCATION_MZONE | LOCATION_GRAVE | LOCATION_HAND, 0, 1, Some(c.into()))
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && lvfilter(chkc, duel, tp));
    }
    let filter = |c, duel: &Duel| lvfilter(c, duel, tp);
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_APPLYTO);
    duel.select_target(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(
        0,
        CATEGORY_REMOVE,
        None,
        1,
        0,
        (LOCATION_HAND | LOCATION_GRAVE | LOCATION_MZONE) as i32,
    );
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !(tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e)) {
        return Ok(());
    }
    let level = tc.get_level(duel);
    let ch = if level == 1 {
        duel.select_option(tp, &[aux::stringid(ID, 0)])?
    } else {
        duel.select_option(tp, &[aux::stringid(ID, 0), aux::stringid(ID, 1)])?
    };
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let max = if ch == 0 { 99 } else { level.saturating_sub(1) as u8 };
    let location = LOCATION_MZONE | LOCATION_HAND | LOCATION_GRAVE;
    let rg = duel.select_matching_card(tp, rmfilter, tp, location, 0, 1, max, Some(tc.into()))?;
    if rg.is_empty() {
        return Ok(());
    }
    let ct = duel.remove(rg, POS_FACEUP, REASON_EFFECT)?;
    if ct > 0 {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_LEVEL);
        e1.set_reset(
            duel,
            if tc == e.get_handler(duel) {
                RESETS_STANDARD_DISABLE_PHASE_END
            } else {
                RESETS_STANDARD_PHASE_END
            },
        );
        e1.set_value(duel, if ch == 1 { -ct } else { ct });
        tc.register_effect(duel, e1);
    }
    Ok(())
}
