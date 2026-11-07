// Xyz Encore
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 78610936;

// Tornado Dragons with Xyz materials on the opponent's field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (6983839, MZONE | OPPONENT), (6983839, MZONE | OPPONENT)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TODECK + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_XYZ) && c.is_able_to_extra(duel) && c.get_overlay_count(duel) > 0
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, filter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, g, 1, 0, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, PLAYER_ALL, LOCATION_GRAVE as i32);
    if e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        duel.set_chain_limit(|_, _, _, _, _| false, 0);
    }
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_location(duel, LOCATION_GRAVE) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, 1 - tp, 0xff)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let mg = tc.get_overlay_group(duel);
    duel.send_to_grave(mg, REASON_EFFECT)?;
    duel.adjust_instantly(Some(tc));
    if duel.send_to_deck(tc, PLAYER_NONE, SEQ_DECKTOP, REASON_EFFECT)? > 0 {
        let mut g = mg.filter(duel, aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp)), None);
        let mut ft = duel.get_location_count(1 - tp, LOCATION_MZONE);
        if ft > 0 && !g.is_empty() {
            if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
                ft = 1;
            }
            if g.len() as i32 > ft {
                duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
                g = g.select(duel, tp, ft as u8, ft as u8, None)?;
            }
            for tc in g {
                duel.special_summon_step(tc, 0, tp, 1 - tp, false, false, POS_FACEUP_DEFENSE)?;
                if tc.has_level(duel) && tc.is_level_above(duel, 2) {
                    let e1 = duel.create_effect(e.get_handler(duel));
                    e1.set_type(duel, EFFECT_TYPE_SINGLE);
                    e1.set_code(duel, EFFECT_UPDATE_LEVEL);
                    e1.set_value(duel, -1);
                    e1.set_reset(duel, RESET_EVENT + RESETS_STANDARD);
                    tc.register_effect(duel, e1);
                }
            }
            duel.special_summon_complete()?;
        }
    }
    Ok(())
}
