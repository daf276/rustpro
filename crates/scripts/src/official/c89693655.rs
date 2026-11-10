// Subspace Battle
use crate::prelude::*;

const ID: u32 = 89693655;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_DESTROY + CATEGORY_DAMAGE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn revealfilter(c: CardId, duel: &Duel) -> bool {
    c.is_attack_above(duel, 0) && !c.is_public(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(revealfilter, tp, LOCATION_DECK, 0, 1, None)
            && duel.get_field_group_count(tp, 0, LOCATION_DECK) > 0);
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, PLAYER_ALL, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_DESTROY, None, 1, PLAYER_ALL, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_DAMAGE, None, 0, PLAYER_ALL, 500);
    Ok(false)
}

fn both_decks_revealable(duel: &Duel, tp: u8) -> bool {
    duel.is_existing_matching_card(revealfilter, tp, LOCATION_DECK, 0, 1, None)
        && duel.is_existing_matching_card(revealfilter, tp, 0, LOCATION_DECK, 1, None)
}

fn reveal(duel: &mut Duel, player: u8) -> Script<Option<CardId>> {
    duel.hint(HINT_SELECTMSG, player, HINTMSG_CONFIRM);
    Ok(duel
        .select_matching_card(player, revealfilter, player, LOCATION_DECK, 0, 1, 1, None)?
        .get_first())
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if !both_decks_revealable(duel, tp) {
        return Ok(());
    }
    let opp = 1 - tp;
    let (mut tp_codes, mut opp_codes) = (vec![], vec![]);
    for i in 1..=3 {
        if i > 1 {
            duel.break_effect()?;
        }
        let (Some(sc1), Some(sc2)) = (reveal(duel, tp)?, reveal(duel, opp)?) else {
            return Ok(());
        };
        duel.confirm_cards(opp, sc1)?;
        duel.confirm_cards(tp, sc2)?;
        tp_codes.push(sc1.get_code(duel));
        opp_codes.push(sc2.get_code(duel));
        sc1.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
        sc2.register_flag_effect(duel, ID + 1, RESETS_STANDARD_PHASE_END, 0, 1);
        let (atk1, atk2) = (sc1.get_attack(duel), sc2.get_attack(duel));
        if atk1 == atk2 {
            duel.shuffle_deck(tp)?;
            duel.shuffle_deck(opp)?;
        } else {
            let (search_card, destroy_card) = if atk1 > atk2 { (sc1, sc2) } else { (sc2, sc1) };
            if duel.send_to_hand(search_card, PLAYER_NONE, REASON_EFFECT)? > 0 {
                let prev_ctrl = search_card.get_previous_controler(duel);
                duel.confirm_cards(1 - prev_ctrl, search_card)?;
                duel.shuffle_hand(prev_ctrl)?;
            } else {
                duel.send_to_grave(search_card, REASON_RULE)?;
            }
            if duel.destroy(destroy_card, REASON_EFFECT)? > 0 {
                let prev_ctrl = destroy_card.get_previous_controler(duel);
                duel.damage(prev_ctrl, 500, REASON_EFFECT)?;
            }
        }
        if !both_decks_revealable(duel, tp) {
            break;
        }
    }
    let c = e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_target_range(duel, 1, 0);
    e1.set_value(duel, Value::EffectPlayer(cannot_activate));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    e1.set_labels(duel, &pack(0, &tp_codes));
    duel.register_effect(e1, tp);
    let e2 = e1.clone_effect(duel);
    e2.set_labels(duel, &pack(1, &opp_codes));
    duel.register_effect(e2, opp);
    Ok(())
}

const CODE_BITS: u32 = 29;
const CODE_MASK: i64 = (1 << CODE_BITS) - 1;

// An effect has two labels: they hold the three revealed codes (29 bits each) and which flag effect marks the revealed card.
fn pack(flag_offset: u32, codes: &[u32]) -> [i64; 2] {
    let code = |i| codes.get(i).map_or(0, |&code| i64::from(code));
    [code(0) | code(1) << CODE_BITS, code(2) | i64::from(flag_offset) << CODE_BITS]
}

fn cannot_activate(duel: &Duel, e: EffectId, re: EffectId, _: u8) -> bool {
    let rc = re.get_handler(duel);
    let [first, second] = *e.get_labels(duel) else { return false };
    let flag = ID + (second >> CODE_BITS) as u32;
    let codes = [first & CODE_MASK, first >> CODE_BITS, second & CODE_MASK];
    re.is_monster_effect(duel) && (rc.has_flag_effect(duel, flag) || codes.iter().any(|&code| code != 0 && rc.is_code(duel, code as u32)))
}
