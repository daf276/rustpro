// Cosmo Queen the Queen of Prayers
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 85314178;

// Field Spells (Umi, Wasteland) to activate from hand, to search from the Deck, and in the opponent's Field Zone.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (22702055, HAND),
    (23424603, HAND),
    (22702055, DECK),
    (23424603, DECK),
    (23424603, DECK),
    (22702055, OPPONENT),
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
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| {
        a.re.is_some_and(|re| re.is_active_type(duel, TYPE_FIELD) && re.is_has_type(duel, EFFECT_TYPE_ACTIVATE))
    });
    e1.set_target(duel, sptg);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP_DEFENSE>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, fieldefftg);
    e2.set_operation(duel, fieldeffop);
    c.register_effect(duel, e2);
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}

fn tgfieldfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    if !c.is_faceup(duel) {
        return false;
    }
    if c.is_controler(duel, tp) {
        let code = c.get_code(duel);
        duel.is_existing_matching_card(|c, duel| thfilter(c, duel, Some(code)), tp, LOCATION_DECK, 0, 1, None)
    } else {
        c.is_negatable_spell_trap(duel) && duel.is_existing_matching_card(|c, duel| thfilter(c, duel, None), tp, LOCATION_DECK, 0, 1, None)
    }
}

fn thfilter(c: CardId, duel: &Duel, code: Option<u32>) -> bool {
    c.is_field_spell(duel) && c.is_able_to_hand(duel) && !code.is_some_and(|code| c.is_code(duel, code))
}

fn fieldefftg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_FZONE) && tgfieldfilter(chkc, duel, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| tgfieldfilter(c, duel, tp), tp, LOCATION_FZONE, LOCATION_FZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_target(
        tp,
        |c, duel| tgfieldfilter(c, duel, tp),
        tp,
        LOCATION_FZONE,
        LOCATION_FZONE,
        1,
        1,
        None,
    )?;
    let Some(tc) = g.and_then(|g| g.get_first()) else {
        return Ok(false);
    };
    if tc.is_controler(duel, tp) {
        e.set_label(duel, 1);
        e.set_category(duel, CATEGORY_DESTROY + CATEGORY_TOHAND + CATEGORY_SEARCH);
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(tc.into()), 1, tp, 0);
    } else {
        e.set_label(duel, 2);
        e.set_category(duel, CATEGORY_DISABLE + CATEGORY_TOHAND + CATEGORY_SEARCH);
        duel.set_operation_info(0, CATEGORY_DISABLE, Some(tc.into()), 1, tp, 0);
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn fieldeffop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let code = match e.get_label(duel) {
        1 => {
            let code = tc.get_code(duel);
            if duel.destroy(tc, REASON_EFFECT)? == 0 {
                return Ok(());
            }
            Some(code)
        }
        2 => {
            if !(tc.is_negatable_spell_trap(duel) && tc.is_can_be_disabled_by_effect(duel, e, true)) {
                return Ok(());
            }
            let c = e.get_handler(duel);
            tc.negate_effects(duel, c, RESET_PHASE | PHASE_END, true);
            duel.adjust_instantly(Some(tc));
            None
        }
        _ => return Ok(()),
    };
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, |c, duel| thfilter(c, duel, code), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, tp, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}
