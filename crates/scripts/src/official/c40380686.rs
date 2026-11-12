// Baby Mudragon
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE};

const ID: u32 = 40380686;
const RACE: u8 = 0;
const ATTRIBUTE: u8 = 1;

// On the field with Ash Blossom (Level 3 Tuner) as Synchro material for Armory Arm (Level 4), which it then changes.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (14558127, MZONE),
    (14558127, MZONE),
    (29071332, EXTRA),
    (29071332, EXTRA),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_NONTUNER);
    e1.set_range(duel, LOCATION_MZONE);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_BE_MATERIAL);
    e2.set_condition(duel, chcon);
    e2.set_target(duel, chtg);
    e2.set_operation(duel, chop);
    c.register_effect(duel, e2);
}

fn chcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_location(duel, LOCATION_GRAVE) && a.r == REASON_SYNCHRO
}

fn chfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_SYNCHRO) && c.is_can_be_effect_target(duel, e)
}

fn chtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(|c, duel| chfilter(c, duel, e), tp, LOCATION_MZONE, 0, None);
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    if duel.select_option(tp, &[aux::stringid(ID, 1), aux::stringid(ID, 2)])? == 0 {
        let race = duel.announce_another_race(g, tp)?;
        e.set_chain_data(duel, RACE, race as i64);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
        let sg = g.filter_select(duel, tp, |c, duel| c.is_race_except(duel, race), 1, 1, None)?;
        duel.set_target_card(sg);
    } else {
        let attribute = duel.announce_another_attribute(g, tp)?;
        e.set_chain_data(duel, ATTRIBUTE, i64::from(attribute));
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
        let sg = g.filter_select(duel, tp, |c, duel| c.is_attribute_except(duel, attribute), 1, 1, None)?;
        duel.set_target_card(sg);
    }
    Ok(false)
}

fn chop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let Some(tc) = duel.get_first_target() else {
        return Ok(());
    };
    if !tc.is_relate_to_effect(duel, e) || tc.is_facedown(duel) {
        return Ok(());
    }
    let race = e.get_chain_data(duel, RACE).filter(|&race| tc.is_race_except(duel, race as u64));
    let Some((code, value)) = race.map(|race| (EFFECT_CHANGE_RACE, race)).or_else(|| {
        let attribute = e.get_chain_data(duel, ATTRIBUTE)?;
        tc.is_attribute_except(duel, attribute as u32)
            .then_some((EFFECT_CHANGE_ATTRIBUTE, attribute))
    }) else {
        return Ok(());
    };
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, code);
    e1.set_value(duel, value as i32);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    tc.register_effect(duel, e1);
    Ok(())
}
