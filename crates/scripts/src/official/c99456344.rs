// Dramaturge of Despia
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 99456344;

// A copy on the field to negate, and one in the hand to Fusion Summon Despian Quaeritis by Polymerization with Mystical Elf.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, HAND),
    (72272462, EXTRA),
    (72272462, EXTRA),
    (CARD_POLYMERIZATION, HAND),
    (15025844, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.eg.is_some_and(|eg| eg.is_exists(duel, cfilter, 1, None)));
    e1.set_target(duel, distg);
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_BE_MATERIAL);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, super::c81555617::spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_FUSION | TYPE_SYNCHRO | TYPE_XYZ | TYPE_LINK) && c.is_faceup(duel)
}

fn disfilter(c: CardId, duel: &Duel) -> bool {
    c.is_type(duel, TYPE_EFFECT) && c.is_faceup(duel) && !c.is_disabled(duel)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && disfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(disfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let g = duel.select_target(tp, disfilter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    let owner = g.and_then(|g| g.get_first()).map_or(PLAYER_NONE, |tc| tc.get_owner(duel));
    duel.set_operation_info(0, CATEGORY_DISABLE, g, 1, owner, LOCATION_MZONE as i32);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) && tc.is_faceup(duel) && !tc.is_disabled(duel) {
        duel.negate_related_chain(tc, RESET_TURN_SET);
        super::c97268402::negate(duel, c, tc, EFFECT_FLAG_CANNOT_DISABLE);
    }
    Ok(())
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    let location = c.get_location(duel);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, location as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if duel.get_location_count(tp, LOCATION_MZONE) == 0 || !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    Ok(())
}
