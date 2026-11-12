// Power Tool Braver Dragon
use super::c97268402::negate;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE, OPPONENT};

const ID: u32 = 63265554;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// In the Extra Deck, Synchro Summoned with Scrap Soldier (a Level 5 Tuner) and Zombino (Level 4), with Equip Spells (Axe of Despair, United We
// Stand, Black Pendant) in the Deck to equip and the opponent's monsters (Sangan) to negate or turn.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (65503206, MZONE),
    (65503206, MZONE),
    (14575467, MZONE),
    (14575467, MZONE),
    (40619825, DECK),
    (40619825, DECK),
    (56747793, DECK),
    (56747793, DECK),
    (65169794, DECK),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, eqtg);
    e1.set_operation(duel, eqop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DISABLE + CATEGORY_POSITION);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END | TIMINGS_CHECK_MONSTER);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, _| duel.is_main_phase());
    e2.set_cost(duel, discost);
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
}

fn eqsfilter(c: CardId, duel: &Duel, tp: u8, ec: CardId) -> bool {
    c.is_type(duel, TYPE_EQUIP) && c.check_equip_target(duel, ec) && c.check_unique_on_field(duel, tp)
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let c = e.get_handler(duel);
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_matching_card(
                |c1, duel| eqsfilter(c1, duel, tp, c),
                tp,
                LOCATION_DECK | LOCATION_GRAVE,
                0,
                1,
                None,
            ));
    }
    duel.set_operation_info(0, CATEGORY_EQUIP, None, 1, 0, (LOCATION_DECK | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_facedown(duel) || !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let filter = aux::necro_valley_filter(|c1, duel| eqsfilter(c1, duel, tp, c));
    let g = duel.get_matching_group(filter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, None);
    if g.is_empty() {
        return Ok(());
    }
    let ft = duel.get_location_count(tp, LOCATION_SZONE).min(3);
    let sg = aux::select_unselect_group(duel, g, 1, ft as usize, &aux::dncheck, tp, HINTMSG_EQUIP)?;
    if !sg.is_empty() {
        for tc in sg {
            duel.equip_full(tp, tc, c, true, true)?;
        }
        duel.equip_complete()?;
    }
    Ok(())
}

fn tgfilter(c: CardId, duel: &Duel, tc: CardId) -> bool {
    c.is_type(duel, TYPE_EQUIP) && c.is_spell(duel) && c.is_able_to_grave_as_cost(duel) && c.get_equip_target(duel) == Some(tc)
}

fn discost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c1, duel| tgfilter(c1, duel, c), tp, LOCATION_SZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, |c1, duel| tgfilter(c1, duel, c), tp, LOCATION_SZONE, 0, 1, 1, None)?;
    duel.send_to_grave(g, REASON_COST)?;
    Ok(false)
}

fn disfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_EFFECT) && (c.is_negatable_monster(duel) || c.is_can_change_position(duel))
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && disfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(disfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SELECT);
    duel.select_target(tp, disfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let b1 = tc.is_can_change_position(duel);
    let b2 = tc.is_negatable_monster(duel);
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?;
    if op == Some(1) {
        duel.change_position_full(tc, [POS_FACEUP_DEFENSE, 0, POS_FACEUP_ATTACK, 0], false)?;
    } else if op == Some(2) {
        let c = e.get_handler(duel);
        duel.negate_related_chain(tc, RESET_TURN_SET);
        negate(duel, c, tc, EFFECT_FLAG_CANNOT_DISABLE);
    }
    Ok(())
}
