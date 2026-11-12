// Egotistical Ape
use crate::prelude::*;
use crate::testing::HAND;

const ID: u32 = 96914272;

// In the hand with Beast Normal Monsters of Levels 1 and 3 to send to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (69380702, HAND), (6297941, HAND)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_condition(duel, lvcon);
    e2.set_operation(duel, lvop);
    e2.set_label_object(duel, Some(e1));
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_BEAST) && c.is_able_to_grave_as_cost(duel)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = a.e.get_handler_player(duel);
    let rg = duel.get_matching_group(spfilter, tp, LOCATION_HAND, 0, Some(c.into()));
    duel.get_location_count(tp, LOCATION_MZONE) > 0 && aux::select_unselect_group_check(duel, rg, 1, 1, &|_, _, _, _| (true, false))
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let rg = duel.get_matching_group(spfilter, tp, LOCATION_HAND, 0, Some(c.into()));
    let g = aux::select_unselect_group_full(duel, rg, 1, 1, &|_, _, _, _| (true, false), tp, HINTMSG_TOGRAVE, None, true)?;
    if g.is_empty() {
        return Ok(false);
    }
    e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let Some(g) = e.get_label_object_group(duel) else { return Ok(()) };
    duel.send_to_grave(g, REASON_COST)?;
    if let Some(tc) = g.get_first() {
        e.set_label(duel, i64::from(tc.get_level(duel)));
    }
    Ok(())
}

fn lvcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).get_summon_type(duel) == SUMMON_TYPE_SPECIAL + 1
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c, tp) = (a.e, a.e.get_handler(duel), a.tp);
    if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
        let lv = e.get_label_object(duel).map_or(0, |e1| e1.get_label(duel));
        let clv = i64::from(c.get_level(duel));
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_LEVEL);
        if lv < clv {
            if duel.select_option(tp, &[aux::stringid(ID, 1), aux::stringid(ID, 2)])? == 0 {
                e1.set_value(duel, lv as i32);
            } else {
                e1.set_value(duel, -lv as i32);
            }
        } else {
            duel.select_option(tp, &[aux::stringid(ID, 1)])?;
            e1.set_value(duel, lv as i32);
        }
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
    }
    Ok(())
}
