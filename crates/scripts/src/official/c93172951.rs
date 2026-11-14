// Clown Crew Meteor
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 93172951;

pub const LISTED_SERIES: &[u16] = &[SET_CLOWN_CREW];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp)
    }
}

// In the Extra Deck and on the field, Synchro Summoned with Effect Veiler (Tuner) and Shapesnatch (Level 5), with Clown Crew
// Biancaviso in the hand to Tribute it and in the Graveyard to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (97268402, MZONE),
    (4035199, MZONE),
    (82159583, HAND),
    (82159583, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];
pub const LISTED_NAMES: &[u32] = &[ID];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_operation(duel, limop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_RELEASE);
    e2.set_target(duel, efftg);
    e2.set_operation(duel, effop);
    c.register_effect(duel, e2);
}

fn tsfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_tribute_summoned(duel) && c.is_summon_player(duel, tp)
}

fn limop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if a.eg.unwrap_or_default().is_exists(duel, |c, duel| tsfilter(c, duel, tp), 1, None) {
        duel.set_chain_limit_till_chain_end(|_, _, rp, tp, _| tp == rp, 0);
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_CLOWN_CREW]) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn synchrofilter(c: CardId, duel: &Duel) -> bool {
    c.is_synchro_monster(duel) && c.is_faceup(duel) && c.is_able_to_extra(duel)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && thfilter(chkc, duel));
    }
    let b1 = !duel.has_flag_effect(tp, ID)
        && duel.is_existing_matching_card(
            synchrofilter,
            tp,
            LOCATION_MZONE | LOCATION_GRAVE,
            LOCATION_MZONE | LOCATION_GRAVE,
            1,
            None,
        );
    let b2 = !duel.has_flag_effect(tp, ID + 1) && duel.is_existing_target(thfilter, tp, LOCATION_GRAVE, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
        e.set_category(duel, CATEGORY_TOEXTRA);
        e.set_property(duel, EFFECT_FLAG_DELAY);
        duel.set_operation_info(0, CATEGORY_TOEXTRA, None, 1, PLAYER_ALL, (LOCATION_MZONE | LOCATION_GRAVE) as i32);
    } else if op == Some(2) {
        duel.register_flag_effect(tp, ID + 1, RESET_PHASE | PHASE_END, 0, 1);
        e.set_category(duel, CATEGORY_TOHAND);
        e.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g = duel.select_target(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
        duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, tp, 0);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    match a.e.get_label(duel) {
        1 => {
            let g = duel.get_matching_group(
                synchrofilter,
                tp,
                LOCATION_MZONE | LOCATION_GRAVE,
                LOCATION_MZONE | LOCATION_GRAVE,
                None,
            );
            if !g.is_empty() {
                duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
            }
        }
        2 => {
            if let Some(tc) = duel.get_first_target()
                && tc.is_relate_to_effect(duel, a.e)
            {
                duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
            }
        }
        _ => {}
    }
    Ok(())
}
