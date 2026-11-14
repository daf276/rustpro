// Lo, the Prayers of the Voiceless Voice
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 25801745;

pub const LISTED_SERIES: &[u16] = &[SET_VOICELESS_VOICE];

struct Tribute;

impl WholeLevelTribute for Tribute {
    fn condition(rit_c: CardId, duel: &Duel, _: EffectId) -> bool {
        rit_c.is_attribute(duel, ATTRIBUTE_LIGHT) && rit_c.is_race(duel, RACE_WARRIOR | RACE_DRAGON)
    }
}

// Voiceless Voice Continuous Spells/Traps in the Deck to place, and a copy in the Graveyard with Saffira, Dragon Queen of the
// Voiceless Voice to Ritual Summon Skull Guardian, Protector of the Voiceless Voice.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (98477480, DECK),
    (39114494, DECK),
    (86310763, DECK),
    (51296484, GRAVE),
    (10774240, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::RitualMaterial,
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Ritual::add_whole_level_tribute::<Tribute>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, pltg);
    e1.set_operation(duel, plop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, spcon);
    e3.set_target(duel, stock::special_summon_self_target);
    e3.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e3);
}

fn plfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_VOICELESS_VOICE])
        && (c.is_continuous_spell(duel) || c.is_continuous_trap(duel))
        && !c.is_forbidden(duel)
        && c.check_unique_on_field(duel, tp)
}

fn pltg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_matching_card(|c, duel| plfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    Ok(false)
}

fn plop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_location_count(tp, LOCATION_SZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let tc = duel
        .select_matching_card(tp, |c, duel| plfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, 1, None)?
        .get_first();
    if let Some(tc) = tc {
        duel.move_to_field(tc, tp, tp, LOCATION_SZONE, POS_FACEUP, true)?;
    }
    Ok(())
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_ritual_monster(duel)
        && c.is_attribute(duel, ATTRIBUTE_LIGHT)
        && c.is_race(duel, RACE_WARRIOR | RACE_DRAGON)
        && c.is_faceup(duel)
        && c.is_controler(duel, tp)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.eg.unwrap_or_default().is_exists(duel, |c, duel| cfilter(c, duel, a.tp), 1, None)
}

// Whether `tp` has a face-up Lo in `location`, a condition of Barrier and Skull Guardian, Protector of the Voiceless Voice.
pub fn is_lo_faceup(duel: &Duel, tp: u8, location: u32) -> bool {
    duel.is_existing_matching_card(aux::faceup_filter(|c, duel| c.is_code(duel, ID)), tp, location, 0, 1, None)
}
