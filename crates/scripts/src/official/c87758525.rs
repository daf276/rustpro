// Favorite HERO Shining Flare Wingman
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 87758525;

pub const LISTED_SERIES: &[u16] = &[SET_ELEMENTAL_HERO];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_type_full(duel, TYPE_FUSION, Some(fc), sumtype, tp)),
        FusionMaterial::Filter(|c, duel, _, _, _| matfilter(c, duel)),
    ];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Elemental HERO Flame Wingman (a Fusion Monster) and Avian, with
// monsters in the Graveyard to shuffle and weak opponent's monsters to destroy by battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (24094653, HAND),
    (24094653, HAND),
    (35809262, MZONE),
    (21844576, MZONE),
    (21844576, HAND),
    (21844576, GRAVE),
    (21844576, GRAVE),
    (21844576, GRAVE),
    (21844576, GRAVE),
    (21844576, GRAVE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TODECK | CATEGORY_DRAW | CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE | EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY | EFFECT_FLAG_PLAYER_TARGET);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_summon_location(duel, LOCATION_EXTRA));
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DAMAGE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EVENT_BATTLE_DESTROYING);
    e2.set_condition(duel, aux::bdcon);
    e2.set_target(duel, damtg);
    e2.set_operation(duel, stock::damage);
    c.register_effect(duel, e2);
}

fn matfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_on_field(duel)
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_able_to_deck(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tdfilter, tp, LOCATION_GRAVE, 0, 5, None) && duel.is_player_can_draw(tp, 2));
    }
    duel.set_target_player(tp);
    duel.set_target_param(2);
    duel.set_operation_info(0, CATEGORY_TODECK, None, 5, tp, LOCATION_GRAVE as i32);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 2);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, tdfilter, tp, LOCATION_GRAVE, 0, 5, 5, None)?;
    if g.len() != 5 {
        return Ok(());
    }
    duel.hint_selection(g);
    if duel.send_to_deck(g, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? == 5 {
        if duel
            .get_operated_group()
            .is_exists(duel, |tc, duel| tc.is_location(duel, LOCATION_DECK), 1, None)
        {
            duel.shuffle_deck(tp)?;
        }
        duel.break_effect()?;
        if duel.draw(p, d, REASON_EFFECT)? == 2 && c.is_relate_to_effect(duel, a.e) && c.is_faceup(duel) {
            c.update_attack(duel, 1000, None, None, None);
        }
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
            .map_or(0, |bc| bc.get_base_attack(duel))
            .max(0);
    duel.set_target_player(1 - tp);
    duel.set_target_param(dam as u32);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, dam);
    Ok(false)
}
