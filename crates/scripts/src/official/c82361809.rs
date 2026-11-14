// Scareclaw Reichheart
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE};

const ID: u32 = 82361809;
const SCARECLAW_ARRIVAL: u32 = 83558891;

pub const LISTED_SERIES: &[u16] = &[SET_SCARECLAW];

// Copies in the hand to summon in the columns of Scareclaw Acro, and Scareclaw Arrival in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (46877100, MZONE),
    (46877100, MZONE),
    (SCARECLAW_ARRIVAL, DECK),
    (SCARECLAW_ARRIVAL, DECK),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_value(duel, Value::SpSummonProcedure(hspval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_DRAW);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3);
}

fn sclawfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_SCARECLAW])
}

fn hspval(duel: &Duel, _: EffectId, c: CardId) -> (i32, i32) {
    let tp = c.get_controler(duel);
    let lg = duel.get_matching_group(sclawfilter, tp, LOCATION_MZONE, 0, None);
    let zone = lg.iter().fold(0, |zone, tc| {
        let left_right = u8::from(tc.is_in_main_mzone(duel, None));
        zone | tc.get_column_zone_full(duel, LOCATION_MZONE, left_right, left_right, tp)
    });
    (0, (zone & ZONES_MMZ) as i32)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SCARECLAW]) && c.is_spell_trap(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_DRAW, None, 1, tp, 1);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if g.is_empty() || duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? < 1 {
        return Ok(());
    }
    duel.confirm_cards(1 - tp, g)?;
    duel.shuffle_deck(tp)?;
    if duel.get_matching_group_count(CardId::is_defense_pos, tp, LOCATION_MZONE, LOCATION_MZONE, None) >= 3
        && duel.is_player_can_draw(tp, 1)
        && duel.select_yes_no(tp, aux::stringid(ID, 1))?
    {
        duel.break_effect()?;
        duel.draw(tp, 1, REASON_EFFECT)?;
    }
    Ok(())
}
