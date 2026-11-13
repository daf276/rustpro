// Master Spirit Tech Force - Pendulum Ruler
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT, PZONE};

const ID: u32 = 68508433;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(matfilter),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_type_full(duel, TYPE_PENDULUM, Some(fc), sumtype, tp)),
    ];
}

// In the Extra Deck, Fusion Summoned by Polymerization from Junoldo and Jupredo; in the Pendulum Zone and on the field, with a Level 7
// Pendulum Monster (Astrograph Sorcerer) in the Deck to add and the opponent's Sangan to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, PZONE),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (10266279, MZONE),
    (43871165, MZONE),
    (76794549, DECK),
    (26202165, MZONE | OPPONENT),
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
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, textg);
    e1.set_operation(duel, texop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, 1);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_code(duel, EFFECT_CANNOT_ATTACK);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_target_card(duel, |duel, _, c| !(c.is_pendulum_monster(duel) && !c.has_level(duel)));
    e3.set_target_range(duel, 0, LOCATION_MZONE);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_DESTROY + CATEGORY_DAMAGE);
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 2, (ID, 1));
    e4.set_cost(duel, descost);
    e4.set_target(duel, destg);
    e4.set_operation(duel, desop);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 2));
    e5.set_type(duel, EFFECT_TYPE_IGNITION);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_count_limit(duel, 1, 0);
    e5.set_target(duel, |duel, a, chk, _| Ok(chk == 0 && duel.check_pendulum_zones(a.tp)));
    e5.set_operation(duel, plop);
    c.register_effect(duel, e5);
}

fn matfilter(c: CardId, duel: &Duel, fc: CardId, sumtype: u64, tp: u8) -> bool {
    c.is_level_above(duel, 7) && c.is_type_full(duel, TYPE_PENDULUM, Some(fc), sumtype, tp)
}

fn textg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_extra(duel));
    }
    duel.set_operation_info(0, CATEGORY_TOEXTRA, Some(c.into()), 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, (LOCATION_DECK | LOCATION_EXTRA) as i32);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level(duel, &[7])
        && c.is_pendulum_monster(duel)
        && (c.is_location(duel, LOCATION_DECK) || c.is_faceup(duel))
        && c.is_able_to_hand(duel)
}

fn texop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e)
        && duel.send_to_deck(c, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? > 0
        && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK | LOCATION_EXTRA, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 3))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK | LOCATION_EXTRA, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.break_effect()?;
            duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, g)?;
        }
    }
    Ok(())
}

fn descostfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    let exg = c.get_equip_group(duel).union(c.into());
    duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, Some(exg))
}

fn descost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, |c, duel| descostfilter(c, duel, tp), (1, 1), false, None, None));
    }
    let g = duel.select_release_group_cost(tp, |c, duel| descostfilter(c, duel, tp), (1, 1), false, None, None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, 1600);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
        && duel.destroy(tc, REASON_EFFECT)? > 0
    {
        duel.damage(1 - a.tp, 1600, REASON_EFFECT)?;
    }
    Ok(())
}

fn plop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.is_relate_to_effect(duel, a.e) {
        duel.move_to_field(c, tp, tp, LOCATION_PZONE, POS_FACEUP, true)?;
    }
    Ok(())
}
