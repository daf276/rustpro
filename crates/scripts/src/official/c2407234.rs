// Number 69: Heraldry Crest
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 2407234;
pub const XYZ_NUMBER: u32 = 69;

// In the Extra Deck, Xyz Summoned with three Level 4 monsters, with Xyz Monsters to negate and copy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (2118022, MZONE),
    (2483611, MZONE),
    (4042268, MZONE),
    (14152862, MZONE),
    (67557908, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_operation(duel, negop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn negfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_XYZ) && !c.is_disabled(duel)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let g = duel.get_matching_group(negfilter, a.tp, LOCATION_MZONE, LOCATION_MZONE, Some(c.into()));
    for tc in g {
        for code in [EFFECT_DISABLE, EFFECT_DISABLE_EFFECT] {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, code);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect(duel, e1);
        }
    }
    Ok(())
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_XYZ)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && filter(chkc, duel) && chkc != c);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, filter, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, Some(c.into()))?;
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if let Some(tc) = duel.get_first_target()
        && c.is_relate_to_effect(duel, e)
        && c.is_faceup(duel)
        && tc.is_faceup(duel)
        && tc.is_relate_to_effect(duel, e)
    {
        let code = tc.get_original_code(duel);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_CHANGE_CODE);
        e1.set_value(duel, code as i32);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e1);
        c.copy_effect(duel, code, RESETS_STANDARD_PHASE_END, 1);
    }
    Ok(())
}
