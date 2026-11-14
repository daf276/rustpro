// World Gears of Theurlogical Demiurgy
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE, OPPONENT};

const ID: u32 = 57282724;

pub const LISTED_SERIES: &[u16] = &[SET_WORLD_LEGACY];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 3;
    const MAX: Option<usize> = Some(3);

    fn f(c: CardId, duel: &Duel, _: CardId, _: u64, _: u8) -> bool {
        c.is_level_above(duel, 5)
    }
}

// Level 5+ monsters of different Races and Attributes to Link Summon with, World Legacy - "World Crown" and "World Wand" in the Deck
// to summon, and the opponent's Extra Deck Summons.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (ID, EXTRA),
    (27918365, DECK),
    (93920420, DECK),
    (27918365, DECK),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_CANNOT_BE_LINK_MATERIAL);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e2.set_value(duel, Value::Effect(efilter));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_SINGLE);
    e3.set_code(duel, EFFECT_MATERIAL_CHECK);
    e3.set_value(duel, Value::MaterialCheck(matcheck));
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_category(duel, CATEGORY_DESTROY);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, ID);
    e4.set_condition(duel, |duel, a| a.e.get_handler(duel).get_flag_effect(duel, ID) != 0);
    e4.set_target(duel, destg);
    e4.set_operation(duel, desop);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e5.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e5.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e5.set_property(duel, EFFECT_FLAG_DELAY);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_count_limit(duel, 1, (ID, 1));
    e5.set_condition(duel, spcon);
    e5.set_target(duel, sptg);
    e5.set_operation(duel, spop);
    c.register_effect(duel, e5);
}

fn efilter(duel: &Duel, e: EffectId, re: EffectId) -> bool {
    re.is_monster_effect(duel) && re.get_owner(duel) != e.get_owner(duel)
}

fn matcheck(duel: &mut Duel, _: EffectId, c: CardId) {
    let g = c.get_material(duel);
    let tp = c.get_controler(duel);
    let sumtype = SUMMON_TYPE_LINK as u64;
    if g.get_class_count(duel, |mc, duel| mc.get_race_full(duel, Some(c), sumtype, tp) as i64) == 3
        && g.get_class_count(duel, |mc, duel| mc.get_attribute_full(duel, Some(c), sumtype, tp) as i64) == 3
    {
        c.register_flag_effect_full(
            duel,
            ID,
            RESET_EVENT | (RESETS_STANDARD & !(RESET_TOFIELD | RESET_LEAVE | RESET_TEMP_REMOVE)),
            EFFECT_FLAG_CLIENT_HINT,
            1,
            0,
            aux::stringid(ID, 0),
        );
    }
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(
        aux::TRUE,
        tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        Some(a.e.get_handler(duel).into()),
    );
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(
        aux::TRUE,
        a.tp,
        LOCATION_ONFIELD,
        LOCATION_ONFIELD,
        Some(a.e.get_handler(duel).into()),
    );
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}

fn spcfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_summon_location(duel, LOCATION_EXTRA) && !c.is_summon_player(duel, tp)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    a.eg.unwrap_or_default()
        .is_exists(duel, |c, duel| spcfilter(c, duel, a.tp), 1, None)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_WORLD_LEGACY]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK, POS_FACEUP)
}
