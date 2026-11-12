// Genex Ally Triforce
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 52709508;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_set_card_full(duel, &[SET_GENEX], Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// In the Extra Deck, Synchro Summoned from Genex Controller (a Level 3 Tuner) and a FIRE or a LIGHT Level 4 non-Tuner (Blazing
// Inpachi, Mystical Elf), against the opponent's Psychic Kappa, with a LIGHT monster in the Graveyard to revive.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (68505803, MZONE),
    (5464695, MZONE),
    (15025844, MZONE),
    (15025844, MZONE),
    (15025844, GRAVE),
    (15025844, GRAVE),
    (7892180, MZONE | OPPONENT),
    (7892180, MZONE | OPPONENT),
    (7892180, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_MATERIAL_CHECK);
    e1.set_value(duel, Value::MaterialCheck(valcheck));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_condition(duel, regcon);
    e2.set_operation(duel, regop);
    c.register_effect(duel, e2);
    e2.set_label_object(duel, Some(e1));
}

fn valcheck(duel: &mut Duel, e: EffectId, c: CardId) {
    let att = c.get_material(duel).get_bitwise_or(duel, |mc, duel| mc.get_attribute(duel).into())
        & u64::from(ATTRIBUTE_EARTH | ATTRIBUTE_FIRE | ATTRIBUTE_LIGHT);
    e.set_label(duel, att as i64);
}

fn regcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_synchro_summoned(duel) && a.e.get_label_object(duel).is_some_and(|e1| e1.get_label(duel) != 0)
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let att = a.e.get_label_object(duel).map_or(0, |e1| e1.get_label(duel)) as u32;
    let c = a.e.get_handler(duel);
    if att & ATTRIBUTE_EARTH != 0 {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
        e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_target_range(duel, 0, 1);
        e1.set_value(
            duel,
            Value::EffectPlayer(|duel, _, re, _| re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)),
        );
        e1.set_condition(duel, |duel, a| duel.get_attacker() == Some(a.e.get_handler(duel)));
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e1);
        c.register_flag_effect_full(
            duel,
            0,
            RESET_EVENT | RESETS_STANDARD,
            EFFECT_FLAG_CLIENT_HINT,
            1,
            0,
            aux::stringid(ID, 2),
        );
    }
    if att & ATTRIBUTE_FIRE != 0 {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, aux::stringid(ID, 0));
        e1.set_category(duel, CATEGORY_DAMAGE);
        e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
        e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
        e1.set_code(duel, EVENT_BATTLE_DESTROYING);
        e1.set_condition(duel, aux::bdcon);
        e1.set_target(duel, damtg);
        e1.set_operation(duel, stock::damage);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e1);
        c.register_flag_effect_full(
            duel,
            0,
            RESET_EVENT | RESETS_STANDARD,
            EFFECT_FLAG_CLIENT_HINT,
            1,
            0,
            aux::stringid(ID, 3),
        );
    }
    if att & ATTRIBUTE_LIGHT != 0 {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, aux::stringid(ID, 1));
        e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_SET);
        e1.set_type(duel, EFFECT_TYPE_IGNITION);
        e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_count_limit(duel, 1, 0);
        e1.set_target(duel, sptg);
        e1.set_operation(duel, spop);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e1);
        c.register_flag_effect_full(
            duel,
            0,
            RESET_EVENT | RESETS_STANDARD,
            EFFECT_FLAG_CLIENT_HINT,
            1,
            0,
            aux::stringid(ID, 4),
        );
    }
    Ok(())
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    let dam =
        a.e.get_handler(duel)
            .get_battle_target(duel)
            .map_or(0, |bc| bc.get_base_attack(duel).max(0)) as u32;
    duel.set_target_player(1 - tp);
    duel.set_target_param(dam);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, dam as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_attribute(duel, ATTRIBUTE_LIGHT) && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEDOWN_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.special_summon(tc, 0, a.tp, a.tp, false, false, POS_FACEDOWN_DEFENSE)?;
        duel.confirm_cards(1 - a.tp, tc)?;
    }
    Ok(())
}
