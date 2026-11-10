// Rex, Freight Fur Hire
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE};

const ID: u32 = 28004531;

// The Deck has no Fur Hire Spell/Trap to search (none ported); the Graveyard effect needs another Fur Hire in the Graveyard
// and one on the field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, GRAVE), (47647354, GRAVE), (93850652, GRAVE), (93850652, MZONE)];
pub const CHECKS: &[Check] = &[
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND + CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_MAIN_END);
    e3.set_condition(duel, gthcon);
    e3.set_cost(duel, Cost::self_banish);
    e3.set_target(duel, gthtg);
    e3.set_operation(duel, gthop);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_FUR_HIRE]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - a.tp, g)?;
    }
    Ok(())
}

fn gthcon(duel: &Duel, a: &Args) -> bool {
    duel.is_main_phase()
        && duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_FUR_HIRE])),
            a.tp,
            LOCATION_MZONE,
            0,
            1,
            None,
        )
}

fn gthfilter(c: CardId, duel: &Duel, ft: i32, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_FUR_HIRE])
        && (c.is_able_to_hand(duel) || (ft > 0 && c.is_monster(duel) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)))
}

fn gthtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && gthfilter(chkc, duel, ft, e, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(
            |c, duel| gthfilter(c, duel, ft, e, tp),
            tp,
            LOCATION_GRAVE,
            0,
            1,
            Some(e.get_handler(duel).into()),
        ));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, |c, duel| gthfilter(c, duel, ft, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, g, 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, g, 1, tp, LOCATION_GRAVE as i32);
    if g.and_then(|g| g.get_first()).is_some_and(|tc| tc.is_monster(duel)) {
        duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, LOCATION_GRAVE as i32);
    }
    Ok(false)
}

fn gthop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    if !tc.is_monster(duel) {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
    } else {
        aux::to_hand_or_else(
            duel,
            tc,
            tp,
            |tc, duel| {
                tc.is_monster(duel)
                    && duel.get_location_count(tp, LOCATION_MZONE) > 0
                    && tc.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            },
            |duel| duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP).map(drop),
            aux::stringid(ID, 2),
        )?;
    }
    Ok(())
}
