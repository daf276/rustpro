// Cocatorium the Heavy Metal Avian
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 94395649;

// Copies in the hand, Graveyard and field, with Beast monsters (Gazelle, Pale Beast, Battle Ox, Takuhee) on the field to
// release and in the Deck to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (ID, MZONE),
    (5818798, MZONE),
    (5818798, MZONE),
    (21263083, DECK),
    (21263083, DECK),
    (5053103, DECK),
    (3170832, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND | LOCATION_GRAVE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, spcost);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, copycost);
    e2.set_operation(duel, copyop);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel, ft: i32, tp: u8) -> bool {
    c.is_monster(duel)
        && c.is_race(duel, RACES_BEAST_BWARRIOR_WINGB)
        && (ft > 0 || (c.is_controler(duel, tp) && c.get_sequence(duel) < 5))
        && (c.is_controler(duel, tp) || c.is_faceup(duel))
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    let filter = |c, duel: &Duel| cfilter(c, duel, ft, tp);
    if chk == 0 {
        return Ok(ft > -1 && duel.check_release_group_cost(tp, filter, (1, 1), false, None, None));
    }
    let g = duel.select_release_group_cost(tp, filter, (1, 1), false, None, None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_can_be_special_summoned_full(duel, a.e, 0, a.tp, false, false, POS_FACEUP, a.tp, 0xff));
    }
    let location = c.get_location(duel);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, location as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? != 0 {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3300);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
        e1.set_value(duel, LOCATION_REMOVED as i32);
        c.register_effect_full(duel, e1, true);
    }
    Ok(())
}

fn costfilter(c: CardId, duel: &Duel, (code, lv, race, att): (u32, u32, u64, u32)) -> bool {
    c.is_monster(duel)
        && c.is_able_to_remove_as_cost(duel)
        && c.is_level_below(duel, 4)
        && c.is_race(duel, RACES_BEAST_BWARRIOR_WINGB)
        && (!c.is_code(duel, code) || !c.is_level(duel, &[lv]) || !c.is_race(duel, race) || !c.is_attribute(duel, att))
}

fn copycost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let stats = (c.get_code(duel), c.get_level(duel), c.get_race(duel), c.get_attribute(duel));
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| costfilter(c, duel, stats), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, |c, duel| costfilter(c, duel, stats), tp, LOCATION_DECK, 0, 1, 1, None)?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    e.set_label_object_group(duel, g.get_first().map(Into::into));
    Ok(false)
}

fn copyop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let c = e.get_handler(duel);
    let Some(tc) = e.get_label_object_group(duel).and_then(|g| g.get_first()) else {
        return Ok(());
    };
    if c.is_relate_to_effect(duel, e) && !c.is_facedown(duel) {
        let others = [
            (EFFECT_CHANGE_LEVEL, tc.get_level(duel) as i32),
            (EFFECT_CHANGE_RACE, tc.get_race(duel) as i32),
            (EFFECT_CHANGE_ATTRIBUTE, tc.get_attribute(duel) as i32),
        ];
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CHANGE_CODE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        e1.set_value(duel, tc.get_code(duel) as i32);
        c.register_effect(duel, e1);
        for (code, value) in others {
            let e2 = e1.clone_effect(duel);
            e2.set_code(duel, code);
            e2.set_value(duel, value);
            c.register_effect(duel, e2);
        }
    }
    Ok(())
}
