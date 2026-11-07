// Clown Crew Matinee Operatics
use super::c83232904::{register_activation_limit, spfilter};
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, SZONE};

const ID: u32 = 57847269;

// In the hand and Spell & Trap Zone, with Clown Crew monsters in the Deck and Extra Deck to Special Summon, and Clown Crew
// Biancaviso in the hand to Tribute Summon with the Ritual Monster Clown Crew Flair, for the End Phase draw.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (82159583, DECK),
    (31533473, EXTRA),
    (82159583, HAND),
    (42759961, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummonedCode(82159583),
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_SZONE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DRAW + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EVENT_PHASE + PHASE_END);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, drtg);
    e2.set_operation(duel, drop);
    c.register_effect(duel, e2);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_RELEASE);
        ge1.set_operation(duel, checkop);
        duel.register_effect(ge1, 0);
    });
}

const TRIBUTED_TYPES: u32 = TYPE_RITUAL | TYPE_PENDULUM | TYPE_EXTRA;

fn checkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g =
        a.eg.unwrap_or_default()
            .filter(duel, |c, duel| c.is_type(duel, TRIBUTED_TYPES), None);
    for rc in g {
        let card_type = i64::from(rc.get_original_type(duel) & TRIBUTED_TYPES);
        let current_label = duel.get_flag_effect_label(0, ID).unwrap_or(0);
        let new_label = current_label | card_type;
        if new_label != current_label {
            duel.register_flag_effect(0, ID, RESET_PHASE | PHASE_END, 0, 1);
            duel.set_flag_effect_label(0, ID, new_label as i32);
        }
    }
    Ok(())
}

fn tributed_type_count(duel: &Duel) -> u32 {
    (duel.get_flag_effect_label(0, ID).unwrap_or(0) as u32).count_ones()
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let mmz_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
        return Ok(duel.is_existing_matching_card(
            |c, duel| spfilter(c, duel, e, tp, mmz_chk),
            tp,
            LOCATION_DECK | LOCATION_EXTRA,
            0,
            1,
            None,
        ));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_DECK | LOCATION_EXTRA) as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let mmz_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
    let g = duel.select_matching_card(
        tp,
        |c, duel| spfilter(c, duel, e, tp, mmz_chk),
        tp,
        LOCATION_DECK | LOCATION_EXTRA,
        0,
        1,
        1,
        None,
    )?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, true, false, POS_FACEUP)?;
    }
    register_activation_limit(duel, e, tp, aux::stringid(ID, 2));
    Ok(())
}

fn drtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let ct = tributed_type_count(duel);
    if chk == 0 {
        return Ok(ct > 0 && duel.is_player_can_draw(tp, ct as usize));
    }
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, ct as i32);
    Ok(false)
}

fn drop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let ct = tributed_type_count(duel);
    if duel.draw(tp, ct, REASON_EFFECT)? > 0 {
        duel.shuffle_hand(tp)?;
        if duel.is_existing_matching_card(CardId::is_ssetable, tp, LOCATION_HAND, 0, 1, None)
            && duel.select_yes_no(tp, aux::stringid(ID, 3))?
        {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
            let g = duel.select_matching_card(tp, CardId::is_ssetable, tp, LOCATION_HAND, 0, 1, 1, None)?;
            if !g.is_empty() {
                duel.break_effect()?;
                duel.sset_full(tp, g, tp, false)?;
            }
        }
    }
    Ok(())
}
