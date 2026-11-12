// Celestial Double Star Shaman
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 25472513;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 1;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, |_, _| true)
    }
}

// In the Extra Deck, Synchro Summoned with Water Spirit and Key Mace (Level 1 Normal Monsters), and Doron (Level 2) in the hand and Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (487395, MZONE),
    (487395, MZONE),
    (1929294, MZONE),
    (1929294, MZONE),
    (756652, HAND),
    (756652, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_synchro_summoned(duel)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.get_level(duel) == 2
        && !c.is_type(duel, TYPE_TUNER)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_HAND | LOCATION_GRAVE)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE).min(4);
    if ft > 1 && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
        ft = 1;
    }
    if ft <= 0 {
        return Ok(());
    }
    let c = e.get_handler(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, ft as u8, None)?;
    if !g.is_empty() {
        for tc in g {
            duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_DISABLE);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect(duel, e1);
            let e2 = duel.create_effect(c);
            e2.set_type(duel, EFFECT_TYPE_SINGLE);
            e2.set_code(duel, EFFECT_DISABLE_EFFECT);
            e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect(duel, e2);
        }
        duel.special_summon_complete()?;
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| {
        !c.is_type(duel, TYPE_SYNCHRO) && c.is_location(duel, LOCATION_EXTRA)
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| !c.is_original_type(duel, TYPE_SYNCHRO));
    Ok(())
}
