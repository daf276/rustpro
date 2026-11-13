// Ancient Secrets
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 30752324;

// Normal Monsters (Hyosube, Psychic Kappa) on the field and in the hand, Deck and Graveyard, and opponent's monsters (Lucky Trinket,
// Sangan) and Traps for the destroying options; Pot of Greed-like draws and the Graveyard option come from the same board.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (7892180, MZONE),
    (2118022, HAND),
    (2118022, DECK),
    (2118022, GRAVE),
    (3985011, MZONE | OPPONENT),
    (26202165, GRAVE | OPPONENT),
    (4206964, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DESTROY + CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, acttg);
    e1.set_operation(duel, actop);
    c.register_effect(duel, e1);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_type(duel, TYPE_NORMAL)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
        && (c.is_level_below(duel, 4) || !c.is_location(duel, LOCATION_DECK))
}

fn normal_card_on_field(duel: &Duel, tp: u8) -> bool {
    let filter = aux::faceup_filter(|c, duel| c.is_original_type(duel, TYPE_NORMAL));
    duel.is_existing_matching_card(filter, tp, LOCATION_ONFIELD, 0, 1, None)
}

fn acttg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let mmz_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
    let monster_card_chk = normal_card_on_field(duel, tp);
    if chk == 0 {
        if mmz_chk
            && duel.is_existing_matching_card(
                |c, duel| spfilter(c, duel, e, tp),
                tp,
                LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE,
                0,
                1,
                None,
            )
        {
            return Ok(true);
        }
        return Ok(monster_card_chk
            && (duel.get_field_group_count(tp, 0, LOCATION_ONFIELD) > 0
                || duel.is_player_can_draw(tp, 2)
                || (mmz_chk
                    && duel.is_existing_matching_card(
                        |c, duel| c.is_can_be_special_summoned(duel, e, 0, tp, false, false),
                        tp,
                        LOCATION_GRAVE,
                        LOCATION_GRAVE,
                        1,
                        None,
                    ))));
    }
    if !monster_card_chk {
        duel.set_operation_info(
            0,
            CATEGORY_SPECIAL_SUMMON,
            None,
            1,
            tp,
            (LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE) as i32,
        );
    }
    duel.set_possible_operation_info(
        0,
        CATEGORY_SPECIAL_SUMMON,
        None,
        1,
        PLAYER_EITHER,
        (LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE) as i32,
    );
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, None, 1, 1 - tp, LOCATION_ONFIELD as i32);
    duel.set_possible_operation_info(0, CATEGORY_DRAW, None, 1, tp, 2);
    Ok(false)
}

fn actop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let mmz_chk = duel.get_location_count(tp, LOCATION_MZONE) > 0;
    let op = if normal_card_on_field(duel, tp) {
        let b1 = mmz_chk
            && duel.is_existing_matching_card(
                aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp)),
                tp,
                LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE,
                0,
                1,
                None,
            );
        let b2 = duel.get_field_group_count(tp, 0, LOCATION_MZONE) > 0;
        let b3 = duel.is_existing_matching_card(CardId::is_spell_trap, tp, 0, LOCATION_ONFIELD, 1, None);
        let b4 = duel.is_player_can_draw(tp, 2);
        let b5 = mmz_chk
            && duel.is_existing_matching_card(
                aux::necro_valley_filter(|c, duel| c.is_can_be_special_summoned(duel, e, 0, tp, false, false)),
                tp,
                LOCATION_GRAVE,
                LOCATION_GRAVE,
                1,
                None,
            );
        duel.select_effect(
            tp,
            &[
                (b1, aux::stringid(ID, 1)),
                (b2, aux::stringid(ID, 2)),
                (b3, aux::stringid(ID, 3)),
                (b4, aux::stringid(ID, 4)),
                (b5, aux::stringid(ID, 5)),
            ],
        )?
    } else {
        Some(1)
    };
    match op {
        Some(1) => {
            if !mmz_chk {
                return Ok(());
            }
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
            let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_DECK | LOCATION_GRAVE, 0, 1, 1, None)?;
            if !g.is_empty() {
                duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
            }
        }
        Some(2) => {
            let g = duel.get_field_group(tp, 0, LOCATION_MZONE);
            if !g.is_empty() {
                duel.destroy(g, REASON_EFFECT)?;
            }
        }
        Some(3) => {
            let g = duel.get_matching_group(CardId::is_spell_trap, tp, 0, LOCATION_ONFIELD, None);
            if !g.is_empty() {
                duel.destroy(g, REASON_EFFECT)?;
            }
        }
        Some(4) => {
            duel.draw(tp, 2, REASON_EFFECT)?;
        }
        Some(5) => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
            let filter = aux::necro_valley_filter(|c, duel| c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
            let g = duel.select_matching_card(tp, filter, tp, LOCATION_GRAVE, LOCATION_GRAVE, 1, 1, None)?;
            if !g.is_empty() {
                duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
            }
        }
        _ => {}
    }
    Ok(())
}
