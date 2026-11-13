// Elemental Saga Doriado
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 50208444;

// Copies in the hand and on the field, with Normal Monsters of four Attributes in the Graveyard and EARTH ones (Protron, Doron) in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (27288416, GRAVE),
    (18914778, GRAVE),
    (99171160, GRAVE),
    (13179332, GRAVE),
    (24154052, DECK),
    (24154052, DECK),
    (756652, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, |duel, a| attribute_condition(duel, a.tp));
    e1.set_target(duel, |duel, a, chk, chkc| {
        let (e, tp) = (a.e, a.tp);
        if let Some(chkc) = chkc {
            return Ok(chkc.is_controler(duel, tp)
                && chkc.is_location(duel, LOCATION_GRAVE)
                && chkc.is_can_be_special_summoned(duel, e, 0, tp, false, false));
        }
        let c = e.get_handler(duel);
        if chk == 0 {
            return Ok(duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
                && duel.get_location_count(tp, LOCATION_MZONE) >= 2
                && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
                && duel.is_existing_target(
                    |c, duel| c.is_can_be_special_summoned(duel, e, 0, tp, false, false),
                    tp,
                    LOCATION_GRAVE,
                    0,
                    1,
                    None,
                ));
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_target(
            tp,
            |c, duel| c.is_can_be_special_summoned(duel, e, 0, tp, false, false),
            tp,
            LOCATION_GRAVE,
            0,
            1,
            1,
            None,
        )?;
        let sg = g.unwrap_or_default().union(c.into());
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(sg), 2, tp, 0);
        duel.set_chain_limit(|_, _, _, _, _| false, 0);
        Ok(false)
    });
    e1.set_operation(duel, |duel, a| {
        let (e, tp) = (a.e, a.tp);
        let c = e.get_handler(duel);
        let Some(tc) = duel.get_first_target() else { return Ok(()) };
        if c.is_relate_to_effect(duel, e)
            && tc.is_relate_to_effect(duel, e)
            && duel.get_location_count(tp, LOCATION_MZONE) >= 2
            && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
            && tc.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        {
            duel.special_summon(CardSet::from_iter([c, tc]), 0, tp, tp, false, false, POS_FACEUP)?;
        }
        Ok(())
    });
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| attribute_condition(duel, a.tp));
    e2.set_target(duel, |duel, a, chk, _| {
        let (e, tp) = (a.e, a.tp);
        if chk == 0 {
            return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
                && duel.is_existing_matching_card(|c, duel| deckspfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, None));
        }
        duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_DECK as i32);
        duel.set_chain_limit(|_, _, _, _, _| false, 0);
        Ok(false)
    });
    e2.set_operation(duel, |duel, a| {
        let (e, tp) = (a.e, a.tp);
        if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
            return Ok(());
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| deckspfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
        }
        Ok(())
    });
    c.register_effect(duel, e2);
}

fn attribute_condition(duel: &Duel, tp: u8) -> bool {
    duel.get_matching_group(CardId::is_monster, tp, LOCATION_GRAVE, 0, None)
        .get_bin_class_count(duel, |c, duel| u64::from(c.get_attribute(duel)))
        >= 3
}

fn deckspfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    let attribute = c.get_attribute(duel);
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && !duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_attribute(duel, attribute)),
            tp,
            LOCATION_MZONE | LOCATION_GRAVE,
            0,
            1,
            None,
        )
}
