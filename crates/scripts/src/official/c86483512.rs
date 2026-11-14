// Virtual World Roshi - Laolao
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, SZONE};

use super::c49088914::{cannot_special_summon, tgfilter, togravefilter};

const ID: u32 = 86483512;

pub const LISTED_SERIES: &[u16] = &[SET_VIRTUAL_WORLD];
// In the hand, with Virtual World Kyubi - Shenshen (a monster) and Virtual World Gate - Qinglong (a Spell) on the field as the
// target, Qinglong in the Deck to send, and Virtual World Mai-Hime - Lulu in the Graveyard to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (92519087, MZONE),
    (50275295, SZONE),
    (50275295, DECK),
    (50275295, DECK),
    (49088914, GRAVE),
    (49088914, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::SpecialSummonedCode(49088914)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_SPECIAL_SUMMON);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let filter = |c: CardId, duel: &Duel| tgfilter(c, duel, tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_ONFIELD) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_ONFIELD, 0, 1, None)
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, filter, tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn ssummonfilter(c: CardId, duel: &Duel, tgid: u32, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_VIRTUAL_WORLD])
        && c.is_monster(duel)
        && !c.is_code(duel, tgid)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let tc = duel.get_first_target();
    cannot_special_summon(duel, c, tp, ID);
    let Some(tc) = tc.filter(|tc| tc.is_relate_to_effect(duel, e)) else {
        return Ok(());
    };
    let ctype = tc.get_type(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, |c, duel| togravefilter(c, duel, ctype), tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() && duel.send_to_grave(g, REASON_EFFECT)? > 0 {
        let ogc = duel.get_operated_group().get_first();
        if ogc.is_some_and(|ogc| ogc.is_location(duel, LOCATION_GRAVE))
            && c.is_relate_to_effect(duel, e)
            && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? > 0
        {
            let gcode = g.get_first().map_or(0, |gc| gc.get_code(duel));
            let gss = duel.get_matching_group(
                aux::necro_valley_filter(|c, duel| ssummonfilter(c, duel, gcode, e, tp)),
                tp,
                LOCATION_GRAVE,
                0,
                None,
            );
            if !gss.is_empty() && duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
                duel.break_effect()?;
                duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
                let sg = gss.select(duel, tp, 1, 1, None)?;
                if let Some(sc) = sg.get_first()
                    && duel.special_summon_step(sc, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?
                {
                    for code in [EFFECT_DISABLE, EFFECT_DISABLE_EFFECT] {
                        let e2 = duel.create_effect(c);
                        e2.set_type(duel, EFFECT_TYPE_SINGLE);
                        e2.set_code(duel, code);
                        e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
                        sc.register_effect_full(duel, e2, true);
                    }
                    duel.special_summon_complete()?;
                }
            }
        }
    }
    Ok(())
}
