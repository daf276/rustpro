// Dogmatika Lawbringer
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 30394645;

pub const LISTED_SERIES: &[u16] = &[SET_DOGMATIKA];
pub const LISTED_NAMES: &[u32] = &[ID];
// In the hand with Extra Deck monsters on the field (Battle Shogun) to summon it, other Dogmatika cards in the Graveyard to
// add, and copies in the Extra Decks to send.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (74752631, MZONE),
    (74752631, MZONE | OPPONENT),
    (74752631, EXTRA),
    (74752631, EXTRA | OPPONENT),
    (60303688, GRAVE),
    (82956214, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, _| {
        duel.is_existing_matching_card(
            |c, duel| c.is_summon_location(duel, LOCATION_EXTRA),
            0,
            LOCATION_MZONE,
            LOCATION_MZONE,
            1,
            None,
        )
    });
    e1.set_target(duel, stock::special_summon_self_target);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOGRAVE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_summon_location(duel, LOCATION_HAND));
    e2.set_target(duel, tgtg);
    e2.set_operation(duel, tgop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 2));
    e3.set_target(duel, thtg);
    e3.set_operation(duel, stock::first_target_to_hand);
    c.register_effect(duel, e3);
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(CardId::is_able_to_grave, tp, LOCATION_EXTRA, 0, 1, None)
                && duel.is_existing_matching_card(CardId::is_able_to_grave, tp, 0, LOCATION_EXTRA, 1, None),
        );
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 2, PLAYER_ALL, LOCATION_EXTRA as i32);
    Ok(false)
}

fn tgop(duel: &mut Duel, _: &Args) -> Script<()> {
    let mut sg = CardSet::default();
    for p in 0..2 {
        duel.hint(HINT_SELECTMSG, p, HINTMSG_TOGRAVE);
        if let Some(sc) = duel
            .select_matching_card(p, CardId::is_able_to_grave, p, LOCATION_EXTRA, 0, 1, 1, None)?
            .get_first()
        {
            sg.insert(sc);
        }
    }
    if !sg.is_empty() {
        duel.send_to_grave(sg, REASON_EFFECT)?;
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DOGMATIKA]) && c.is_able_to_hand(duel) && !c.is_code(duel, ID)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, tp, 0);
    Ok(false)
}
