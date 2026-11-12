// Tricorn the Cacophonous Concert
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 78905039;

// In the hand, with Raigeki for the opponent to destroy the monsters in MZONE. On the field with Swordsman of Landstar
// (Level 3) for a Level 5 Synchro Summon of Accel Synchron (DARK Machine), and Cyber Commander (DARK Machine) in the
// Graveyard to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (ID, MZONE),
    (3573512, MZONE),
    (3573512, MZONE),
    (37675907, EXTRA),
    (37675907, EXTRA),
    (6400512, GRAVE),
    (6400512, GRAVE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_DESTROYED);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcond);
    e1.set_target(duel, stock::special_summon_self_target);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_ONFIELD));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, stock::first_target_to_hand);
    c.register_effect(duel, e2);
}

fn spconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
        && c.is_previous_controler(duel, tp)
        && (c.is_previous_location(duel, LOCATION_MZONE) || (!c.is_previous_location(duel, LOCATION_ONFIELD) && c.is_monster(duel)))
}

fn spcond(duel: &Duel, a: &Args) -> bool {
    a.eg.unwrap_or_default()
        .is_exists(duel, |c, duel| spconfilter(c, duel, a.tp), 1, None)
}

fn cfilter(c: CardId, duel: &Duel, rac: u64, att: u32) -> bool {
    c.is_faceup(duel) && c.is_race(duel, rac) && c.is_attribute(duel, att)
}

fn thfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_monster(duel)
        && c.is_able_to_hand(duel)
        && duel.is_existing_matching_card(
            |mc, duel| cfilter(mc, duel, c.get_race(duel), c.get_attribute(duel)),
            tp,
            LOCATION_MZONE,
            0,
            1,
            None,
        )
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && thfilter(chkc, duel, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| thfilter(c, duel, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, |c, duel| thfilter(c, duel, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, tp, 0);
    Ok(false)
}
