// Starving Venom Wing Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT, PZONE};

const ID: u32 = 5148778;

pub const LISTED_SERIES: &[u16] = &[SET_CLEAR_WING];
pub const COUNTER_PLACE_LIST: &[u16] = &[COUNTER_PREDATOR];

pub const MATERIAL_SETCODE: &[u16] = &[SET_CLEAR_WING];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, _, _, _| c.is_type(duel, TYPE_FUSION)),
        FusionMaterial::Filter(|c, duel, _, _, _| c.is_set_card(duel, &[SET_CLEAR_WING])),
    ];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Alligator's Sword Dragon and Clear Wing Synchro Dragon; in the Pendulum Zone and
// on the field, with Sangan and Elfnote Lucina to be Special Summoned by the opponent, and Dark Hole to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, PZONE),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (3366982, MZONE),
    (82044279, MZONE),
    (26202165, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Pendulum::add_procedure_full(duel, c, false, None);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_operation(duel, counterplaceop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DISABLE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, atknegeffcon);
    e2.set_target(duel, atknegefftg);
    e2.set_operation(duel, atknegeffop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_condition(duel, |duel, a| {
        let loc: u32 = duel.get_chain_info(a.ev as u8, CHAININFO_TRIGGERING_LOCATION);
        a.re.is_some_and(|re| re.get_handler(duel) != a.e.get_handler(duel)) && loc & LOCATION_ONFIELD > 0
    });
    e3.set_target(duel, destg);
    e3.set_operation(duel, desop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_DELAY);
    e4.set_code(duel, EVENT_DESTROYED);
    e4.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_MZONE));
    e4.set_target(duel, pentg);
    e4.set_operation(duel, penop);
    c.register_effect(duel, e4);
}

fn counterplaceop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    let g = a.eg.unwrap_or_default().filter(duel, |c, duel| c.is_controler(duel, 1 - tp), None);
    for tc in g {
        if tc.add_counter(duel, COUNTER_PREDATOR, 1) && tc.is_level_above(duel, 2) {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
            e1.set_code(duel, EFFECT_CHANGE_LEVEL);
            e1.set_condition(duel, |duel, a| a.e.get_handler(duel).has_counter(duel, COUNTER_PREDATOR));
            e1.set_value(duel, 1);
            e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            tc.register_effect(duel, e1);
        }
    }
    Ok(())
}

fn atknegefffilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_summon_player(duel, 1 - tp)
        && c.is_can_be_effect_target(duel, e)
        && c.is_location(duel, LOCATION_MZONE)
        && c.has_non_zero_attack(duel)
}

fn atknegeffcon(duel: &Duel, a: &Args) -> bool {
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| atknegefffilter(c, duel, a.e, a.tp), 1, None))
}

fn atknegefftg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let eg = a.eg.unwrap_or_default();
    if let Some(chkc) = chkc {
        return Ok(eg.contains(chkc) && atknegefffilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(eg.is_exists(duel, |c, duel| atknegefffilter(c, duel, e, tp), 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let tc = if eg.len() == 1 {
        eg
    } else {
        eg.filter_select(duel, tp, |c, duel| atknegefffilter(c, duel, e, tp), 1, 1, None)?
    };
    duel.set_target_card(tc);
    duel.set_operation_info(0, CATEGORY_DISABLE, Some(tc), 1, 0, 0);
    Ok(false)
}

fn atknegeffop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, e) = (a.e.get_handler(duel), a.e);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if c.is_relate_to_effect(duel, e)
        && c.is_faceup(duel)
        && tc.is_relate_to_effect(duel, e)
        && tc.is_faceup(duel)
        && c.update_attack(duel, tc.get_attack(duel), Some(RESETS_STANDARD_DISABLE_PHASE_END), None, None) != 0
        && tc.is_negatable_monster(duel)
    {
        tc.negate_effects(duel, c, RESETS_STANDARD_PHASE_END, false);
    }
    Ok(())
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    let g = duel.get_field_group(tp, LOCATION_ONFIELD, LOCATION_ONFIELD);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, tp, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}

fn pentg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.check_pendulum_zones(a.tp));
    }
    let c = a.e.get_handler(duel);
    if c.is_location(duel, LOCATION_GRAVE) {
        duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, Some(c.into()), 1, a.tp, 0);
    }
    Ok(false)
}

fn penop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.is_relate_to_effect(duel, a.e) {
        duel.move_to_field(c, tp, tp, LOCATION_PZONE, POS_FACEUP, true)?;
    }
    Ok(())
}
