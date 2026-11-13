// The Flute of Guiding Dragon
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 93871599;

// Copies in the hand, with a Spellcaster (Thousand-Eyes Idol) on the field, Level 8 Dragons (Rabidragon) in the Deck, and two of the
// opponent's monsters (Elfnote Lucina) for the second Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, HAND),
    (27125110, MZONE),
    (95788410, DECK),
    (95788410, DECK),
    (95788410, DECK),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, |duel, _| {
        duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_race(duel, RACE_SPELLCASTER)),
            0,
            LOCATION_MZONE,
            LOCATION_MZONE,
            1,
            None,
        )
    });
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        duel.is_main_phase2() && c.get_turn_id(duel) == duel.get_turn_count() && !c.is_reason(duel, REASON_RETURN)
    });
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, stock::draw_target::<1>);
    e2.set_operation(duel, stock::draw);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_above(duel, 5) && c.is_race(duel, RACE_DRAGON) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn samenamespfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, code: u32) -> bool {
    c.is_code(duel, code) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if let Some(sc) = g.first()
            && duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP)? > 0
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.get_field_group_count(tp, 0, LOCATION_MZONE) >= 2
        {
            let code = sc.get_code(duel);
            let samename = move |c: CardId, duel: &Duel| samenamespfilter(c, duel, e, tp, code);
            if duel.is_existing_matching_card(samename, tp, LOCATION_DECK, 0, 1, None) && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
                duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
                let g = duel.select_matching_card(tp, samename, tp, LOCATION_DECK, 0, 1, 1, None)?;
                if !g.is_empty() {
                    duel.break_effect()?;
                    duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
                }
            }
        }
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 3));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_race_except(duel, RACE_DRAGON | RACE_SPELLCASTER));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}
