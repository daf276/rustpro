// Destiny HERO - Dogma
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 17132130;

pub const LISTED_SERIES: &[u16] = &[SET_DESTINY_HERO];
// In the hand, Special Summoned by Tributing Destiny HERO - Malicious, Denier and Sangan.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (9411399, MZONE), (16605586, MZONE), (26202165, MZONE)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    c.add_must_be_special_summoned(duel);
    let e0 = duel.create_effect(c);
    e0.set_description(duel, aux::stringid(ID, 0));
    e0.set_type(duel, EFFECT_TYPE_FIELD);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_SPSUMMON_PROC);
    e0.set_range(duel, LOCATION_HAND);
    e0.set_condition(duel, spcon);
    e0.set_target(duel, sptg);
    e0.set_operation(duel, spop);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_PHASE + PHASE_STANDBY);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, |duel, a| {
        duel.is_turn_player(1 - a.tp) && a.e.get_handler(duel).has_flag_effect(duel, ID)
    });
    e1.set_operation(duel, |duel, a| {
        let opp = 1 - a.tp;
        let lp = (f64::from(duel.get_lp(opp)) / 2.0).round() as i32;
        duel.set_lp(opp, lp);
        Ok(())
    });
    c.register_effect(duel, e1);
}

fn rescon(duel: &Duel, sg: CardSet, tp: u8) -> (bool, bool) {
    (
        duel.get_mzone_count(tp, Some(sg)) > 0 && sg.is_exists(duel, |c, duel| c.is_set_card(duel, &[SET_DESTINY_HERO]), 1, None),
        false,
    )
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    if a.c.is_none() {
        return true;
    }
    let tp = a.e.get_handler_player(duel);
    let g = duel.get_matching_group(CardId::is_releasable, tp, LOCATION_MZONE, 0, None);
    g.len() >= 3 && aux::select_unselect_group_check(duel, g, 3, 3, &|duel, sg, _, _| rescon(duel, sg, tp))
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(CardId::is_releasable, tp, LOCATION_MZONE, 0, None);
    let rescon = |duel: &Duel, sg, _, _| rescon(duel, sg, tp);
    let sg = aux::select_unselect_group_full(duel, g, 3, 3, &rescon, tp, HINTMSG_RELEASE, None, true)?;
    if sg.is_empty() {
        return Ok(false);
    }
    a.e.set_label_object_group(duel, Some(sg));
    Ok(true)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let sg = a.e.get_label_object_group(duel).unwrap_or_default();
    if let Some(c) = a.c
        && sg.len() == 3
    {
        duel.release(sg, REASON_COST)?;
        c.register_flag_effect(
            duel,
            ID,
            RESET_EVENT | (RESETS_STANDARD & !(RESET_TOFIELD | RESET_TEMP_REMOVE | RESET_LEAVE)) | RESET_PHASE | PHASE_STANDBY,
            0,
            1,
        );
    }
    Ok(())
}
