// Epurrely Happiness
use super::c24434049::qpovcon;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 52645235;

pub const LISTED_NAMES: &[u32] = &[82105704];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(2);
    const CT: usize = 2;
}

// In the Extra Deck, Xyz Summoned with two Level 2 monsters, with Purrely cards in the Deck to search and Quick-Play Spells in the hand to
// attach, and Sangan to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (756652, MZONE),
    (2830619, MZONE),
    (12482652, MZONE),
    (10598400, MZONE),
    (82105704, DECK),
    (82105704, DECK),
    (55584558, DECK),
    (55584558, HAND),
    (82105704, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_DAMAGE_STEP_END);
    e1.set_condition(duel, thcon);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_BATTLED);
    e2.set_operation(duel, |duel, a| {
        a.e.get_handler(duel)
            .register_flag_effect(duel, ID, RESET_PHASE | PHASE_DAMAGE, 0, 1);
        Ok(())
    });
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 3, 0);
    e3.set_condition(duel, qpovcon);
    e3.set_target(duel, qpovtg);
    e3.set_operation(duel, qpovop);
    c.register_effect(duel, e3);
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.has_flag_effect(duel, ID) && (c.is_relate_to_battle(duel) || c.is_reason(duel, REASON_BATTLE))
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_PURRELY]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e)
        && c.get_overlay_group(duel)
            .is_exists(duel, |c, duel| c.is_code(duel, 82105704), 1, None)
        && duel.is_existing_matching_card(CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATKDEF);
        let g = duel.select_matching_card(tp, CardId::is_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
        let Some(tc) = g.get_first() else { return Ok(()) };
        duel.hint_selection_full(tc, true);
        let atk = tc.get_attack(duel) / 2;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_value(duel, atk);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn qpovtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, 1 - a.tp, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn rthfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn qpovop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(re) = a.re else { return Ok(()) };
    let rc = re.get_handler(duel);
    if c.is_relate_to_effect(duel, e)
        && rc.is_relate_to_effect(duel, re)
        && !c.is_immune_to_effect(duel, e)
        && !rc.is_immune_to_effect(duel, e)
        && rc.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT)
    {
        duel.overlay(c, rc)?;
        if !c.get_overlay_group(duel).contains(rc) {
            return Ok(());
        }
        rc.cancel_to_grave(duel);
        if duel.is_existing_matching_card(rthfilter, tp, 0, LOCATION_ONFIELD, 1, None) && duel.select_yes_no(tp, aux::stringid(ID, 3))? {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
            let g = duel.select_matching_card(tp, rthfilter, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
            if g.is_empty() {
                return Ok(());
            }
            duel.hint_selection_full(g, true);
            duel.break_effect()?;
            duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        }
    }
    Ok(())
}
