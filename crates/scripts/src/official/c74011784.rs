// Artmage Varnish -Alteration-
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 74011784;

pub const LISTED_SERIES: &[u16] = &[SET_ARTMAGE];

const ACROPOLIS: u32 = 74733322;

pub const LISTED_NAMES: &[u32] = &[CARD_MEDIUS_THE_PURE, ACROPOLIS, ID];

// In hand to activate and in the Graveyard to negate the attack of Elfnote Lucina of the opponent on Artmage Litera, with Acropolis
// and Artmage monsters in the Deck and Medius the Pure in hand to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (ACROPOLIS, DECK),
    (ACROPOLIS, DECK),
    (97434754, DECK),
    (97434754, MZONE),
    (97434754, MZONE),
    (CARD_MEDIUS_THE_PURE, HAND),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(CARD_MEDIUS_THE_PURE),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BE_BATTLE_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, negcon);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, negtg);
    e2.set_operation(duel, negop);
    c.register_effect(duel, e2);
}

fn plthfilter(c: CardId, duel: &Duel, tohand_chk: bool) -> bool {
    (c.is_code(duel, ACROPOLIS) && !c.is_forbidden(duel))
        || (tohand_chk
            && c.is_set_card(duel, &[SET_ARTMAGE])
            && c.is_able_to_hand(duel)
            && c.is_location(duel, LOCATION_DECK)
            && !c.is_code(duel, ID))
}

fn acropolis_on_field(duel: &Duel, tp: u8) -> bool {
    duel.is_existing_matching_card(
        aux::faceup_filter(|c, duel| c.is_code(duel, ACROPOLIS)),
        tp,
        LOCATION_ONFIELD,
        0,
        1,
        None,
    )
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let tohand_chk = acropolis_on_field(duel, tp);
        return Ok(duel.is_existing_matching_card(
            |c, duel| plthfilter(c, duel, tohand_chk),
            tp,
            LOCATION_DECK | LOCATION_GRAVE,
            0,
            1,
            None,
        ));
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let tohand_chk = acropolis_on_field(duel, tp);
    let hint_desc = if tohand_chk { aux::stringid(ID, 2) } else { HINTMSG_TOFIELD };
    duel.hint(HINT_SELECTMSG, tp, hint_desc);
    let filter = aux::necro_valley_filter(|c, duel| plthfilter(c, duel, tohand_chk));
    let Some(sc) = duel
        .select_matching_card(tp, filter, tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    if sc.is_code(duel, ACROPOLIS) {
        if !tohand_chk {
            duel.place_field_spell(sc, tp)?;
        } else {
            aux::to_hand_or_else(
                duel,
                sc,
                tp,
                |_, duel| tohand_chk && !sc.is_forbidden(duel),
                |duel| duel.place_field_spell(sc, tp).map(drop),
                aux::stringid(ID, 3),
            )?;
        }
    } else {
        duel.send_to_hand(sc, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, sc)?;
    }
    Ok(())
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    duel.get_attack_target()
        .is_some_and(|bc| bc.is_set_card(duel, &[SET_ARTMAGE]) && bc.is_controler(duel, a.tp) && bc.is_faceup(duel))
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, a.tp, LOCATION_HAND as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, CARD_MEDIUS_THE_PURE) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.negate_attack()?
        && duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 4))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.break_effect()?;
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
