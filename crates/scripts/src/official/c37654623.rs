// Hideout in the Sky, Coulomb
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 37654623;
const TOKEN_MECH_BEAST: u32 = ID + 1;

pub const LISTED_NAMES: &[u32] = &[TOKEN_MECH_BEAST];
// Copies in the hand and Graveyard, Surge Blitzclique and Emi Blitzclique to destroy cards and to add, and the opponent's monster.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (22912101, HAND),
    (22912101, MZONE),
    (11895663, DECK),
    (11895663, DECK),
    (9430387, HAND),
    (15025844, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1a = duel.create_effect(c);
    e1a.set_type(duel, EFFECT_TYPE_FIELD);
    e1a.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1a.set_range(duel, LOCATION_FZONE);
    e1a.set_target_range(duel, LOCATION_MZONE, 0);
    e1a.set_target_card(duel, |duel, _, c| {
        (c.is_normal_summoned(duel) || c.is_special_summoned(duel)) && c.is_summon_location(duel, LOCATION_HAND)
    });
    e1a.set_value(duel, 500);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_code(duel, EFFECT_UPDATE_DEFENSE);
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN + CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_count_limit(duel, 1, (ID, 0));
    e2.set_target(duel, tokentg);
    e2.set_operation(duel, tokenop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, selfthcon);
    e3.set_target(duel, selfthtg);
    e3.set_operation(duel, stock::self_to_hand);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_BLITZCLIQUE]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn can_summon_token(duel: &mut Duel, tp: u8, target_player: u8) -> bool {
    let stats = MonsterStats::new(0, TYPES_TOKEN, 2000, 2000, 6, RACE_MACHINE, ATTRIBUTE_EARTH);
    duel.is_player_can_special_summon_monster_full(tp, TOKEN_MECH_BEAST, stats, POS_FACEUP, target_player, 0)
}

fn tokentg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_location_count(1 - tp, LOCATION_MZONE) > 0
            && can_summon_token(duel, tp, 1 - tp)
            && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn tokenop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (tp, opp) = (a.tp, 1 - a.tp);
    if duel.get_location_count(opp, LOCATION_MZONE) <= 0 || !can_summon_token(duel, tp, opp) {
        return Ok(());
    }
    let Some(token) = duel.create_token(tp, TOKEN_MECH_BEAST) else {
        return Ok(());
    };
    if duel.special_summon(token, 0, tp, opp, false, false, POS_FACEUP)? > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(opp, g)?;
        }
    }
    Ok(())
}

fn selfthcon(duel: &Duel, a: &Args) -> bool {
    let (Some(re), Some(eg)) = (a.re, a.eg) else { return false };
    a.rp == a.tp
        && !eg.is_contains(a.e.get_handler(duel))
        && eg.is_exists(duel, |c, duel| c.is_reason(duel, REASON_EFFECT), 1, None)
        && re.get_handler(duel).is_set_card(duel, &[SET_BLITZCLIQUE])
}

fn selfthtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, a.tp, 0);
    Ok(false)
}
