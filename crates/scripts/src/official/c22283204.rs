// The Gaze of Timaeus
use super::c1784686::spfilter;
use crate::prelude::*;
use crate::testing::{EXTRA, GRAVE, MZONE};

const ID: u32 = 22283204;
pub const LISTED_NAMES: &[u32] = &[CARD_DARK_MAGICIAN, CARD_DARK_MAGICIAN_GIRL, 1784686];

// Dark Magician, and Fusion Monsters that list it (Red-Eyes Dark Dragoon, Dark Cavalry, Dark Magician of Destruction).
pub const SUPPORT: &[(u32, u16)] = &[
    (CARD_DARK_MAGICIAN, MZONE),
    (CARD_DARK_MAGICIAN, GRAVE),
    (37818794, EXTRA),
    (73452089, EXTRA),
    (59400890, EXTRA),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TODECK + CATEGORY_SPECIAL_SUMMON + CATEGORY_FUSION_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn tdfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    (c.is_code(duel, CARD_DARK_MAGICIAN) || c.is_code(duel, CARD_DARK_MAGICIAN_GIRL))
        && c.is_faceup(duel)
        && c.is_can_be_fusion_material(duel, None, 0)
        && c.is_able_to_deck(duel)
        && duel.is_existing_matching_card(|sc, duel| spfilter(sc, duel, e, tp, c), tp, LOCATION_EXTRA, 0, 1, None)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let locations = LOCATION_MZONE | LOCATION_GRAVE;
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| tdfilter(c, duel, e, tp), tp, locations, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FMATERIAL);
    let g = duel.select_target(tp, |c, duel| tdfilter(c, duel, e, tp), tp, locations, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, g, 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

struct Banish;

impl aux::DelayedOperation for Banish {
    fn operate(duel: &mut Duel, ag: CardSet, _: &Args) -> Script<()> {
        duel.remove(ag, POS_FACEUP, REASON_EFFECT)?;
        Ok(())
    }

    fn condition(duel: &Duel, _: CardSet, a: &Args) -> bool {
        i64::from(duel.get_turn_count()) == duel.get_script_state(turn_summoned_key(a.e.get_label(duel))) + 1
    }
}

// The turn the Fusion Monster was summoned in, per delayed effect (its first label is the fid).
fn turn_summoned_key(fid: i64) -> u64 {
    u64::from(ID) << 32 | fid as u64
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && tc.is_can_be_fusion_material(duel, None, 0)
        && !tc.is_immune_to_effect(duel, e)
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, tc), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
        if let Some(sc) = g.get_first() {
            if tc.is_facedown(duel) {
                duel.confirm_cards(1 - tp, tc)?;
            }
            sc.set_material(duel, Some(tc.into()));
            duel.send_to_deck(tc, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT | REASON_MATERIAL | REASON_FUSION)?;
            duel.break_effect()?;
            if duel.special_summon(sc, SUMMON_TYPE_FUSION, tp, tp, false, false, POS_FACEUP)? > 0 {
                sc.complete_procedure(duel);
                let turn = i64::from(duel.get_turn_count());
                let reset = RESET_PHASE | PHASE_END;
                if let Some(delayed) =
                    aux::delayed_operation_full::<Banish>(duel, sc.into(), PHASE_END, reset, ID, e, tp, aux::stringid(ID, 1), 2)
                {
                    let key = turn_summoned_key(delayed.get_label(duel));
                    duel.set_script_state(key, turn);
                }
            }
        }
    }
    Ok(())
}
