// Sylvan Dancepione
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, MZONE};

const ID: u32 = 21903613;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, st: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_PLANT, Some(lc), st, tp)
    }
}

// Link Summoned from Sunseed Genius Loci and Mushroom Man (Plants), with Plants of Levels 2 and 3 in the Deck to excavate and the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (27520594, MZONE),
    (27520594, MZONE),
    (14181608, MZONE),
    (14181608, MZONE),
    (34320307, DECK),
    (34320307, DECK),
    (34320307, DECK),
    (14181608, DECK),
    (34320307, GRAVE),
    (14181608, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DECKDES + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_LVCHANGE);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, lvtg);
    e2.set_operation(duel, lvop);
    c.register_effect(duel, e2);
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_player_can_discard_deck(tp, 1));
    }
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_DECKDES, None, 2, tp, 3);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_PLANT) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if !duel.is_player_can_discard_deck(tp, 1) {
        return Ok(());
    }
    let ct = duel.get_field_group_count(tp, LOCATION_DECK, 0).min(3);
    if ct == 0 {
        return Ok(());
    }
    // Lua passes the arguments swapped; the core truncates the hint type to a byte
    duel.hint(HINTMSG_NUMBER as u8, tp, u64::from(HINT_NUMBER));
    let ac = duel.announce_number_range(tp, 1, ct as u32)? as usize;
    duel.confirm_decktop(tp, ac)?;
    let mut g = duel.get_decktop_group(tp, ac);
    let pg = g.filter(duel, |c, duel| spfilter(c, duel, e, tp), None);
    let mut ft = duel.get_location_count(tp, LOCATION_MZONE).min(2);
    duel.disable_shuffle_check(true);
    if ft > 0 && !pg.is_empty() && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() {
            ft = 1;
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sg = pg.select(duel, tp, 1, ft as u8, None)?;
        if sg.is_empty() {
            return Ok(());
        }
        let c = e.get_handler(duel);
        for sc in sg.iter() {
            if duel.special_summon_step(sc, 0, tp, tp, false, false, POS_FACEUP)? {
                let e1 = duel.create_effect(c);
                e1.set_description(duel, 3312);
                e1.set_type(duel, EFFECT_TYPE_SINGLE);
                e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
                e1.set_code(duel, EFFECT_CANNOT_BE_LINK_MATERIAL);
                e1.set_value(duel, 1);
                e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
                sc.register_effect_full(duel, e1, true);
                g.remove(sc);
            }
        }
        duel.special_summon_complete()?;
    }
    duel.send_to_grave(g, REASON_EFFECT | REASON_EXCAVATE)?;
    Ok(())
}

fn lvfilter(c: CardId, duel: &Duel, lg: CardSet) -> bool {
    c.is_race(duel, RACE_PLANT) && c.has_level(duel) && lg.is_exists(duel, |lc, duel| !lc.is_level(duel, &[c.get_level(duel)]), 1, None)
}

fn linked_levels(c: CardId, duel: &Duel) -> CardSet {
    c.get_linked_group(duel)
        .filter(duel, |c, duel| c.is_faceup(duel) && c.has_level(duel), None)
}

fn lvtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let lg = linked_levels(a.e.get_handler(duel), duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && lvfilter(chkc, duel, lg));
    }
    if chk == 0 {
        return Ok(!lg.is_empty() && duel.is_existing_target(|c, duel| lvfilter(c, duel, lg), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, |c, duel| lvfilter(c, duel, lg), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    Ok(false)
}

fn lvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let Some(tc) = duel.get_first_target() else {
        return Ok(());
    };
    if !tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let lg = linked_levels(c, duel);
    if lg.is_empty() {
        return Ok(());
    }
    let lv = tc.get_level(duel);
    for lc in lg.iter() {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CHANGE_LEVEL);
        e1.set_value(duel, lv as i32);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        lc.register_effect(duel, e1);
    }
    Ok(())
}
