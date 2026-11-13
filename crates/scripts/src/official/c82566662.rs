// Taotie Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 82566662;
const LINK_DEVOTEE: u32 = 48068378;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, _: CardId, _: u64, _: u8) -> bool {
        c.is_summon_location(duel, LOCATION_EXTRA)
    }
}

// In the Extra Deck, Link Summoned from two Link Devotees, which are themselves Link Summoned from Protrons.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (LINK_DEVOTEE, EXTRA),
    (LINK_DEVOTEE, EXTRA),
    (LINK_DEVOTEE, EXTRA),
    (24154052, MZONE),
    (24154052, MZONE),
    (24154052, MZONE),
    (24154052, MZONE),
];
// The Fusion, Synchro and Xyz effects need materials of those types, which no scenario provides.
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
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
    let typ = c.get_material(duel).iter().fold(0, |typ, tc| typ | tc.get_type(duel));
    e.set_label(duel, i64::from(typ & (TYPE_FUSION + TYPE_SYNCHRO + TYPE_XYZ)));
}

fn regcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_link_summoned(duel) && a.e.get_label_object(duel).is_some_and(|e1| e1.get_label(duel) != 0)
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let typ = a.e.get_label_object(duel).map_or(0, |e1| e1.get_label(duel)) as u32;
    if typ & TYPE_FUSION != 0 {
        register_limit(duel, c, fuscond, |duel, _, re, _| re.is_monster_effect(duel), 1);
    }
    if typ & TYPE_SYNCHRO != 0 {
        register_limit(duel, c, syncond, |duel, _, re, _| re.is_spell_trap_effect(duel), 2);
    }
    if typ & TYPE_XYZ != 0 {
        register_limit(
            duel,
            c,
            xyzcond,
            |duel, _, re, _| re.get_activate_location(duel) == LOCATION_GRAVE,
            3,
        );
    }
    Ok(())
}

fn register_limit(
    duel: &mut Duel,
    c: CardId,
    condition: fn(&Duel, &Args) -> bool,
    value: fn(&Duel, EffectId, EffectId, u8) -> bool,
    string: u64,
) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 0, 1);
    e1.set_condition(duel, condition);
    e1.set_value(duel, Value::EffectPlayer(value));
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    c.register_effect(duel, e1);
    c.register_flag_effect_full(
        duel,
        0,
        RESET_EVENT | RESETS_STANDARD,
        EFFECT_FLAG_CLIENT_HINT,
        1,
        0,
        aux::stringid(ID, string),
    );
}

fn fuscond(duel: &Duel, _: &Args) -> bool {
    duel.is_battle_phase()
}

fn syncond(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(a.e.get_handler_player(duel)) && duel.is_main_phase()
}

fn xyzcond(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(a.e.get_handler_player(duel)) && (duel.is_battle_phase() || duel.is_main_phase())
}
