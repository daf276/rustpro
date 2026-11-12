// D/D/D Super Doom King Bright Armageddon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT, PZONE};

const ID: u32 = 72402069;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_DD], Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, |c, duel| {
            c.is_set_card_full(duel, &[SET_DDD], Some(sc), sumtype, tp)
        })
    }
}

// A copy in the Pendulum Zone, one on the field and one in the Extra Deck, Synchro Summoned with D/D Orthros (a Level 4 Pendulum Tuner)
// and Clovis (Level 6), with Sangan and D/D Orthros on the other side to attack and to be negated.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (ID, MZONE),
    (ID, EXTRA),
    (72181263, MZONE),
    (70576413, MZONE),
    (72181263, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    Pendulum::add_procedure_full(duel, c, false, None);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, |duel, a| {
        duel.get_attacker().is_some_and(|attacker| attacker.is_controler(duel, 1 - a.tp))
    });
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e2.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e2);
    let e3a = duel.create_effect(c);
    e3a.set_description(duel, aux::stringid(ID, 1));
    e3a.set_category(duel, CATEGORY_DISABLE);
    e3a.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3a.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e3a.set_code(duel, EVENT_SUMMON_SUCCESS);
    e3a.set_range(duel, LOCATION_MZONE);
    e3a.set_condition(duel, |duel, a| a.eg.is_some_and(|eg| !eg.is_contains(a.e.get_handler(duel))));
    e3a.set_target(duel, distg);
    e3a.set_operation(duel, disop);
    c.register_effect(duel, e3a);
    let e3b = e3a.clone_effect(duel);
    e3b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3b);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_DELAY);
    e4.set_code(duel, EVENT_DESTROYED);
    e4.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_MZONE));
    e4.set_target(duel, |duel, a, chk, _| Ok(chk == 0 && duel.check_pendulum_zones(a.tp)));
    e4.set_operation(duel, penop);
    c.register_effect(duel, e4);
}

fn tgfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    let atk = c.get_attack(duel);
    c.is_set_card(duel, &[SET_DDD])
        && c.is_type(duel, TYPE_SYNCHRO)
        && c.is_faceup(duel)
        && duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_defense_below(duel, atk)),
            tp,
            0,
            LOCATION_MZONE,
            1,
            None,
        )
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && tgfilter(chkc, duel, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| tgfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let Some(tc) = duel
        .select_target(tp, |c, duel| tgfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?
        .and_then(|g| g.get_first())
    else {
        return Ok(false);
    };
    let atk = tc.get_attack(duel);
    let g = duel.get_matching_group(
        aux::faceup_filter(|c, duel| c.is_defense_below(duel, atk)),
        tp,
        0,
        LOCATION_MZONE,
        None,
    );
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, tp, 0);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, g.len() as i32 * 1000);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel) {
        let atk = tc.get_attack(duel);
        let g = duel.get_matching_group(
            aux::faceup_filter(|c, duel| c.is_defense_below(duel, atk)),
            tp,
            0,
            LOCATION_MZONE,
            None,
        );
        if g.is_empty() {
            return Ok(());
        }
        let ct = duel.destroy(g, REASON_EFFECT)?;
        if ct > 0 {
            duel.damage(1 - tp, ct as u32 * 1000, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn pendulumdisfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_type(duel, TYPE_PENDULUM)
        && c.is_faceup(duel)
        && duel.is_existing_matching_card(CardId::is_negatable_monster, tp, 0, LOCATION_MZONE, 1, Some(c.into()))
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| pendulumdisfilter(c, duel, tp), tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, None, 1, 1 - tp, LOCATION_MZONE as i32);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, 1 - tp, aux::stringid(ID, 3));
    let sg = duel.select_matching_card(1 - tp, |c, duel| pendulumdisfilter(c, duel, tp), tp, 0, LOCATION_MZONE, 1, 1, None)?;
    if sg.is_empty() {
        return Ok(());
    }
    duel.hint_selection(sg);
    let c = a.e.get_handler(duel);
    let g = duel.get_matching_group(CardId::is_negatable_monster, tp, 0, LOCATION_MZONE, Some(sg));
    for dis_c in g {
        dis_c.negate_effects(duel, c, 0, false);
    }
    Ok(())
}

fn penop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.is_relate_to_effect(duel, a.e) {
        duel.move_to_field(c, tp, tp, LOCATION_PZONE, POS_FACEUP, true)?;
    }
    Ok(())
}
