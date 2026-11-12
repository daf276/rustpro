// Underground Arachnid
use crate::prelude::*;
use crate::testing::{Check, MZONE, OPPONENT};

const ID: u32 = 63465535;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 1;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_attribute_full(duel, ATTRIBUTE_DARK, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, |c, duel| c.is_race_full(duel, RACE_INSECT, Some(sc), sumtype, tp))
    }
}

// Two on the field, with Psychic Kappa to equip and Elfnote Lucina on the opponent's side to destroy it in battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (7892180, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
// The destruction replacement needs it destroyed in battle while equipped, too rare to assert.
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Report(&Check::Replaced)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 0, 1);
    e1.set_condition(duel, |duel, a| duel.get_attacker() == Some(a.e.get_handler(duel)));
    e1.set_value(
        duel,
        Value::EffectPlayer(|duel, _, re, _| re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_EQUIP);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, eqcon);
    e2.set_target(duel, eqtg);
    e2.set_operation(duel, eqop);
    c.register_effect(duel, e2);
    aux::add_er_equip_limit(duel, c, Some(eqcon), e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e3.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e3.set_code(duel, EFFECT_DESTROY_REPLACE);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target(duel, desreptg);
    e3.set_operation(duel, desrepop);
    c.register_effect(duel, e3);
}

fn eqcon(duel: &Duel, a: &Args) -> bool {
    !a.e.get_handler(duel)
        .get_equip_group(duel)
        .is_exists(duel, |c, duel| c.has_flag_effect(duel, ID), 1, None)
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE)
            && chkc.is_controler(duel, 1 - tp)
            && chkc.is_able_to_change_controler(duel)
            && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_target(
                aux::faceup_filter(CardId::is_able_to_change_controler),
                tp,
                0,
                LOCATION_MZONE,
                1,
                None,
            ));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    let g = duel.select_target(
        tp,
        aux::faceup_filter(CardId::is_able_to_change_controler),
        tp,
        0,
        LOCATION_MZONE,
        1,
        1,
        None,
    )?;
    duel.set_operation_info(0, CATEGORY_EQUIP, g, 1, tp, 0);
    Ok(false)
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && tc.is_controler(duel, 1 - tp) && tc.is_faceup(duel) && tc.is_monster(duel) {
        e.get_handler(duel)
            .equip_by_effect_and_limit_register(duel, e, tp, tc, Some(ID), None)?;
    }
    Ok(())
}

fn desrepfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.has_flag_effect(duel, ID) && c.is_destructable_full(duel, e) && !c.is_status(duel, STATUS_DESTROY_CONFIRMED)
}

fn desreptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if chk == 0 {
        return Ok(c.is_reason(duel, REASON_BATTLE) && c.get_equip_group(duel).is_exists(duel, |c, duel| desrepfilter(c, duel, e), 1, None));
    }
    duel.select_effect_yes_no_full(a.tp, c, 96)
}

fn desrepop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let ec = e
        .get_handler(duel)
        .get_equip_group(duel)
        .filter(duel, |c, duel| desrepfilter(c, duel, e), None);
    duel.destroy(ec, REASON_EFFECT | REASON_REPLACE)?;
    Ok(())
}
