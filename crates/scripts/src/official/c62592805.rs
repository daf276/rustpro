// Epurrely Noir
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, OPPONENT};

const ID: u32 = 62592805;

pub const LISTED_NAMES: &[u32] = &[21347668];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(2);
    const CT: usize = 2;
}

// On the field with Purrely Sleepy Memory and Purrely Happy Memory in the hand to activate and attach, and Sangan to bounce.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (21347668, HAND),
    (21347668, HAND),
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
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, thcost);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 3, 0);
    e2.set_condition(duel, attachcond);
    e2.set_operation(duel, attachop);
    c.register_effect(duel, e2);
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, None));
    }
    duel.discard_hand(tp, CardId::is_discardable, 1, 1, REASON_COST | REASON_DISCARD)?;
    Ok(false)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp) && chkc.is_able_to_hand(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_able_to_hand, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    let overlay = a.e.get_handler(duel).get_overlay_group(duel);
    let ct = if overlay.is_exists(duel, |c, duel| c.is_code(duel, 21347668), 1, None) {
        2
    } else {
        1
    };
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(tp, CardId::is_able_to_hand, tp, 0, LOCATION_ONFIELD, 1, ct, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, g.map_or(0, |g| g.len() as u8), 0, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}

fn attachcond(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    if a.rp == 1 - a.tp || !re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        return false;
    }
    let rc = re.get_handler(duel);
    rc.is_set_card(duel, &[SET_PURRELY])
        && rc.is_quick_play_spell(duel)
        && rc.is_on_field(duel)
        && rc.is_can_be_xyz_material(duel, Some(a.e.get_handler(duel)), a.tp, REASON_EFFECT)
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_PURRELY]) && c.is_trap(duel) && c.is_ssetable(duel)
}

fn attachop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c, tp) = (a.e, a.e.get_handler(duel), a.tp);
    let Some(re) = a.re else { return Ok(()) };
    let rc = re.get_handler(duel);
    if c.is_relate_to_effect(duel, e)
        && rc.is_relate_to_effect(duel, re)
        && !c.is_immune_to_effect(duel, e)
        && !rc.is_immune_to_effect(duel, e)
        && rc.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT)
    {
        duel.overlay(c, rc)?;
        if !c.get_overlay_group(duel).is_contains(rc) {
            return Ok(());
        }
        rc.cancel_to_grave(duel);
        let g = duel.get_matching_group(setfilter, tp, LOCATION_DECK, 0, None);
        if !g.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
            let sg = g.select(duel, tp, 1, 1, None)?;
            duel.break_effect()?;
            duel.sset(tp, sg)?;
        }
    }
    Ok(())
}
