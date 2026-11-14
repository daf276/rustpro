// Gladiator Beast's Comeback
use crate::prelude::*;
use crate::testing::{GRAVE, HAND};

const ID: u32 = 20201255;

pub const LISTED_SERIES: &[u16] = &[SET_GLADIATOR];
// In the hand, with Gladiator Beast Hoplomus and Andal (different races) in the hand and GY.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (4253484, HAND), (4253484, GRAVE), (90582719, GRAVE), (90582719, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    let race = c.get_race(duel);
    c.is_set_card(duel, &[SET_GLADIATOR])
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && !duel.is_existing_matching_card(
            |mc, duel| mc.is_faceup(duel) && mc.is_race(duel, race),
            tp,
            LOCATION_MZONE,
            0,
            1,
            None,
        )
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, e, tp), LOCATION_HAND | LOCATION_GRAVE)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first()
        && duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)? != 0
    {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_description(duel, 3000);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
