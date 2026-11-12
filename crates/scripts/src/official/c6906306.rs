// Rubysapphirus, the Adamant Jewel
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};
use engine::constants::MSG_SELECT_EFFECTYN;

const ID: u32 = 6906306;

// In the Extra Deck, Xyz Summoned with Vermillion Dragon Mech (Fire) and Navy Dragon Mech (Water), both Level 9, with the opponent's Black Luster Soldier
// (ATK 3000) to attack. The destruction replacement is no chain link and moves no card; its prompt is the MSG_SELECT_EFFECTYN.
// The ATK doubling needs a Fire material and an attack on a stronger monster, which random play reached once in 64 scenarios.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (66698383, MZONE),
    (66698383, MZONE),
    (80040886, MZONE),
    (10000100, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 0))),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::Message(MSG_SELECT_EFFECTYN),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(9);
    const CT: usize = 2;
    const MAXCT: usize = Xyz::INFINITE_MATS;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_DESTROY_REPLACE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target(duel, reptg);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, atkcon);
    e2.set_operation(duel, atkop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DISABLE);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER | TIMING_MAIN_END);
    e3.set_condition(duel, negcon);
    e3.set_target(duel, negtg);
    e3.set_operation(duel, negop);
    c.register_effect(duel, e3);
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(!c.is_reason(duel, REASON_REPLACE) && c.check_remove_overlay_card(duel, tp, 1, REASON_EFFECT));
    }
    Ok(duel.select_effect_yes_no_full(tp, c, 96)? && c.remove_overlay_card(duel, tp, 1, 1, REASON_EFFECT)? > 0)
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    if !c
        .get_overlay_group(duel)
        .is_exists(duel, |c, duel| c.is_attribute(duel, ATTRIBUTE_FIRE), 1, None)
    {
        return false;
    }
    duel.get_attack_target()
        .is_some_and(|bc| bc.is_faceup(duel) && bc.get_attack(duel) > c.get_attack(duel))
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_faceup(duel) && c.is_relate_to_battle(duel) {
        let atk = c.get_attack(duel) * 2;
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_value(duel, atk);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_DAMAGE);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(1 - a.tp)
        && a.e
            .get_handler(duel)
            .get_overlay_group(duel)
            .is_exists(duel, |c, duel| c.is_attribute(duel, ATTRIBUTE_WATER), 1, None)
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, 0, 1, 1, None)?;
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tc = duel.get_first_target();
    unported!(duel, tc.is_none());
    let Some(tc) = tc else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) {
        tc.register_flag_effect_full(
            duel,
            ID,
            RESETS_STANDARD_PHASE_END,
            EFFECT_FLAG_CLIENT_HINT,
            1,
            0,
            aux::stringid(ID, 2),
        );
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EVENT_CHAIN_SOLVING);
        e1.set_condition(duel, discon);
        e1.set_operation(duel, disop);
        e1.set_label_object_group(duel, Some(tc.into()));
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e1, tp);
    }
    Ok(())
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    if !(a.rp == 1 - a.tp && re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET)) {
        return false;
    }
    let tc = a.e.get_label_object_group(duel).and_then(|g| g.get_first());
    let tg: Option<CardSet> = duel.get_chain_info(a.ev as u8, CHAININFO_TARGET_CARDS);
    tc.is_some_and(|tc| tg.is_some_and(|tg| tg.contains(tc)) && tc.get_flag_effect(duel, ID) > 0)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.hint(HINT_CARD, 0, u64::from(ID));
    duel.negate_effect(a.ev as u8);
    Ok(())
}
