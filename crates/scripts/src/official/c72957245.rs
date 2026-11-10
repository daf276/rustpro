// Adamancipator Signs
use crate::prelude::*;
use crate::testing::{DECK, GRAVE};

const ID: u32 = 72957245;

// An Adamancipator (Researcher) and a Rock (Destroyer Golem) in the Graveyard, with Rock Level 4 or lower Normal Monsters in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (85914562, GRAVE),
    (73481154, GRAVE),
    (73481154, DECK),
    (73481154, DECK),
    (73481154, DECK),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_ROCK) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && filter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| filter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| filter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_ROCK) && c.is_level_below(duel, 4)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !(tc.is_relate_to_effect(duel, e) && duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)? > 0) {
        return Ok(());
    }
    let Some(dc) = duel.get_operated_group().get_first() else {
        return Ok(());
    };
    if dc.is_set_card(duel, &[SET_ADAMANCIPATOR])
        && duel.is_existing_matching_card(tdfilter, tp, LOCATION_DECK, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 1))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
        if let Some(tc) = duel
            .select_matching_card(tp, tdfilter, tp, LOCATION_DECK, 0, 1, 1, None)?
            .get_first()
        {
            duel.break_effect()?;
            duel.shuffle_deck(tp)?;
            duel.move_sequence(tc, 0)?;
            duel.confirm_decktop(tp, 1)?;
        }
    }
    Ok(())
}
