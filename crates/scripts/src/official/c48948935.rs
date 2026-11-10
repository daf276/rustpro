// Masked Beast Des Gardius
use super::c38423248::equip_limit;
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 48948935;
const GRAND_TIKI_ELDER: u32 = 13676474;
const MELCHID: u32 = 86569121;
const MASK_OF_REMNANTS: u32 = 22610082;

pub const LISTED_NAMES: &[u32] = &[MASK_OF_REMNANTS];
// Grand Tiki Elder and Melchid, Normal Monsters, to release with a monster of ours; The Mask of Remnants in the Deck to equip
// after the opponent's Raigeki sends this card to the Graveyard, with a monster on their side as the target.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (GRAND_TIKI_ELDER, MZONE),
    (MELCHID, MZONE),
    (MASK_OF_REMNANTS, DECK),
    (MASK_OF_REMNANTS, DECK),
    (12580477, HAND | OPPONENT),
    (GRAND_TIKI_ELDER, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_EQUIP);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_ONFIELD));
    e2.set_target(duel, eqtg);
    e2.set_operation(duel, eqop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    (c.is_code(duel, GRAND_TIKI_ELDER) || c.is_code(duel, MELCHID)) && (c.is_controler(duel, tp) || c.is_faceup(duel))
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    if a.c.is_none() {
        return true;
    }
    let tp = a.e.get_handler_player(duel);
    let rg1 = duel.get_release_group(tp);
    let rg2 = duel.get_matching_group(|c, duel| spfilter(c, duel, tp), tp, LOCATION_MZONE, 0, None);
    aux::select_unselect_group_check(duel, rg1, 2, 2, &aux::chkf_mmz(tp, 1))
        && aux::select_unselect_group_check(duel, rg2, 1, 1, &aux::chkf_mmz(tp, 1))
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let rg1 = duel.get_matching_group(|c, duel| spfilter(c, duel, tp), tp, LOCATION_MZONE, 0, None);
    let mut mg1 = aux::select_unselect_group_full(duel, rg1, 1, 1, &aux::chkf_mmz(tp, 1), tp, HINTMSG_RELEASE, None, true)?;
    if let Some(sg) = mg1.get_first() {
        let mut rg2 = duel.get_release_group(tp);
        rg2.remove(sg);
        let mg2 = aux::select_unselect_group_full(duel, rg2, 1, 1, &aux::chkf_mmz(tp, 1), tp, HINTMSG_RELEASE, None, true)?;
        mg1.merge(mg2);
    }
    if mg1.len() == 2 {
        e.set_label_object_group(duel, Some(mg1));
        return Ok(true);
    }
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(g) = a.e.get_label_object_group(duel) {
        duel.release(g, REASON_COST)?;
    }
    Ok(())
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_faceup(duel) && chkc.is_controler(duel, 1 - tp));
    }
    if chk == 0 {
        return Ok(true);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, CardId::is_faceup, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_EQUIP, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, MASK_OF_REMNANTS) && !c.is_forbidden(duel)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_SZONE) <= 0 {
        return Ok(());
    }
    if let Some(tc) = duel.get_first_target()
        && tc.is_faceup(duel)
        && tc.is_relate_to_effect(duel, e)
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
        let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        let Some(eqc) = g.get_first() else { return Ok(()) };
        if !duel.equip(tp, eqc, tc)? {
            return Ok(());
        }
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_EQUIP_LIMIT);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        e1.set_value(duel, Value::Card(equip_limit));
        e1.set_label_object_group(duel, Some(tc.into()));
        eqc.register_effect(duel, e1);
        let e2 = duel.create_effect(eqc);
        e2.set_type(duel, EFFECT_TYPE_EQUIP);
        e2.set_code(duel, EFFECT_SET_CONTROL);
        e2.set_value(duel, i32::from(tp));
        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        eqc.register_effect(duel, e2);
    }
    Ok(())
}
