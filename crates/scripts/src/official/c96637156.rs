// Musical Sumo Dice Games
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 96637156;

// s.roll_dice, which only cards that are not ported read, is left out.
// In the Extra Deck, Xyz Summoned with two Level 6 Normal Monsters (Boulder Tortoise).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (9540040, MZONE),
    (9540040, MZONE),
    (9540040, MZONE),
    (9540040, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(6);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DICE + CATEGORY_CONTROL + CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_PHASE | PHASE_BATTLE_START);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e1.set_target(duel, mvtg);
    e1.set_operation(duel, mvop);
    c.register_effect(duel, e1);
}

fn mvtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DICE, None, 0, a.tp, 1);
    Ok(false)
}

fn cannot_move(duel: &mut Duel, c: CardId) -> Script<()> {
    duel.send_to_grave(c, REASON_EFFECT)?;
    Ok(())
}

fn mvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, c) = (a.e, a.tp, a.e.get_handler(duel));
    if !(c.is_relate_to_effect(duel, e) && c.is_in_main_mzone(duel, None)) {
        return Ok(());
    }
    let Some(&dice) = duel.toss_dice(tp, 1)?.first() else {
        return Ok(());
    };
    if !(1..=6).contains(&dice) {
        return Ok(());
    }
    let mut seq = c.get_sequence(duel) as i32;
    let col3 = duel.is_duel_type(DUEL_3_COLUMNS_FIELD);
    let max = if col3 { 4 } else { 5 };
    for _ in 0..dice {
        seq -= 1;
        if seq == 0 && col3 {
            seq = -2;
        } else if seq < -max {
            seq = max - 1;
        }
    }
    let switch = seq < 0;
    if switch && !c.is_able_to_change_controler(duel) {
        return cannot_move(duel, c);
    }
    let fp = if switch { 1 - tp } else { tp };
    let nseq = if switch { 5 + seq } else { seq } as u32;
    let tc = duel.get_field_card(fp, LOCATION_MZONE, nseq);
    if duel.get_mzone_count_full(fp, tc.map(CardSet::from), tp, LOCATION_REASON_TOFIELD, 1 << nseq) < 1 {
        return cannot_move(duel, c);
    }
    let mut win = false;
    if let Some(tc) = tc {
        if c == tc || tc.is_immune_to_effect(duel, e) || tc.is_type(duel, TYPE_TOKEN) || !c.is_type(duel, TYPE_XYZ) {
            return cannot_move(duel, c);
        }
        let prev = c.get_overlay_count(duel);
        duel.overlay(c, tc)?;
        win = prev < 7 && c.get_overlay_count(duel) > 6;
    }
    // The Lua's GetControl count is 0 on failure, which is truthy, so a switch never falls through to MoveSequence.
    let moved = if switch {
        duel.get_control_full(c, 1 - tp, 0, 0, 1 << nseq, duel.get_reason_player())?;
        true
    } else {
        duel.move_sequence(c, seq as u8)?
    };
    if !moved {
        return cannot_move(duel, c);
    }
    if win {
        duel.win(tp, WIN_REASON_MUSICAL_SUMO);
    }
    Ok(())
}
