// Atlantis Call
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 65785782;

pub const LISTED_NAMES: &[u32] = &[CARD_ATLANTIS_CITY_OF_THE_SEA_DRAGON, CARD_UMI];

// In the hand and the Graveyard, with Atlantis monsters in the Deck to search, and Umi in the hand to activate and a WATER monster
// to boost, and Atlantis, City of the Sea Dragon in the hand and a random zone for the negation.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (ID, GRAVE),
    (37092104, DECK),
    (37092104, DECK),
    (63481713, DECK),
    (22702055, HAND),
    (22702055, HAND),
    (37092104, MZONE),
    (CARD_ATLANTIS_CITY_OF_THE_SEA_DRAGON, HAND),
    (CARD_ATLANTIS_CITY_OF_THE_SEA_DRAGON, 0),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_DISABLE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_DEFCHANGE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_code(duel, CARD_UMI)),
            a.tp,
            LOCATION_ONFIELD,
            0,
            1,
            None,
        )
    });
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, |duel, a, chk, _| {
        if chk == 0 {
            return Ok(duel.is_existing_matching_card(waterfilter, a.tp, LOCATION_MZONE, 0, 1, None));
        }
        duel.set_operation_info(0, CATEGORY_ATKCHANGE, None, 1, a.tp, 500);
        duel.set_operation_info(0, CATEGORY_DEFCHANGE, None, 1, a.tp, 500);
        Ok(false)
    });
    e2.set_operation(duel, |duel, a| {
        let c = a.e.get_handler(duel);
        for tc in duel.get_matching_group(waterfilter, a.tp, LOCATION_MZONE, 0, None) {
            tc.update_attack(duel, 500, None, Some(c), None);
            tc.update_defense(duel, 500, None, Some(c), None);
        }
        Ok(())
    });
    c.register_effect(duel, e2);
}

fn waterfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_attribute(duel, ATTRIBUTE_WATER)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.lists_code(duel, CARD_ATLANTIS_CITY_OF_THE_SEA_DRAGON) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn negfilter(c: CardId, duel: &Duel) -> bool {
    c.is_effect_monster(duel) && c.is_negatable_monster(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_DISABLE, None, 1, 1 - tp, LOCATION_MZONE as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() && duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? > 0 {
        duel.confirm_cards(1 - tp, g)?;
        duel.shuffle_hand(tp)?;
        duel.shuffle_deck(tp)?;
        if duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_code(duel, CARD_ATLANTIS_CITY_OF_THE_SEA_DRAGON)),
            tp,
            LOCATION_ONFIELD,
            0,
            1,
            None,
        ) && duel.is_existing_matching_card(negfilter, tp, 0, LOCATION_MZONE, 1, None)
            && duel.select_yes_no(tp, aux::stringid(ID, 2))?
        {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
            let Some(sc) = duel
                .select_matching_card(tp, negfilter, tp, 0, LOCATION_MZONE, 1, 1, None)?
                .get_first()
            else {
                return Ok(());
            };
            duel.hint_selection(sc);
            duel.break_effect()?;
            sc.negate_effects(duel, a.e.get_handler(duel), RESET_PHASE | PHASE_END, false);
        }
    }
    Ok(())
}
