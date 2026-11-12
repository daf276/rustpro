// Sacred Tree Beast, Hyperyton
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 9349094;

// Twice in the Extra Deck, Xyz Summoned with Level 9 monsters, with Spells in the Graveyard and hand to attach and chain to, and the
// opponent's Spells to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (43363035, MZONE),
    (48770333, MZONE),
    (12197223, MZONE),
    (55144522, HAND),
    (83764718, HAND),
    (12580477, GRAVE),
    (53129443, GRAVE),
    (12580477, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(9);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_LEAVE_GRAVE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_condition(duel, attcon);
    e1.set_target(duel, atttg);
    e1.set_operation(duel, attop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, negcon);
    e2.set_target(duel, negtg);
    e2.set_operation(duel, negop);
    c.register_effect(duel, e2);
    aux::double_snare_validity(duel, c, LOCATION_MZONE);
}

fn active_type(duel: &Duel, re: EffectId) -> u32 {
    re.get_active_type(duel) & (TYPE_SPELL | TYPE_MONSTER | TYPE_TRAP)
}

fn attcon(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(a.tp) && a.rp == a.tp && !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED)
}

fn atttg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let Some(re) = a.re else { return Ok(false) };
    let rt = active_type(duel, re);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && chkc.is_type(duel, rt));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| c.is_type(duel, rt), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_target(tp, |c, duel| c.is_type(duel, rt), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, g, 1, 0, 0);
    Ok(false)
}

fn attop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        let g = duel.get_target_cards(a.e).unwrap_or_default();
        if !g.is_empty() {
            duel.overlay(c, g)?;
        }
    }
    Ok(())
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(1 - a.tp) && duel.is_chain_negatable(a.ev as u8) && !a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let Some(re) = a.re else { return Ok(false) };
    if chk == 0 {
        let rt = active_type(duel, re);
        return Ok(a
            .e
            .get_handler(duel)
            .get_overlay_group(duel)
            .is_exists(duel, |c, duel| c.is_type(duel, rt), 1, None));
    }
    duel.set_operation_info(0, CATEGORY_NEGATE, a.eg, 1, 0, 0);
    let rc = re.get_handler(duel);
    if rc.is_destructable(duel) && rc.is_relate_to_effect(duel, re) {
        duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, 0, 0);
    }
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(re) = a.re else { return Ok(()) };
    let c = e.get_handler(duel);
    let rt = active_type(duel, re);
    let ov = c.get_overlay_group(duel).filter(duel, |c, duel| c.is_type(duel, rt), None);
    if ov.is_empty() {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_XMATERIAL);
    let sg = ov.select(duel, tp, 1, 1, None)?;
    if !sg.is_empty() && duel.send_to_grave(sg, REASON_EFFECT)? > 0 {
        duel.raise_single_event(c, EVENT_DETACH_MATERIAL, Some(e), 0, 0, 0, 0)?;
        if duel.negate_activation(a.ev as u8) && re.get_handler(duel).is_relate_to_effect(duel, re) {
            duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
        }
    }
    Ok(())
}
