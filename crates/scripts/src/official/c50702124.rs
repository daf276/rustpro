// Synchro Magnet
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 50702124;

// In hand while Fleur Synchron and Kojikocy (Level 4 Normal Monster) on the field Synchro Summon Iron Chain Dragon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (19642774, MZONE),
    (19642774, MZONE),
    (1184620, MZONE),
    (1184620, MZONE),
    (19974580, EXTRA),
    (19974580, EXTRA),
    (19974580, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let eg = a.eg.unwrap_or_default();
    eg.len() == 1
        && eg
            .get_first()
            .is_some_and(|tc| tc.is_controler(duel, a.tp) && tc.is_synchro_summoned(duel))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, true, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if !c.is_relate_to_effect(duel, a.e) {
        return Ok(());
    }
    if duel.special_summon(c, 0, tp, tp, true, false, POS_FACEUP)? != 0 {
        c.complete_procedure(duel);
    }
    Ok(())
}
