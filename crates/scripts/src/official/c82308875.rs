// Number 7: Lucky Straight
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE};

const ID: u32 = 82308875;
pub const XYZ_NUMBER: u32 = 7;

// In the Extra Deck, Xyz Summoned with three Level 7 monsters.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (2964201, MZONE),
    (3797883, MZONE),
    (6368038, MZONE),
    (6740720, MZONE),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(7);
    const CT: usize = 3;
}

// s.roll_dice, which only cards that are not ported read, is left out.
pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(
        duel,
        CATEGORY_ATKCHANGE + CATEGORY_TOGRAVE + CATEGORY_SPECIAL_SUMMON + CATEGORY_DRAW + CATEGORY_HANDES,
    );
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DICE, None, 0, tp, 2);
    duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, PLAYER_ALL, LOCATION_ONFIELD as i32);
    duel.set_possible_operation_info(
        0,
        CATEGORY_SPECIAL_SUMMON,
        None,
        1,
        PLAYER_ALL,
        (LOCATION_HAND | LOCATION_GRAVE) as i32,
    );
    duel.set_possible_operation_info(0, CATEGORY_DRAW, None, 0, tp, 3);
    duel.set_possible_operation_info(0, CATEGORY_HANDES, None, 0, tp, 2);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, c) = (a.e, a.tp, a.e.get_handler(duel));
    if !(c.is_relate_to_effect(duel, e) && c.is_faceup(duel)) {
        return Ok(());
    }
    let &[mut d1, mut d2] = duel.toss_dice(tp, 2)?.as_slice() else {
        return Ok(());
    };
    if d2 > d1 {
        (d1, d2) = (d2, d1);
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
    e1.set_value(duel, i32::from(d1) * 700);
    e1.set_reset_full(duel, RESETS_STANDARD_DISABLE_PHASE_END, 2);
    c.register_effect(duel, e1);
    if d1 + d2 != 7 {
        return Ok(());
    }
    let b1 = duel.is_existing_matching_card(CardId::is_able_to_grave, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, Some(c.into()));
    let b2 = duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(
            aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp)),
            tp,
            LOCATION_HAND | LOCATION_GRAVE,
            LOCATION_GRAVE,
            1,
            None,
        );
    let b3 = duel.is_player_can_draw(tp, 3);
    if !(b1 || b2 || b3) {
        return Ok(());
    }
    let op = duel.select_effect(
        tp,
        &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2)), (b3, aux::stringid(ID, 3))],
    )?;
    match op {
        Some(1) => {
            let g = duel.get_matching_group(CardId::is_able_to_grave, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, Some(c.into()));
            if g.is_empty() {
                return Ok(());
            }
            duel.break_effect()?;
            duel.send_to_grave(g, REASON_EFFECT)?;
        }
        Some(2) => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let sg = duel.select_matching_card(
                tp,
                aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp)),
                tp,
                LOCATION_HAND | LOCATION_GRAVE,
                LOCATION_GRAVE,
                1,
                1,
                None,
            )?;
            if sg.is_empty() {
                return Ok(());
            }
            duel.break_effect()?;
            duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
        }
        Some(3) => {
            if !duel.is_player_can_draw(tp, 0) {
                return Ok(());
            }
            duel.break_effect()?;
            if duel.draw(tp, 3, REASON_EFFECT)? != 3 {
                return Ok(());
            }
            duel.break_effect()?;
            duel.discard_hand(tp, aux::TRUE, 2, 2, REASON_EFFECT | REASON_DISCARD)?;
        }
        _ => {}
    }
    Ok(())
}
