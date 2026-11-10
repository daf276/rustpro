// Sacred Spirit Sword Aiwass
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, OPPONENT};

const ID: u32 = 63926180;
const AIWASS: u32 = 84288367;

pub const LISTED_NAMES: &[u32] = &[AIWASS];

// Aiwass and Aleister the Invoker in the Deck, and Secreterion Dragon in the opponent's Extra Deck, for the three effects.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (AIWASS, DECK),
    (AIWASS, DECK),
    (86120751, DECK),
    (86120751, DECK),
    (89851827, EXTRA | OPPONENT),
    (89851827, EXTRA | OPPONENT),
    (89851827, EXTRA | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_code(duel, AIWASS) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn tgbanfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_ALEISTER]) && c.is_monster(duel) && (c.is_able_to_grave(duel) || c.is_able_to_remove(duel))
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let b1 = duel.get_location_count(tp, LOCATION_MZONE) > 0
        && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK | LOCATION_GRAVE, 0, 1, None);
    let b2 = duel.is_existing_matching_card(tgbanfilter, tp, LOCATION_DECK, 0, 1, None);
    let b3 = duel.is_existing_matching_card(
        |c, duel| c.is_facedown(duel) && c.is_able_to_remove(duel),
        tp,
        0,
        LOCATION_EXTRA,
        3,
        None,
    );
    if chk == 0 {
        return Ok(b1 || b2 || b3);
    }
    let op = duel.select_effect(
        tp,
        &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2)), (b3, aux::stringid(ID, 3))],
    )?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    match op {
        Some(1) => {
            e.set_category(duel, CATEGORY_SPECIAL_SUMMON);
            duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
        }
        Some(2) => {
            e.set_category(duel, CATEGORY_TOGRAVE + CATEGORY_REMOVE);
            duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
            duel.set_possible_operation_info(0, CATEGORY_REMOVE, None, 1, tp, LOCATION_DECK as i32);
        }
        Some(3) => {
            e.set_category(duel, CATEGORY_REMOVE);
            duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, 1 - tp, LOCATION_EXTRA as i32);
        }
        _ => {}
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    match e.get_label(duel) {
        1 => {
            if duel.get_location_count(tp, LOCATION_MZONE) > 0 {
                duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
                let g = duel.select_matching_card(
                    tp,
                    aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp)),
                    tp,
                    LOCATION_DECK | LOCATION_GRAVE,
                    0,
                    1,
                    1,
                    None,
                )?;
                if !g.is_empty() {
                    duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
                }
            }
        }
        2 => {
            duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 4));
            let sc = duel
                .select_matching_card(tp, tgbanfilter, tp, LOCATION_DECK, 0, 1, 1, None)?
                .get_first();
            if let Some(sc) = sc {
                let op = duel.select_effect(
                    tp,
                    &[
                        (sc.is_able_to_grave(duel), aux::stringid(ID, 5)),
                        (sc.is_able_to_remove(duel), aux::stringid(ID, 6)),
                    ],
                )?;
                if op == Some(1) {
                    duel.send_to_grave(sc, REASON_EFFECT)?;
                } else if op == Some(2) {
                    duel.remove(sc, POS_FACEUP, REASON_EFFECT)?;
                }
            }
        }
        3 => {
            let g = duel.get_field_group(tp, 0, LOCATION_EXTRA).random_select(duel, tp, 3);
            if !g.is_empty() {
                duel.confirm_cards(tp, g)?;
                duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
                let rg = g.filter_select(duel, tp, CardId::is_able_to_remove, 1, 1, None)?;
                if !rg.is_empty() {
                    duel.remove(rg, POS_FACEUP, REASON_EFFECT)?;
                }
                duel.shuffle_extra(1 - tp)?;
            }
        }
        _ => {}
    }
    let c = e.get_handler(duel);
    aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 7)));
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_code(duel, EFFECT_CANNOT_ATTACK_ANNOUNCE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| !c.is_fusion_monster(duel));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}
