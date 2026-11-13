// Alligator's Dragon Knight
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 4881365;

pub const LISTED_NAMES: &[u32] = &[CARD_DARK_TIME_WIZARD];

// In the hand with cards that mention Dark Time Wizard to reveal, and Spells that mention it in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (40235813, HAND),
    (77482666, HAND),
    (40235813, DECK),
    (40235813, DECK),
    (65118318, DECK),
    (65118318, DECK),
    (3129527, DECK),
    (76630812, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_DIRECT_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, (ID, 0));
    e2.set_cost(duel, |duel, a, chk| {
        Cost::reveal(
            duel,
            a,
            chk,
            |c, duel, _, _| c.lists_code(duel, CARD_DARK_TIME_WIZARD),
            true,
            (1, 1),
            None,
            LOCATION_HAND,
        )
    });
    e2.set_target(duel, stock::special_summon_self_target);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
    let e3a = duel.create_effect(c);
    e3a.set_description(duel, aux::stringid(ID, 1));
    e3a.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_HANDES);
    e3a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3a.set_property(duel, EFFECT_FLAG_DELAY);
    e3a.set_code(duel, EVENT_SUMMON_SUCCESS);
    e3a.set_count_limit(duel, 1, (ID, 1));
    e3a.set_target(duel, thtg);
    e3a.set_operation(duel, thop);
    c.register_effect(duel, e3a);
    let e3b = e3a.clone_effect(duel);
    e3b.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3b);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel) && c.lists_code(duel, CARD_DARK_TIME_WIZARD) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_HANDES, None, 0, tp, 1);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
    if g.is_empty() {
        return Ok(());
    }
    let sg = aux::select_unselect_group(duel, g, 1, 2, &aux::dncheck, tp, HINTMSG_ATOHAND)?;
    if !sg.is_empty() && duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)? > 0 {
        duel.confirm_cards(1 - tp, sg)?;
        duel.shuffle_hand(tp)?;
        duel.break_effect()?;
        duel.discard_hand(
            tp,
            |c, duel| c.is_discardable_full(duel, REASON_EFFECT, None),
            1,
            1,
            REASON_EFFECT | REASON_DISCARD,
        )?;
    }
    Ok(())
}
