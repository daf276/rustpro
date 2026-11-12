// Ice Beast Zerofyne
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE, OPPONENT};

const ID: u32 = 86848580;

// In the Extra Deck, Xyz Summoned with Level 4 Winged Beast Normal Monsters (Takuhee, Skull Red Bird), with the opponent's monster to negate.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (3170832, MZONE),
    (3170832, MZONE),
    (10202894, MZONE),
    (10202894, MZONE),
    (26202165, MZONE | OPPONENT),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_race_full(duel, RACE_WINGEDBEAST, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, distg);
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_negatable, a.tp, 0, LOCATION_ONFIELD, 1, None));
    }
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, c) = (a.e, a.tp, a.e.get_handler(duel));
    let g = duel.get_matching_group(CardId::is_negatable, tp, 0, LOCATION_ONFIELD, None);
    if g.get_first().is_none() {
        return Ok(());
    }
    for tc in g {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_DISABLE);
        e1.set_reset_full(duel, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_STANDBY, 2);
        tc.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_DISABLE_EFFECT);
        e2.set_reset_full(duel, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_STANDBY, 2);
        tc.register_effect(duel, e2);
        if tc.is_type(duel, TYPE_TRAPMONSTER) {
            let e3 = duel.create_effect(c);
            e3.set_type(duel, EFFECT_TYPE_SINGLE);
            e3.set_code(duel, EFFECT_DISABLE_TRAPMONSTER);
            e3.set_reset_full(duel, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_STANDBY, 2);
            tc.register_effect(duel, e3);
        }
    }
    if !c.is_relate_to_effect(duel, e) || c.is_facedown(duel) {
        return Ok(());
    }
    let atk = duel.get_matching_group_count(CardId::is_faceup, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, Some(c.into())) * 300;
    if atk > 0 {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, atk as i32);
        e1.set_reset_full(duel, RESET_EVENT | RESETS_STANDARD_DISABLE | RESET_PHASE | PHASE_STANDBY, 2);
        c.register_effect(duel, e1);
    }
    Ok(())
}
