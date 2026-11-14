// Merciless Scorpion of Serket
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 34926568;

pub const LISTED_NAMES: &[u32] = &[CARD_TEMPLE_OF_THE_KINGS];

// Temple of the Kings to control and to add, Uria as the Level 10 monster to banish, Sangan to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (CARD_TEMPLE_OF_THE_KINGS, SZONE),
    (CARD_TEMPLE_OF_THE_KINGS, HAND),
    (CARD_TEMPLE_OF_THE_KINGS, DECK),
    (6007213, DECK),
    (6007213, DECK),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e0 = duel.create_effect(c);
    e0.set_description(duel, aux::stringid(ID, 0));
    e0.set_type(duel, EFFECT_TYPE_FIELD);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_SPSUMMON_PROC);
    e0.set_range(duel, LOCATION_HAND);
    e0.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e0.set_condition(duel, spcon);
    e0.set_target(duel, sptg);
    e0.set_operation(duel, spop);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 2));
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_ATKCHANGE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BATTLE_START);
    e2.set_target(duel, destg);
    e2.set_operation(duel, desop);
    c.register_effect(duel, e2);
}

fn spcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level_above(duel, 10) && c.is_able_to_remove_as_cost(duel)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(
            |c, duel| c.is_faceup(duel) && c.is_code(duel, CARD_TEMPLE_OF_THE_KINGS),
            tp,
            LOCATION_ONFIELD,
            0,
            1,
            None,
        )
        && duel.is_existing_matching_card(spcostfilter, tp, LOCATION_HAND | LOCATION_DECK, 0, 1, Some(c.into()))
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let rg = duel.get_matching_group(spcostfilter, tp, LOCATION_HAND | LOCATION_DECK, 0, a.c.map(CardSet::from));
    let g = aux::select_unselect_group_full(duel, rg, 1, 1, &|_, _, _, _| (true, false), tp, HINTMSG_REMOVE, None, true)?;
    if g.is_empty() {
        return Ok(false);
    }
    a.e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(g) = a.e.get_label_object_group(duel) {
        duel.remove(g, POS_FACEUP, REASON_COST)?;
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_code(duel, CARD_TEMPLE_OF_THE_KINGS) || (c.is_spell(duel) && c.lists_code(duel, CARD_TEMPLE_OF_THE_KINGS)))
        && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let bc = a.e.get_handler(duel).get_battle_target(duel);
    if chk == 0 {
        return Ok(bc.is_some_and(|bc| bc.is_controler(duel, 1 - a.tp)));
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, bc.map(CardSet::from), 1, a.tp, 0);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(bc) = c.get_battle_target(duel) else { return Ok(()) };
    if !(bc.is_relate_to_battle(duel) && bc.is_controler(duel, 1 - a.tp)) {
        return Ok(());
    }
    let atk = f64::from(bc.get_base_attack(duel)) / 2.0;
    if duel.destroy(bc, REASON_EFFECT)? > 0 && atk > 0.0 && c.is_relate_to_effect(duel, a.e) && c.is_faceup(duel) {
        c.update_attack(duel, atk.round() as i32, None, None, None);
    }
    Ok(())
}
