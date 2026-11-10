// Magical Hats
use crate::prelude::*;
use crate::testing::{DECK, MZONE, OPPONENT, SZONE};

const ID: u32 = 81210420;

// Set, with a monster on the field and Spells in the Deck, against the opponent's attackers.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (26976414, MZONE),
    (14851496, MZONE),
    (12580477, DECK),
    (53129443, DECK),
    (24094653, DECK),
    (13597785, MZONE | OPPONENT),
    (70781052, MZONE | OPPONENT),
];

const RESET_HAT: u32 = RESET_EVENT | RESET_TOGRAVE | RESET_REMOVE | RESET_TEMP_REMOVE | RESET_TOHAND | RESET_TODECK | RESET_OVERLAY;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_POSITION + CATEGORY_SPECIAL_SUMMON + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, 0, TIMING_BATTLE_START | TIMING_BATTLE_END);
    e1.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp) && duel.is_battle_phase());
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.get_sequence(duel) < 5 && !c.is_type(duel, TYPE_TOKEN + TYPE_LINK)
}

fn spfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    let stats = MonsterStats::new(0, TYPE_MONSTER | TYPE_NORMAL, 0, 0, 0, 0, 0);
    c.is_spell_trap(duel)
        && duel.check_mut(|duel| duel.is_player_can_special_summon_monster_full(tp, c.get_code(duel), stats, POS_FACEDOWN, tp, 0))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    Ok(chk != 0
        || (duel.is_existing_matching_card(filter, tp, LOCATION_MZONE, 0, 1, None)
            && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.get_location_count(tp, LOCATION_MZONE) > 1
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, tp), tp, LOCATION_DECK, 0, 2, None)))
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() || duel.get_location_count(tp, LOCATION_MZONE) < 2 {
        return Ok(());
    }
    let g = duel.get_matching_group(|c, duel| spfilter(c, duel, tp), tp, LOCATION_DECK, 0, None);
    if g.len() < 2 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let Some(tc) = duel
        .select_matching_card(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    if tc.is_immune_to_effect(duel, e) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let mut sg = g.select(duel, tp, 2, 2, None)?;
    if tc.is_faceup(duel) {
        if tc.is_has_effect(duel, EFFECT_LIGHT_OF_INTERVENTION).is_some() {
            duel.change_position(tc, POS_FACEUP_DEFENSE)?;
        } else {
            duel.change_position(tc, POS_FACEDOWN_DEFENSE)?;
            tc.clear_effect_relation(duel);
        }
    }
    let fid = e.get_handler(duel).get_field_id(duel);
    for tg in sg {
        let e1 = duel.create_effect(tg);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CHANGE_TYPE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_value(duel, (TYPE_NORMAL + TYPE_MONSTER) as i32);
        e1.set_reset(duel, RESET_HAT);
        tg.register_effect_full(duel, e1, true);
        let changes = [
            (EFFECT_REMOVE_RACE, RACE_ALL as i32),
            (EFFECT_REMOVE_ATTRIBUTE, 0xff),
            (EFFECT_SET_BASE_ATTACK, 0),
            (EFFECT_SET_BASE_DEFENSE, 0),
        ];
        for (code, value) in changes {
            let ex = e1.clone_effect(duel);
            ex.set_code(duel, code);
            ex.set_value(duel, value);
            tg.register_effect_full(duel, ex, true);
        }
        tg.register_flag_effect_full(duel, ID, RESET_HAT | RESET_PHASE | PHASE_BATTLE, 0, 1, i64::from(fid), 0);
        tg.set_status(duel, STATUS_NO_LEVEL, true);
    }
    duel.special_summon(sg, 0, tp, tp, true, false, POS_FACEDOWN_DEFENSE)?;
    duel.confirm_cards(1 - tp, sg)?;
    sg.insert(tc);
    duel.shuffle_set_card(sg)?;
    sg.remove(tc);
    let de = duel.create_effect(e.get_handler(duel));
    de.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    de.set_code(duel, EVENT_PHASE | PHASE_BATTLE);
    de.set_reset(duel, RESET_PHASE | PHASE_BATTLE);
    de.set_property(duel, EFFECT_FLAG_IGNORE_IMMUNE);
    de.set_count_limit(duel, 1, 0);
    de.set_label(duel, i64::from(fid));
    de.set_label_object_group(duel, Some(sg));
    de.set_operation(duel, desop);
    duel.register_effect(de, tp);
    Ok(())
}

fn desfilter(c: CardId, duel: &Duel, fid: i64) -> bool {
    c.get_flag_effect_label(duel, ID) == Some(fid)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = a.e.get_label_object_group(duel).unwrap_or_default();
    let fid = a.e.get_label(duel);
    let tg = g.filter(duel, |c, duel| desfilter(c, duel, fid), None);
    duel.destroy(tg, REASON_EFFECT)?;
    let tg2 = tg.filter(duel, |c, duel| desfilter(c, duel, fid), None);
    duel.send_to_grave(tg2, REASON_EFFECT)?;
    Ok(())
}
