// Empire of Endymion
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 34041788;

pub const LISTED_NAMES: &[u32] = &[CARD_REGULUS_THE_PRINCE_OF_ENDYMION];
// In the hand and Set, with Regulus in the Deck to add and on the field to be destroyed instead, Dark Magician (a Spellcaster
// Normal Monster) in the hand to Special Summon, and the opponent's Dark Hole and monster to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (ID, SZONE),
    (ID, SZONE),
    (CARD_REGULUS_THE_PRINCE_OF_ENDYMION, DECK),
    (CARD_REGULUS_THE_PRINCE_OF_ENDYMION, MZONE),
    (CARD_REGULUS_THE_PRINCE_OF_ENDYMION, HAND),
    (46986414, HAND),
    (46986414, MZONE),
    (53129443, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
    (7892180, MZONE | OPPONENT),
];
// The destruction it substitutes is counted by Regulus's Replaced check: Check::Replaced counts the move of the card under test.
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, (ID, 0), EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EFFECT_DESTROY_REPLACE);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, desreptg);
    e2.set_operation(duel, desrepop);
    e2.set_value(
        duel,
        Value::Card(|duel, e, c| i32::from(repfilter(c, duel, e.get_handler_player(duel)))),
    );
    c.register_effect(duel, e2);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_code(duel, CARD_REGULUS_THE_PRINCE_OF_ENDYMION)
        || (c.is_monster(duel) && c.lists_code(duel, CARD_REGULUS_THE_PRINCE_OF_ENDYMION)))
        && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_SPELLCASTER) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() && duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? > 0 {
        duel.confirm_cards(1 - tp, g)?;
        duel.shuffle_hand(tp)?;
        if duel.get_field_group_count(tp, 0, LOCATION_MZONE) > 0
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, None)
            && duel.select_yes_no(tp, aux::stringid(ID, 1))?
        {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let sg = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
            if !sg.is_empty() {
                duel.break_effect()?;
                duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
            }
        }
    }
    Ok(())
}

fn repfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_controler(duel, tp)
        && c.is_on_field(duel)
        && c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
        && !c.is_reason(duel, REASON_REPLACE)
}

fn desfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_code(duel, CARD_REGULUS_THE_PRINCE_OF_ENDYMION)
        && (c.is_location(duel, LOCATION_HAND) || c.is_faceup(duel))
        && c.is_destructable_full(duel, e)
        && !c.is_status(duel, STATUS_DESTROY_CONFIRMED | STATUS_BATTLE_DESTROYED)
}

fn desreptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(a.eg.unwrap_or_default().is_exists(duel, |c, duel| repfilter(c, duel, tp), 1, None)
            && duel.is_existing_matching_card(|c, duel| desfilter(c, duel, e), tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, None));
    }
    if !duel.select_effect_yes_no_full(tp, e.get_handler(duel), 96)? {
        return Ok(false);
    }
    duel.hint(HINT_CARD, 0, ID.into());
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESREPLACE);
    let g = duel.select_matching_card(
        tp,
        |c, duel| desfilter(c, duel, e),
        tp,
        LOCATION_HAND | LOCATION_MZONE,
        0,
        1,
        1,
        None,
    )?;
    let Some(sc) = g.get_first() else {
        return Ok(true);
    };
    e.set_label_object_group(duel, Some(sc.into()));
    sc.set_status(duel, STATUS_DESTROY_CONFIRMED, true);
    if sc.is_location(duel, LOCATION_HAND) {
        duel.confirm_cards(1 - tp, sc)?;
    } else {
        duel.hint_selection(sc);
    }
    Ok(true)
}

fn desrepop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(sc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) {
        sc.set_status(duel, STATUS_DESTROY_CONFIRMED, false);
        duel.destroy(sc, REASON_EFFECT | REASON_REPLACE)?;
    }
    Ok(())
}
