// Dark Magical Curtain
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND};

const ID: u32 = 41350417;

pub const LISTED_NAMES: &[u32] = &[CARD_DARK_MAGICIAN, CARD_DARK_MAGICIAN_GIRL, ID];

// Dark Magician and Dark Magician Girl in the Deck and the hand to Special Summon, and Dark Magic Attack in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (46986414, DECK),
    (46986414, DECK),
    (46986414, HAND),
    (38033121, DECK),
    (38033121, DECK),
    (2314238, DECK),
    (2314238, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::SpecialSummonedCode(CARD_DARK_MAGICIAN_GIRL),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0);
    }
    duel.set_possible_operation_info(
        0,
        CATEGORY_SPECIAL_SUMMON,
        None,
        1,
        PLAYER_EITHER,
        (LOCATION_HAND | LOCATION_DECK) as i32,
    );
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, sp: u8) -> bool {
    c.is_attribute(duel, ATTRIBUTE_DARK)
        && c.is_race(duel, RACE_SPELLCASTER)
        && c.is_can_be_special_summoned_full(duel, e, 0, sp, false, false, POS_FACEUP, sp, 0xff)
}

fn spresolution(duel: &mut Duel, e: EffectId, sp: u8) -> Script<bool> {
    let mut name_chk = false;
    if duel.get_location_count_full(sp, LOCATION_MZONE, sp, LOCATION_REASON_TOFIELD, 0xff) > 0
        && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, sp), sp, LOCATION_HAND | LOCATION_DECK, 0, 1, None)
        && duel.select_yes_no(sp, aux::stringid(ID, 1))?
    {
        duel.hint(HINT_SELECTMSG, sp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(
            sp,
            |c, duel| spfilter(c, duel, e, sp),
            sp,
            LOCATION_HAND | LOCATION_DECK,
            0,
            1,
            1,
            None,
        )?;
        if let Some(sc) = g.get_first()
            && duel.special_summon_step(sc, 0, sp, sp, false, false, POS_FACEUP)?
        {
            let code = sc.get_original_code_rule(duel);
            name_chk = code == CARD_DARK_MAGICIAN || code == CARD_DARK_MAGICIAN_GIRL;
            let e1 = duel.create_effect(e.get_handler(duel));
            e1.set_description(duel, 3302);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
            e1.set_code(duel, EFFECT_CANNOT_TRIGGER);
            e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
            sc.register_effect(duel, e1);
        }
    }
    Ok(name_chk)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel) && c.lists_code(duel, CARD_DARK_MAGICIAN) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let turn_player = duel.get_turn_player();
    let non_turn_player = 1 - turn_player;
    let turn_player_name_chk = spresolution(duel, e, turn_player)?;
    let non_turn_player_name_chk = spresolution(duel, e, non_turn_player)?;
    if duel.special_summon_complete()? == 0 {
        return Ok(());
    }
    if !((turn_player_name_chk && tp == turn_player) || (non_turn_player_name_chk && tp == non_turn_player)) {
        return Ok(());
    }
    if duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None) && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.break_effect()?;
            duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, g)?;
        }
    }
    Ok(())
}
