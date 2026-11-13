// Galaxy Satellite Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 92362073;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_DRAGON, Some(lc), sumtype, tp)
    }
}

// Copies on the field, in the Extra Deck and in the Graveyard, with Number 62 and Number 107 (LIGHT Dragon Xyz Monsters) to power up, and Dragons to Link Summon from.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, GRAVE),
    (ID, EXTRA),
    (31801517, MZONE),
    (88177324, MZONE),
    (75356564, MZONE),
    (76704943, MZONE),
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
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE | LOCATION_GRAVE);
    e1.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, _| {
        let ph = duel.get_current_phase();
        ph > PHASE_MAIN1 && ph < PHASE_MAIN2
    });
    e1.set_cost(duel, Cost::self_banish);
    e1.set_target(duel, atktg);
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_PHASE | PHASE_END);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e2.set_target(duel, tdtg);
    e2.set_operation(duel, tdop);
    c.register_effect(duel, e2);
}

fn atkfilter(c: CardId, duel: &Duel) -> bool {
    let Some(number) = crate::official::xyz_number(c.get_code(duel)) else {
        return false;
    };
    c.is_faceup(duel)
        && c.is_set_card(duel, &[SET_NUMBER])
        && c.is_type(duel, TYPE_XYZ)
        && c.get_original_race(duel) & RACE_DRAGON == RACE_DRAGON
        && c.get_original_attribute(duel) & ATTRIBUTE_LIGHT == ATTRIBUTE_LIGHT
        && number > 0
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && atkfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(atkfilter, tp, LOCATION_MZONE, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, atkfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CHANGE_BATTLE_DAMAGE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 0, 1);
    e1.set_value(duel, HALF_DAMAGE as i32);
    e1.set_reset(duel, RESET_PHASE | PHASE_BATTLE);
    duel.register_effect(e1, tp);
    aux::register_client_hint_full(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 2)), PHASE_BATTLE, 1);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_facedown(duel) || !tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let number = crate::official::xyz_number(tc.get_code(duel)).unwrap_or_default();
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_SET_ATTACK_FINAL);
    e3.set_value(duel, number as i32 * 100);
    e3.set_reset_full(duel, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_BATTLE, 1);
    tc.register_effect(duel, e3);
    Ok(())
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(aux::TRUE, a.tp, LOCATION_DECK, 0, 1, None));
    }
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 3));
    let g = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.first() {
        duel.shuffle_deck(tp)?;
        duel.move_sequence(tc, 0)?;
        duel.confirm_decktop(tp, 1)?;
    }
    Ok(())
}
