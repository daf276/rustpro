// Cartoon Shade
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 55363218;

// In the hand and on the field with Chaos Sorcerer, which cannot be Normal Summoned, to enable and Chaos Sorcerer and Mirage Knight in the
// Deck to add, and Level 7 Witchcrafter Haine in the hand to Tribute Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (9596126, MZONE),
    (9596126, DECK),
    (49217579, DECK),
    (84523092, HAND),
    (84523092, HAND),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e0 = duel.create_effect(c);
    e0.set_description(duel, aux::stringid(ID, 0));
    e0.set_type(duel, EFFECT_TYPE_FIELD);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_SPSUMMON_PROC);
    e0.set_range(duel, LOCATION_HAND);
    e0.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e0.set_condition(duel, |duel, a| {
        let Some(c) = a.c else { return true };
        duel.get_location_count(c.get_controler(duel), LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(
                aux::faceup_filter(|c, duel| !c.is_summonable_card(duel)),
                0,
                LOCATION_MZONE,
                LOCATION_MZONE,
                1,
                None,
            )
    });
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 2));
    e2.set_category(duel, CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_RELEASE);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_ONFIELD));
    e2.set_target(duel, retthtg);
    e2.set_operation(duel, retthop);
    c.register_effect(duel, e2);
}

fn monsterwith2000atkdef(c: CardId, duel: &Duel) -> bool {
    c.is_attack(duel, &[2000]) || c.is_defense(duel, &[2000])
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    monsterwith2000atkdef(c, duel) && !c.is_summonable_card(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn retthtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_able_to_hand(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_able_to_hand, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    let maxct = if duel.is_existing_matching_card(aux::faceup_filter(monsterwith2000atkdef), tp, LOCATION_MZONE, 0, 1, None) {
        2
    } else {
        1
    };
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(tp, CardId::is_able_to_hand, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, maxct, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, g.map_or(0, |g| g.len() as u8), tp, 0);
    Ok(false)
}

fn retthop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tg = duel.get_target_cards(a.e).unwrap_or_default();
    if !tg.is_empty() {
        duel.send_to_hand(tg, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
