// Number F0: Utopic Future Slash
use crate::prelude::*;
use crate::testing::{EXTRA, GRAVE, MZONE};

const ID: u32 = 43490025;

pub const LISTED_SERIES: &[u16] = &[SET_NUMBER, SET_UTOPIA];
pub const XYZ_NUMBER: u32 = 0;
const UTOPIC_FUTURE: u32 = 65305468;

pub const LISTED_NAMES: &[u32] = &[UTOPIC_FUTURE];

// In the Extra Deck, Xyz Summoned over Number F0: Utopic Future on the field, with a Number Xyz Monster in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (UTOPIC_FUTURE, MZONE),
    (UTOPIC_FUTURE, MZONE),
    (63746411, GRAVE),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(xyzfilter);
    const LV: Option<u32> = None;
    const CT: usize = 2;
    const ALTERF: Option<AlterFilter> = Some(ovfilter);
    const DESC: u64 = aux::stringid(ID, 0);

    fn exchk(duel: &Duel, g: CardSet, _: u8, _: CardId) -> bool {
        let mg = g.filter(duel, |c, duel| c.is_has_effect(duel, EFFECT_EQUIP_SPELL_XYZ_MAT).is_none(), None);
        mg.get_class_count(duel, |c, duel| i64::from(c.get_rank(duel))) == 1
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(
        duel,
        Value::Card(|duel, _, _| {
            let numbers = |c: CardId, duel: &Duel| c.is_set_card(duel, &[SET_NUMBER]) && c.is_type(duel, TYPE_XYZ);
            duel.get_matching_group_count(numbers, 0, LOCATION_GRAVE, LOCATION_GRAVE, None) as i32 * 500
        }),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, _| duel.is_able_to_enter_bp());
    e3.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e3.set_target(duel, secondatktg);
    e3.set_operation(duel, secondatkop);
    c.register_effect(duel, e3);
}

fn xyzfilter(c: CardId, duel: &Duel, xyz: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_type_full(duel, TYPE_XYZ, Some(xyz), sumtype, tp) && !c.is_set_card_full(duel, &[SET_NUMBER], Some(xyz), sumtype, tp)
}

fn ovfilter(c: CardId, duel: &Duel, tp: u8, xyzc: CardId) -> bool {
    (c.is_set_card_full(duel, &[SET_UTOPIA], Some(xyzc), SUMMON_TYPE_XYZ as u64, tp)
        || c.is_summon_code(duel, Some(xyzc), SUMMON_TYPE_XYZ as u64, tp, &[UTOPIC_FUTURE]))
        && c.is_faceup(duel)
}

fn secondatktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    Ok(chk == 0 && c.can_attack(duel) && c.get_effect_count(duel, EFFECT_EXTRA_ATTACK) == 0)
}

fn secondatkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3201);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_EXTRA_ATTACK);
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}
