// Spirit of Yubel
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 90829280;

pub const LISTED_NAMES: &[u32] = &[CARD_YUBEL];

// On the field and in the hand, with Yubel in the Deck to Special Summon, the opponent's Dark Hole to destroy it and Sangans to attack.
// No ported Spell/Trap mentions Yubel, so the search is only reported.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, HAND),
    (CARD_YUBEL, DECK),
    (CARD_YUBEL, DECK),
    (53129443, HAND | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 2))),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_AVOID_BATTLE_DAMAGE);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 1));
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e4.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e4.set_range(duel, LOCATION_HAND);
    e4.set_count_limit(duel, 1, ID);
    e4.set_condition(duel, |duel, a| {
        duel.get_attacker().is_some_and(|at| at.is_controler(duel, 1 - a.tp))
    });
    e4.set_target(duel, stock::special_summon_self_target_player_0);
    e4.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 2));
    e5.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SET);
    e5.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e5.set_property(duel, EFFECT_FLAG_DELAY);
    e5.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e5.set_count_limit(duel, 1, (ID, 1));
    e5.set_target(duel, thtg);
    e5.set_operation(duel, thop);
    c.register_effect(duel, e5);
}

const SOURCES: u32 = LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE | LOCATION_REMOVED;

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, CARD_YUBEL)
        && (c.is_faceup(duel) || c.is_location(duel, LOCATION_HAND | LOCATION_DECK))
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), SOURCES)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, SOURCES, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel) && c.lists_code(duel, CARD_YUBEL) && (c.is_able_to_hand(duel) || c.is_ssetable(duel))
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 3));
    let Some(tc) = duel
        .select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    aux::to_hand_or_else(
        duel,
        tc,
        tp,
        CardId::is_ssetable,
        |duel| {
            duel.sset(tp, tc)?;
            Ok(())
        },
        aux::stringid(ID, 4),
    )
}
