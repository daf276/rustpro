// DoomZ XII Zero - Drastea
use super::c54265980::add_xyz_effect;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE};

const ID: u32 = 92472273;

// In the hand, with DoomZ VII Seven - Elara on the field to destroy, Axe of Despair in the Deck to equip, and DoomZ XII End -
// Drastrius in the Extra Deck to attach it to.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (54265980, MZONE),
    (28877382, MZONE),
    (40619825, DECK),
    (40619825, DECK),
    (95626382, EXTRA),
    (95626382, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON + CATEGORY_EQUIP);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    add_xyz_effect(duel, c, ID);
}

fn desfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_DOOMZ]) && c.is_faceup(duel) && duel.get_mzone_count(tp, Some(c.into())) > 0
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_on_field(duel) && desfilter(chkc, duel, tp));
    }
    let filter = |c, duel: &Duel| desfilter(c, duel, tp);
    if chk == 0 {
        return Ok(
            duel.is_existing_target(filter, tp, LOCATION_ONFIELD, 0, 1, None) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, filter, tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_EQUIP, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn eqsfilter(c: CardId, duel: &Duel, tp: u8, ec: CardId) -> bool {
    c.is_equip_spell(duel) && c.check_equip_target(duel, ec) && c.check_unique_on_field(duel, tp)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else {
        return Ok(());
    };
    if tc.is_relate_to_effect(duel, e)
        && duel.destroy(tc, REASON_EFFECT)? > 0
        && c.is_relate_to_effect(duel, e)
        && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0
        && duel.get_location_count(tp, LOCATION_SZONE) > 0
        && duel.is_existing_matching_card(|sc, duel| eqsfilter(sc, duel, tp, c), tp, LOCATION_DECK, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
        let g = duel.select_matching_card(tp, |sc, duel| eqsfilter(sc, duel, tp, c), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if let Some(eqc) = g.get_first() {
            duel.break_effect()?;
            duel.equip(tp, eqc, c)?;
        }
    }
    Ok(())
}
