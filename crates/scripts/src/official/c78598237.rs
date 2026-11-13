// Vaalmonica Intonare
use super::c42193638::select_option;
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, PZONE};

const ID: u32 = 78598237;

// In the hand with Angello and Dimonno in the Pendulum Zone, and Level 4 monsters in the Graveyard to Summon or add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (3048768, PZONE),
    (30432463, PZONE),
    (38142739, GRAVE),
    (38142739, GRAVE),
    (12493482, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_RECOVER + CATEGORY_SPECIAL_SUMMON + CATEGORY_DAMAGE + CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    duel.set_possible_operation_info(0, CATEGORY_RECOVER, None, 0, tp, 500);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    duel.set_possible_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 500);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level(duel, &[4]) && c.is_able_to_hand(duel)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let op = select_option(duel, a, ID)?;
    if op == Some(1) {
        let g = duel.get_matching_group(
            aux::necro_valley_filter(|c, duel| c.is_can_be_special_summoned(duel, e, 0, tp, false, false)),
            tp,
            LOCATION_GRAVE,
            0,
            None,
        );
        if duel.recover(tp, 500, REASON_EFFECT)? > 0 && !g.is_empty() {
            duel.hint(HINT_SELECTMSG, 1 - tp, HINTMSG_SPSUMMON);
            let sg = g.select(duel, 1 - tp, 1, 1, None)?;
            duel.break_effect()?;
            duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    } else if op == Some(2) {
        let g = duel.get_matching_group(aux::necro_valley_filter(thfilter), tp, LOCATION_GRAVE, 0, None);
        if duel.damage(tp, 500, REASON_EFFECT)? > 0 && !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 5))? {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
            let hg = g.select(duel, tp, 1, 1, None)?;
            duel.break_effect()?;
            duel.send_to_hand(hg, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, hg)?;
        }
    }
    Ok(())
}
