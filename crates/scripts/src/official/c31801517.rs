// Number 62: Galaxy-Eyes Prime Photon Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 31801517;
pub const XYZ_NUMBER: u32 = 62;

pub const LISTED_NAMES: &[u32] = &[CARD_GALAXYEYES_P_DRAGON];

// In the Extra Deck, Xyz Summoned with Galaxy-Eyes Photon Dragon and a Level 8 Normal Monster, the opponent's Dark Hole and Raigeki to
// destroy it, and the opponent's monsters to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (CARD_GALAXYEYES_P_DRAGON, MZONE),
    (89631139, MZONE),
    (CARD_GALAXYEYES_P_DRAGON, MZONE),
    (53129443, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
    (7892180, MZONE | OPPONENT),
    (59053232, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(8);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_PRE_DAMAGE_CALCULATE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_relate_to_battle(duel));
    e1.set_cost(duel, |duel, a, chk| {
        Cost::and(duel, a, chk, &[Cost::detach_from_self::<1, 1>, Cost::soft_once_per_battle::<ID>])
    });
    e1.set_operation(duel, atkop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_CHANGE_BATTLE_DAMAGE);
    e3.set_condition(duel, |duel, a| !has_photon_material(duel, a.e.get_handler(duel)));
    e3.set_value(duel, Value::BattleDamage(aux::change_battle_damage::<1, HALF_DAMAGE>));
    c.register_effect(duel, e3);
}

fn has_photon_material(duel: &Duel, c: CardId) -> bool {
    c.get_overlay_group(duel)
        .is_exists(duel, |mc, duel| mc.is_code(duel, CARD_GALAXYEYES_P_DRAGON), 1, None)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && c.is_faceup(duel) {
        let rks = duel
            .get_matching_group(CardId::is_faceup, a.tp, LOCATION_MZONE, LOCATION_MZONE, None)
            .get_sum(duel, |tc, duel| i64::from(tc.get_rank(duel)));
        if rks > 0 {
            let reset = RESET_EVENT | RESETS_STANDARD_DISABLE | RESET_PHASE | PHASE_DAMAGE_CAL;
            c.update_attack(duel, rks as i32 * 200, Some(reset), None, None);
        }
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_controler(duel, a.tp)
        && a.rp == 1 - a.tp
        && c.is_reason(duel, REASON_EFFECT)
        && c.is_previous_location(duel, LOCATION_MZONE)
        && has_photon_material(duel, c)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let c = a.e.get_handler(duel);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, a.tp, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) {
        c.set_turn_counter(duel, 0);
        let turn_count = duel.get_turn_count();
        let reset_count = if duel.is_phase(PHASE_STANDBY) && duel.is_turn_player(tp) {
            3
        } else {
            2
        };
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
        e1.set_range(duel, c.get_location(duel));
        e1.set_count_limit(duel, 1, 0);
        e1.set_labels(duel, &[i64::from(turn_count), reset_count]);
        e1.set_condition(duel, standbycon);
        e1.set_operation(duel, standbyspop);
        e1.set_reset_full(
            duel,
            RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_STANDBY | RESET_SELF_TURN,
            reset_count as u16,
        );
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn standbycon(duel: &Duel, a: &Args) -> bool {
    let &[turn_count, reset_count] = a.e.get_labels(duel) else {
        return false;
    };
    duel.is_turn_player(a.tp) && (reset_count == 2 || i64::from(duel.get_turn_count()) != turn_count)
}

fn standbyspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let ct = c.get_turn_counter(duel) + 1;
    c.set_turn_counter(duel, ct);
    if ct == 2 && duel.special_summon_step(c, 0, tp, tp, false, false, POS_FACEUP)? {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK);
        e1.set_value(duel, c.get_attack(duel) * 2);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        c.register_effect(duel, e1);
    }
    duel.special_summon_complete()?;
    Ok(())
}
