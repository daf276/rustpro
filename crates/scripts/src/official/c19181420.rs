// Mementotlan Twin Dragon
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, OPPONENT};

const ID: u32 = 19181420;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] =
        &[FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_set_card_full(duel, &[SET_MEMENTO], Some(fc), sumtype, tp)); 2];
}

// In the Extra Deck, Fusion Summoned by Polymerization with two Mementotlan monsters, with Mementotlan monsters in the Deck
// and Graveyard, and destroyed by the opponent's Raigeki.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (24094653, HAND),
    (24094653, HAND),
    (54550967, HAND),
    (54550967, HAND),
    (81677154, HAND),
    (81677154, HAND),
    (54550967, DECK),
    (81677154, DECK),
    (54550967, GRAVE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_code(duel, EFFECT_BATTLE_DESTROY_REDIRECT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_value(duel, LOCATION_REMOVED as i32);
    e1.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_MEMENTO]));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_SEARCH + CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_fusion_summoned(duel));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, spcon);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn desfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MEMENTO]) && c.is_monster(duel) && (c.is_faceup(duel) || c.is_location(duel, LOCATION_HAND))
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MEMENTO]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(desfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, None)
                && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None),
        );
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, None, 1, tp, (LOCATION_HAND | LOCATION_MZONE) as i32);
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 2, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let desg = duel.select_matching_card(tp, desfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, 1, None)?;
    if duel.destroy(desg, REASON_EFFECT)? == 0 {
        return Ok(());
    }
    let g = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
    if g.is_empty() {
        return Ok(());
    }
    let sg = aux::select_unselect_group(duel, g, 1, 2, &aux::dncheck, tp, HINTMSG_ATOHAND)?;
    if !sg.is_empty() {
        duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, sg)?;
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_previous_location(duel, LOCATION_MZONE) && c.is_fusion_summoned(duel)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 6) && c.is_set_card(duel, &[SET_MEMENTO]) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_GRAVE)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_GRAVE, POS_FACEUP)
}
