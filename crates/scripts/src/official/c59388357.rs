// Circle of the Fire Kings
use crate::prelude::*;
use crate::testing::{GRAVE, HAND, MZONE};

const ID: u32 = 59388357;

// Copies in hand, Fire King Courtier Ulcanix on the field to destroy, and FIRE monsters in the Graveyard to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (44455560, MZONE),
    (44455560, MZONE),
    (44455560, GRAVE),
    (90681088, GRAVE),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn desfilter(c: CardId, duel: &Duel, ft: i32) -> bool {
    c.is_faceup(duel) && c.is_attribute(duel, ATTRIBUTE_FIRE) && (ft > 0 || c.get_sequence(duel) < 5)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_attribute(duel, ATTRIBUTE_FIRE) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chkc.is_some() {
        return Ok(false);
    }
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    if chk == 0 {
        return Ok(ft > -1
            && duel.is_existing_target(|c, duel| desfilter(c, duel, ft), tp, LOCATION_MZONE, 0, 1, None)
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g1 = duel.select_target(tp, |c, duel| desfilter(c, duel, ft), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g2 = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g1, 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g2, 1, 0, 0);
    e.set_label_object_group(duel, g1.and_then(|g| g.get_first()).map(Into::into));
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let targets = duel
        .get_chain_info::<Option<CardSet>>(0, CHAININFO_TARGET_CARDS)
        .unwrap_or_default();
    let label = e.get_label_object_group(duel).and_then(|g| g.get_first());
    let (Some(tc1), Some(tc2)) = (label, label.and_then(|tc1| targets.difference(tc1.into()).get_first())) else {
        return Ok(());
    };
    if tc1.is_controler(duel, tp)
        && tc1.is_relate_to_effect(duel, e)
        && duel.destroy(tc1, REASON_EFFECT)? > 0
        && tc2.is_relate_to_effect(duel, e)
    {
        duel.special_summon(tc2, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
