// Drill Warrior
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 3429238;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_summon_code(duel, Some(sc), sumtype, tp, &[56286179]) || c.is_has_effect(duel, 20932152).is_some()
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub const MATERIAL: &[u32] = &[56286179];
pub const LISTED_NAMES: &[u32] = &[56286179];
pub const MATERIAL_SETCODE: &[u16] = &[SET_SYNCHRON];
// Two on the field and one in the Extra Deck, Synchro Summoned with Drill Synchron and Swordsman of Landstar, with Psychic
// Kappa, a Normal Monster, in the hand to discard and in the Graveyard to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (ID, MZONE),
    (56286179, MZONE),
    (3573512, MZONE),
    (7892180, HAND),
    (7892180, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, |duel, _| duel.is_phase(PHASE_MAIN1));
    e1.set_target(duel, dattg);
    e1.set_operation(duel, datop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_count_limit(duel, 1, 0);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target(duel, rmtg);
    e2.set_operation(duel, rmop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e3.set_range(duel, LOCATION_REMOVED);
    e3.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn dattg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let description = a.e.get_description(duel);
    duel.hint(HINT_OPSELECTED, 1 - a.tp, description);
    Ok(false)
}

fn datop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && c.is_faceup(duel) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_value(duel, (f64::from(c.get_attack(duel)) / 2.0).round() as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_code(duel, EFFECT_DIRECT_ATTACK);
        e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e2);
    }
    Ok(())
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.get_field_group_count(a.tp, LOCATION_HAND, 0) > 0 && c.is_able_to_remove(duel));
    }
    let description = a.e.get_description(duel);
    duel.hint(HINT_OPSELECTED, 1 - a.tp, description);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DISCARD);
    let g = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_HAND, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_grave(g, REASON_EFFECT | REASON_DISCARD)?;
        if c.is_relate_to_effect(duel, e) && duel.remove(c, POS_FACEUP, REASON_EFFECT)? != 0 {
            c.register_flag_effect(
                duel,
                ID,
                RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_STANDBY | RESET_SELF_TURN,
                0,
                1,
            );
        }
    }
    Ok(())
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.get_flag_effect(duel, ID) != 0);
    }
    c.reset_flag_effect(duel, ID);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    if duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)? != 0 {
        duel.break_effect()?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g = duel.select_matching_card(tp, aux::necro_valley_filter(filter), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, g)?;
        }
    }
    Ok(())
}
