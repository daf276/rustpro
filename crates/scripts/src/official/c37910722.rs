// Ascension Sky Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 37910722;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// In the Extra Deck, Synchro Summoned with Junk Changer (Level 3 Tuner) and Gaia The Fierce Knight (Level 7), with the opponent's
// Elfnote Lucina and Raigeki to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (1006081, MZONE),
    (1006081, MZONE),
    (6368038, MZONE),
    (6368038, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, e) {
        let ct = duel.get_field_group_count(tp, LOCATION_HAND, 0) as i32;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, ct * 800);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    a.rp != a.tp
        && c.is_reason(duel, REASON_DESTROY)
        && c.is_previous_location(duel, LOCATION_ONFIELD)
        && c.is_previous_controler(duel, a.tp)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, sync: CardId) -> bool {
    c.is_controler(duel, tp)
        && c.is_location(duel, LOCATION_GRAVE)
        && c.get_reason(duel) & (REASON_SYNCHRO | REASON_MATERIAL) == REASON_SYNCHRO | REASON_MATERIAL
        && c.get_reason_card(duel) == Some(sync)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let mg = c.get_material(duel);
    let ct = mg.len();
    if chk == 0 {
        return Ok(c.is_synchro_summoned(duel)
            && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && ct > 0
            && duel.get_location_count(tp, LOCATION_MZONE) >= ct as i32
            && mg.filter_count(duel, |mc, duel| spfilter(mc, duel, e, tp, c), None) == ct);
    }
    duel.set_target_card(mg);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(mg), ct as u8, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        return Ok(());
    }
    let c = e.get_handler(duel);
    let mg: CardSet = duel
        .get_chain_info::<Option<CardSet>>(0, CHAININFO_TARGET_CARDS)
        .unwrap_or_default();
    let g = mg.filter(duel, |tc, duel| tc.is_relate_to_effect(duel, e), None);
    if g.len() < mg.len() || duel.get_location_count(tp, LOCATION_MZONE) < g.len() as i32 {
        return Ok(());
    }
    for tc in g {
        duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)?;
        for code in [EFFECT_DISABLE, EFFECT_DISABLE_EFFECT] {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, code);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect(duel, e1);
        }
    }
    duel.special_summon_complete()?;
    Ok(())
}
