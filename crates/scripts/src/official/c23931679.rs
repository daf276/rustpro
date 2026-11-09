// Ocean Dragon Lord - Kairyu-Shin
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 23931679;

pub const LISTED_NAMES: &[u32] = &[CARD_UMI];

// On the field with Umi in the hand to activate and in the Deck to search, and Sangans of both players for the one non-WATER
// monster limit. No "Kairyu-Shin" or "Sea Stealth" Spell/Trap is ported.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (CARD_UMI, HAND),
    (CARD_UMI, DECK),
    (CARD_UMI, DECK),
    (26202165, MZONE),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    e1.set_code(duel, EVENT_ADJUST);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_operation(duel, adjustop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_CANNOT_SUMMON);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_target_range(duel, 1, 1);
    e2.set_condition(duel, umicon);
    e2.set_target_spsummon(duel, sumlimit);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_CANNOT_FLIP_SUMMON);
    c.register_effect(duel, e3);
    let e4 = e2.clone_effect(duel);
    e4.set_code(duel, EFFECT_FORCE_SPSUMMON_POSITION);
    e4.set_value(duel, POS_FACEDOWN as i32);
    c.register_effect(duel, e4);
    let e5 = duel.create_effect(c);
    e5.set_description(duel, aux::stringid(ID, 0));
    e5.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e5.set_type(duel, EFFECT_TYPE_IGNITION);
    e5.set_range(duel, LOCATION_MZONE);
    e5.set_count_limit(duel, 1, ID);
    e5.set_target(duel, thtg);
    e5.set_operation(duel, thop);
    c.register_effect(duel, e5);
    let e6 = duel.create_effect(c);
    e6.set_type(duel, EFFECT_TYPE_FIELD);
    e6.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e6.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e6.set_range(duel, LOCATION_MZONE);
    e6.set_target_range(duel, 1, 1);
    e6.set_value(duel, Value::EffectPlayer(activatelimit));
    c.register_effect(duel, e6);
}

fn umicon(duel: &Duel, _: &Args) -> bool {
    let umi = aux::faceup_filter(|c, duel| c.is_code(duel, CARD_UMI));
    duel.is_existing_matching_card(umi, 0, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None) || duel.is_environment(CARD_UMI)
}

fn nonwaterfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_attribute_except(duel, ATTRIBUTE_WATER)
}

fn adjustop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let flags = [ID, ID + 1].map(|code| c.has_flag_effect(duel, code));
    if !umicon(duel, a) {
        for (code, flag) in [ID, ID + 1].into_iter().zip(flags) {
            if flag {
                c.reset_flag_effect(duel, code);
            }
        }
        return Ok(());
    }
    let phase = duel.get_current_phase();
    if (phase == PHASE_DAMAGE && !duel.is_damage_calculated()) || phase == PHASE_DAMAGE_CAL {
        return Ok(());
    }
    let mut sides = [
        (
            ID,
            flags[0],
            tp,
            duel.get_matching_group(nonwaterfilter, tp, LOCATION_MZONE, 0, None),
        ),
        (
            ID + 1,
            flags[1],
            1 - tp,
            duel.get_matching_group(nonwaterfilter, tp, 0, LOCATION_MZONE, None),
        ),
    ];
    for (code, flag, player, g) in &mut sides {
        let count = g.len();
        if count == 0 && *flag {
            c.reset_flag_effect(duel, *code);
        } else if count > 0 {
            if *flag && count > 1 {
                let fid = c.get_flag_effect_label(duel, *code);
                g.keep_matching(duel, |c, duel| Some(i64::from(c.get_field_id(duel))) != fid, None);
            } else if *flag {
                g.clear();
            } else {
                let mut tc = g.get_first();
                if count > 1 {
                    duel.hint(HINT_SELECTMSG, *player, aux::stringid(ID, 1));
                    tc = g.select(duel, *player, 1, 1, None)?.get_first();
                    if let Some(tc) = tc {
                        duel.hint_selection_full(tc, true);
                    }
                }
                if let Some(tc) = tc {
                    g.remove(tc);
                    c.register_flag_effect_full(
                        duel,
                        *code,
                        RESET_EVENT | RESETS_STANDARD_DISABLE,
                        0,
                        1,
                        i64::from(tc.get_field_id(duel)),
                        0,
                    );
                }
            }
        }
    }
    let mut readjust = false;
    for (_, _, player, g) in sides {
        if !g.is_empty() {
            duel.send_to_grave_full(g, REASON_RULE, PLAYER_NONE, player)?;
            readjust = true;
        }
    }
    if readjust {
        duel.readjust()?;
    }
    Ok(())
}

// The summoning player is the target player's, which a nil target player stands for in Lua.
fn sumlimit(duel: &Duel, _: EffectId, c: CardId, _: u8, _: u32, _: u8, targetp: u8, _: Option<EffectId>) -> bool {
    !c.is_attribute(duel, ATTRIBUTE_WATER) && duel.is_existing_matching_card(nonwaterfilter, targetp, LOCATION_MZONE, 0, 1, None)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_hand(duel)
        && (c.is_code(duel, CARD_UMI) || (c.is_set_card(duel, &[SET_KAIRYU_SHIN, SET_SEA_STEALTH]) && c.is_spell_trap(duel)))
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, a.tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, a.tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, g)?;
    }
    Ok(())
}

fn activatelimit(duel: &Duel, _: EffectId, re: EffectId, tp: u8) -> bool {
    re.has_self_change_position_cost(duel)
        && re.get_handler(duel).is_attribute_except(duel, ATTRIBUTE_WATER)
        && duel.is_existing_matching_card(nonwaterfilter, tp, LOCATION_MZONE, 0, 1, None)
}
