// Gigantic "Champion" Sargas
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 11132674;

pub const LISTED_SERIES: &[u16] = &[SET_SPRINGANS, SET_THERION];
// On the field with Samuel, which detaches a material that Xyz Import took from the opponent's Sangan, and Springans Kitt and
// Therion "King" Regulus in the Deck to search. No Springans Xyz Monster is ported, so the alternative Xyz Summon is not reached.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (76213610, MZONE),
    (14602126, HAND),
    (14602126, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (33734439, GRAVE),
    (45484331, DECK),
    (45484331, DECK),
    (10604644, DECK),
    (10604644, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(8);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
    const ALTERF: Option<AlterFilter> = Some(ovfilter);
    const DESC: u64 = aux::stringid(ID, 0);

    fn op_check(duel: &Duel, _: EffectId, tp: u8, _: CardId) -> bool {
        duel.get_flag_effect(tp, ID) == 0
    }

    fn op(duel: &mut Duel, _: EffectId, tp: u8, _: CardId) -> Script<bool> {
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
        Ok(true)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).get_overlay_count(duel) > 0);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 2));
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_DETACH_MATERIAL);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |_, a| a.r & REASON_LOST_TARGET == 0);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
    duel.enable_global_flag(GLOBALFLAG_DETACH_EVENT);
}

fn ovfilter(c: CardId, duel: &Duel, tp: u8, lc: CardId) -> bool {
    c.is_faceup(duel)
        && c.is_set_card_full(duel, &[SET_SPRINGANS], Some(lc), SUMMON_TYPE_XYZ as u64, tp)
        && c.is_type_full(duel, TYPE_XYZ, Some(lc), SUMMON_TYPE_XYZ as u64, tp)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SPRINGANS, SET_THERION]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|_, _| true, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_target(tp, |_, _| true, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, g, 1, tp, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        let op = duel.select_effect(
            tp,
            &[(true, aux::stringid(ID, 3)), (tc.is_able_to_hand(duel), aux::stringid(ID, 4))],
        )?;
        if op == Some(1) {
            duel.destroy(tc, REASON_EFFECT)?;
        } else {
            duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
        }
    }
    Ok(())
}
