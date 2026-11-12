// Nirvana High Paladin
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT, PZONE};

const ID: u32 = 80896940;

// In the Pendulum Zone and Extra Deck, Synchro Summoned from Hallohallo (Level 3 Pendulum Tuner) and Narwhal (Level 7 Synchro), with a
// Pendulum Monster to attack the opponent's monsters.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (77994337, MZONE),
    (77994337, MZONE),
    (6568731, MZONE),
    (6568731, MZONE),
    (94415058, MZONE),
    (46130346, GRAVE),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
];
// The salvage needs a Pendulum Summoned Pendulum Tuner as material, which takes two scales and a Pendulum Summon in one turn.
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 2)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 3))),
    Check::ResolvesEffect(aux::stringid(ID, 4)),
    Check::ResolvesEffect(aux::stringid(ID, 5)),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, |c, duel| c.is_type_full(duel, TYPE_SYNCHRO, Some(sc), sumtype, tp))
    }

    fn sub1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_PENDULUM, Some(sc), sumtype, tp) && c.is_pendulum_summoned(duel)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Pendulum::add_procedure_full(duel, c, false, None);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e3.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e3.set_range(duel, LOCATION_PZONE);
    e3.set_condition(duel, indcon);
    e3.set_operation(duel, indop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_category(duel, CATEGORY_ATKCHANGE);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e4.set_code(duel, EVENT_DAMAGE_STEP_END);
    e4.set_range(duel, LOCATION_PZONE);
    e4.set_condition(duel, atkcon);
    e4.set_operation(duel, atkop);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 3));
    e5.set_category(duel, CATEGORY_TOHAND);
    e5.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e5.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e5.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e5.set_condition(duel, thcon);
    e5.set_target(duel, thtg);
    e5.set_operation(duel, stock::first_target_to_hand);
    c.register_effect(duel, e5);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_SINGLE);
    e0.set_code(duel, EFFECT_MATERIAL_CHECK);
    e0.set_value(duel, Value::MaterialCheck(valcheck));
    e0.set_label_object(duel, Some(e5));
    c.register_effect(duel, e0);
    let e6 = duel.create_effect(c);
    e6.set_description(duel, aux::stringid(ID, 4));
    e6.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e6.set_code(duel, EVENT_BATTLE_DESTROYING);
    e6.set_condition(duel, aux::bdocon);
    e6.set_operation(duel, lpop);
    c.register_effect(duel, e6);
    let e7 = duel.create_effect(c);
    e7.set_description(duel, aux::stringid(ID, 5));
    e7.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e7.set_code(duel, EVENT_DESTROYED);
    e7.set_property(duel, EFFECT_FLAG_DELAY);
    e7.set_condition(duel, pencon);
    e7.set_target(duel, pentg);
    e7.set_operation(duel, penop);
    c.register_effect(duel, e7);
}

fn indcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.e.get_handler_player(duel);
    duel.get_attacker()
        .is_some_and(|attacker| attacker.is_type(duel, TYPE_PENDULUM) && attacker.is_controler(duel, tp))
}

fn indop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_attacker() else { return Ok(()) };
    let owner = a.e.get_handler(duel);
    for code in [EFFECT_INDESTRUCTABLE_BATTLE, EFFECT_AVOID_BATTLE_DAMAGE] {
        let e1 = duel.create_effect(owner);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, code);
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_DAMAGE);
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.e.get_handler_player(duel);
    duel.get_attacker().is_some_and(|attacker| {
        attacker.is_relate_to_battle(duel) && attacker.is_type(duel, TYPE_PENDULUM) && attacker.is_controler(duel, tp)
    })
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let tg = duel.get_matching_group(CardId::is_faceup, a.tp, 0, LOCATION_MZONE, None);
    for tc in tg {
        let atk = duel.get_attacker().map_or(0, |attacker| attacker.get_attack(duel));
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, -atk);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn thcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_synchro_summoned(duel) && a.e.get_label(duel) == 1
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && chkc.is_able_to_hand(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_able_to_hand, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, CardId::is_able_to_hand, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
    Ok(false)
}

fn mfilter(c: CardId, duel: &Duel, sc: CardId) -> bool {
    c.is_type_full(duel, TYPE_PENDULUM, Some(sc), SUMMON_TYPE_SYNCHRO as u64, PLAYER_NONE)
        && c.is_type_full(duel, TYPE_TUNER, Some(sc), SUMMON_TYPE_SYNCHRO as u64, PLAYER_NONE)
        && c.is_pendulum_summoned(duel)
}

fn valcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let g = c.get_material(duel);
    let label = i64::from(g.is_exists(duel, |mc, duel| mfilter(mc, duel, c), 1, None));
    if let Some(e5) = e.get_label_object(duel) {
        e5.set_label(duel, label);
    }
}

fn lpop(duel: &mut Duel, a: &Args) -> Script<()> {
    let opponent = 1 - a.tp;
    let lp = duel.get_lp(opponent);
    duel.set_lp(opponent, (lp + 1) / 2);
    Ok(())
}

fn pencon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    a.r & (REASON_EFFECT + REASON_BATTLE) != 0 && c.is_previous_location(duel, LOCATION_MZONE) && c.is_faceup(duel)
}

fn pentg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.check_pendulum_zones(a.tp))
}

fn penop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if !duel.check_pendulum_zones(tp) {
        return Ok(());
    }
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.move_to_field(c, tp, tp, LOCATION_PZONE, POS_FACEUP, true)?;
    }
    Ok(())
}
