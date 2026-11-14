// Shiranui Sunsaga
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 59843383;

pub const LISTED_SERIES: &[u16] = &[SET_SHIRANUI];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_ZOMBIE, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_race_full(duel, RACE_ZOMBIE, Some(sc), sumtype, tp)
    }
}

// In the Extra Deck, Synchro Summoned with Shiranui Spectralsword (a Level 2 Tuner) and two Zombinos (Level 4), with a Zombie Synchro Monster
// in the GY to return, a Shiranui monster in the GY to banish and the opponent's Dark Hole to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (36630403, MZONE),
    (36630403, MZONE),
    (14575467, MZONE),
    (14575467, MZONE),
    (14575467, MZONE),
    (5309481, GRAVE),
    (41562624, GRAVE),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Replaced];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.set_sp_summon_once(duel, ID);
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TODECK + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_target(duel, tdtg);
    e1.set_operation(duel, tdop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_DESTROY_REPLACE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target(duel, reptg);
    e2.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(repfilter(c, duel, e.get_handler_player(duel)))),
    );
    c.register_effect(duel, e2);
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_location(duel, LOCATION_GRAVE) || c.is_faceup(duel))
        && c.is_type(duel, TYPE_SYNCHRO)
        && c.is_race(duel, RACE_ZOMBIE)
        && c.is_able_to_deck(duel)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, tdfilter, LOCATION_GRAVE | LOCATION_REMOVED, CATEGORY_TODECK)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(tdfilter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, None);
    if g.is_empty() {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let sg = g.select(duel, tp, 1, g.len() as u8, None)?;
    let ct = duel.send_to_deck(sg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? as usize;
    let dg = duel.get_matching_group(aux::TRUE, tp, 0, LOCATION_ONFIELD, None);
    if ct > 0 && dg.len() >= ct && duel.select_yes_no(tp, aux::stringid(ID, 1))? {
        duel.break_effect()?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
        let sg2 = dg.select(duel, tp, ct as u8, ct as u8, None)?;
        duel.destroy(sg2, REASON_EFFECT)?;
    }
    Ok(())
}

fn repfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_controler(duel, tp)
        && c.is_location(duel, LOCATION_MZONE)
        && c.is_race(duel, RACE_ZOMBIE)
        && !c.is_reason(duel, REASON_REPLACE)
        && c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SHIRANUI]) && c.is_monster(duel) && c.is_able_to_remove(duel) && aux::sp_elim_filter(c, duel, true, false)
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(a.eg.unwrap_or_default().is_exists(duel, |c, duel| repfilter(c, duel, tp), 1, None)
            && duel.is_existing_matching_card(rmfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, None));
    }
    let handler = e.get_handler(duel);
    if duel.select_effect_yes_no_full(tp, handler, 96)? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
        let g = duel.select_matching_card(tp, rmfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, 1, None)?;
        duel.remove(g, POS_FACEUP, REASON_EFFECT | REASON_REPLACE)?;
        return Ok(true);
    }
    Ok(false)
}
