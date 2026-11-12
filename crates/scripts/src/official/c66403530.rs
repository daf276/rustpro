// Topologic Zeroboros
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 66403530;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }
}

// In the Extra Deck, Link Summoned with two Sangans (Effect Monsters); Special Summons of the opponent's Sangans trigger the banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Link::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_FORCE_MZONE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_property(
        duel,
        EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_CANNOT_DISABLE,
    );
    e1.set_target_range(duel, 1, 0);
    e1.set_value(
        duel,
        Value::Zones(|duel, e, _, _, _| !((e.get_handler(duel).get_linked_zone(duel) & ZONES_EMZ) as i32)),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_value(
        duel,
        Value::Card(|duel, _, c| duel.get_field_group_count(c.get_controler(duel), LOCATION_REMOVED, LOCATION_REMOVED) as i32 * 200),
    );
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_REMOVE);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, rmcon);
    e3.set_target(duel, rmtg);
    e3.set_operation(duel, rmop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
    e4.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e4.set_code(duel, EVENT_REMOVE);
    e4.set_operation(duel, spreg);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 1));
    e5.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e5.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e5.set_range(duel, LOCATION_REMOVED);
    e5.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e5.set_condition(duel, spcon);
    e5.set_target(duel, sptg);
    e5.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    e5.set_label_object(duel, Some(e4));
    c.register_effect(duel, e5);
}

fn rmcon(duel: &Duel, a: &Args) -> bool {
    let eg = a.eg.unwrap_or_default();
    if eg.is_contains(a.e.get_handler(duel)) {
        return false;
    }
    let g = duel.get_matching_group(|c, duel| c.is_type(duel, TYPE_LINK), a.tp, LOCATION_MZONE, LOCATION_MZONE, None);
    g.iter().any(|tc| aux::zptgroupcon(duel, eg, aux::TRUE, tc))
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let g = duel.get_matching_group(CardId::is_able_to_remove, a.tp, LOCATION_ONFIELD, LOCATION_ONFIELD, None);
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_field_group(a.tp, LOCATION_ONFIELD, LOCATION_ONFIELD);
    duel.remove(g, POS_FACEUP, REASON_EFFECT)?;
    Ok(())
}

fn spreg(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if a.re.is_some_and(|re| re.get_handler(duel) == c) {
        let turn = i64::from(duel.get_turn_count());
        a.e.set_label(duel, turn);
        c.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 2);
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    a.e.get_label_object(duel)
        .is_some_and(|e| e.get_label(duel) != i64::from(duel.get_turn_count()))
        && c.get_flag_effect(duel, ID) > 0
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let c = a.e.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    c.reset_flag_effect(duel, ID);
    Ok(false)
}
