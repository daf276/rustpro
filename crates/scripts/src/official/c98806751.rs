// Obsessive Uvualoop
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 98806751;

// In the hand and Graveyard, with Synchro Monsters (Armory Arm, Toxic Boxfish) in the Graveyard and on the field to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (29071332, GRAVE),
    (29071332, GRAVE),
    (48882106, MZONE),
    (48882106, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_type(duel, TYPE_SYNCHRO)
        && c.is_able_to_remove(duel)
        && aux::sp_elim_filter(c, duel, true, true)
        && duel.get_mzone_count(tp, Some(c.into())) > 0
        && c.is_faceup(duel)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE | LOCATION_GRAVE) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, tp));
    }
    if chk == 0 {
        return Ok(
            duel.is_existing_target(|c, duel| spfilter(c, duel, tp), tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, None)
                && c.is_can_be_special_summoned(duel, e, 0, tp, false, false),
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(
        tp,
        |c, duel| spfilter(c, duel, tp),
        tp,
        LOCATION_MZONE | LOCATION_GRAVE,
        0,
        1,
        1,
        None,
    )?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let Some(tc) = duel.get_first_target() else {
        return Ok(());
    };
    if tc.is_relate_to_effect(duel, a.e)
        && duel.remove(tc, POS_FACEUP, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_REMOVED)
        && c.is_relate_to_effect(duel, a.e)
    {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_SYNCHRO) && c.is_able_to_remove(duel) && aux::sp_elim_filter(c, duel, true, true) && c.is_faceup(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE | LOCATION_GRAVE) && chkc.is_controler(duel, tp) && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel) && duel.is_existing_target(thfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, 0, LOCATION_GRAVE as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else {
        return Ok(());
    };
    if tc.is_relate_to_effect(duel, a.e)
        && duel.remove(tc, POS_FACEUP, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_REMOVED)
        && c.is_relate_to_effect(duel, a.e)
    {
        duel.send_to_hand(c, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
