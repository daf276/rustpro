// Dark-Eyes Illusionist Faceless Mage
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 34314989;
const CARD_MIND_SCAN: u32 = 34298391;
const TOON_GEMINI_ELF: u32 = 42386471;

pub const LISTED_NAMES: &[u32] = &[CARD_MIND_SCAN, CARD_TOON_WORLD];

// Mind Scan in the Deck to place and Toon Gemini Elf (it mentions Toon World) in the Graveyard to add and Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (CARD_MIND_SCAN, DECK),
    (CARD_MIND_SCAN, DECK),
    (TOON_GEMINI_ELF, GRAVE),
    (TOON_GEMINI_ELF, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::SpecialSummonedCode(TOON_GEMINI_ELF),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_target_card(duel, |duel, e, c| {
        let handler = e.get_handler(duel);
        c == handler || Some(c) == handler.get_battle_target(duel)
    });
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_cost(duel, Cost::self_discard);
    e2.set_target(duel, efftg);
    e2.set_operation(duel, effop);
    c.register_effect(duel, e2);
}

fn plfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_code(duel, CARD_MIND_SCAN) && !c.is_forbidden(duel) && c.check_unique_on_field(duel, tp)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.lists_code(duel, CARD_TOON_WORLD) && c.is_able_to_hand(duel)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let b1 = duel.get_location_count(tp, LOCATION_SZONE) > 0
        && duel.is_existing_matching_card(|c, duel| plfilter(c, duel, tp), tp, LOCATION_HAND | LOCATION_DECK, 0, 1, None)
        && !duel.has_flag_effect(tp, ID);
    let b2 = duel.is_existing_matching_card(thfilter, tp, LOCATION_GRAVE, 0, 1, Some(e.get_handler(duel).into()))
        && !duel.has_flag_effect(tp, ID + 1);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        e.set_category(duel, 0);
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
    } else if op == Some(2) {
        e.get_handler(duel).create_effect_relation(duel, e);
        e.set_category(duel, CATEGORY_TOHAND + CATEGORY_SPECIAL_SUMMON);
        duel.register_flag_effect(tp, ID + 1, RESET_PHASE | PHASE_END, 0, 1);
        duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_GRAVE as i32);
        duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    match e.get_label(duel) {
        1 => {
            if duel.get_location_count(tp, LOCATION_SZONE) <= 0 {
                return Ok(());
            }
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
            let sc = duel.select_matching_card(
                tp,
                |c, duel| plfilter(c, duel, tp),
                tp,
                LOCATION_HAND | LOCATION_DECK,
                0,
                1,
                1,
                None,
            )?;
            if let Some(sc) = sc.get_first() {
                duel.move_to_field(sc, tp, tp, LOCATION_SZONE, POS_FACEUP, true)?;
            }
        }
        2 => {
            let c = e.get_handler(duel);
            let exc = c.is_relate_to_effect(duel, e).then_some(CardSet::from(c));
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
            let Some(sc) = duel
                .select_matching_card(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, exc)?
                .get_first()
            else {
                return Ok(());
            };
            duel.hint_selection(sc);
            if duel.send_to_hand(sc, PLAYER_NONE, REASON_EFFECT)? > 0
                && sc.is_location(duel, LOCATION_HAND)
                && duel.get_location_count(tp, LOCATION_MZONE) > 0
                && sc.is_can_be_special_summoned(duel, e, 0, tp, true, false)
            {
                duel.shuffle_hand(tp)?;
                if duel.select_yes_no(tp, aux::stringid(ID, 3))? {
                    duel.break_effect()?;
                    duel.special_summon(sc, 0, tp, tp, true, false, POS_FACEUP)?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}
