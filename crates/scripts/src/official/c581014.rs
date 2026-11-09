// Daigusto Emeral
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 581014;

// In the Extra Deck, Xyz Summoned with Mystical Elf and Flying Kamakiri #2, with Normal Monsters in the Graveyard to shuffle
// into the Deck or Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (15025844, MZONE),
    (3134241, MZONE),
    (79629370, GRAVE),
    (79629370, GRAVE),
    (15025844, GRAVE),
    (3134241, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::Resolves, Check::SpecialSummonedCode(79629370)];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_non_effect_monster(duel) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_able_to_deck(duel)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        if !(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp)) {
            return Ok(false);
        }
        match e.get_label(duel) {
            1 => return Ok(tdfilter(chkc, duel)),
            2 => return Ok(spfilter(chkc, duel, e, tp)),
            _ => {}
        }
    }
    let b1 = duel.is_player_can_draw(tp, 1) && duel.is_existing_target(tdfilter, tp, LOCATION_GRAVE, 0, 3, None);
    let b2 = duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    match op {
        Some(1) => {
            e.set_category(duel, CATEGORY_TODECK + CATEGORY_DRAW);
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
            let g = duel.select_target(tp, tdfilter, tp, LOCATION_GRAVE, 0, 3, 3, None)?;
            duel.set_operation_info(0, CATEGORY_TODECK, g, 3, tp, 0);
            duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
        }
        Some(2) => {
            e.set_category(duel, CATEGORY_SPECIAL_SUMMON);
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
            duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
        }
        _ => {}
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    match e.get_label(duel) {
        1 => {
            let tg = duel.get_target_cards(e).unwrap_or_default();
            if tg.len() != 3 || duel.send_to_deck(tg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? != 3 {
                return Ok(());
            }
            let og = duel.get_operated_group();
            if og.is_exists(duel, |c, duel| c.is_location(duel, LOCATION_DECK), 1, None) {
                duel.shuffle_deck(tp)?;
            }
            if og.filter_count(duel, |c, duel| c.is_location(duel, LOCATION_DECK | LOCATION_EXTRA), None) == 3 {
                duel.break_effect()?;
                duel.draw(tp, 1, REASON_EFFECT)?;
            }
        }
        2 => {
            if let Some(tc) = duel.get_first_target()
                && tc.is_relate_to_effect(duel, e)
            {
                duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)?;
            }
        }
        _ => {}
    }
    Ok(())
}
