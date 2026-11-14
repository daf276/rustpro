// T.G. Recipro Dragonfly
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 62560742;

pub const LISTED_SERIES: &[u16] = &[SET_TG];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 1;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// In the Extra Deck, Synchro Summoned with Water Spirit (a Level 1 Tuner) and Key Mace (Level 1), with T.G. Mighty Striker on the
// field to send to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (487395, MZONE),
    (487395, MZONE),
    (1929294, MZONE),
    (1929294, MZONE),
    (32480825, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_TG]) && c.is_type(duel, TYPE_SYNCHRO)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_MZONE, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 2));
    let g = duel.select_target(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, Some(c.into()))?;
    duel.set_operation_info(0, CATEGORY_TOGRAVE, g, 1, 0, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn mgfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, sync: CardId) -> bool {
    c.is_controler(duel, tp)
        && c.is_location(duel, LOCATION_GRAVE)
        && c.get_reason(duel) & (REASON_MATERIAL + REASON_SYNCHRO) == REASON_MATERIAL + REASON_SYNCHRO
        && c.get_reason_card(duel) == Some(sync)
        && c.is_type(duel, TYPE_SYNCHRO)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let mg = tc.get_material(duel);
    let ct = mg.len();
    let sumtype = tc.get_summon_type(duel);
    if duel.send_to_grave(tc, REASON_EFFECT)? != 0
        && sumtype == SUMMON_TYPE_SYNCHRO
        && ct > 0
        && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
        && ct as i32 <= duel.get_location_count(tp, LOCATION_MZONE)
        && mg.filter_count(duel, aux::necro_valley_filter(|c, duel| mgfilter(c, duel, e, tp, tc)), None) == ct
        && duel.select_yes_no(tp, aux::stringid(ID, 1))?
    {
        duel.break_effect()?;
        duel.special_summon(mg, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
