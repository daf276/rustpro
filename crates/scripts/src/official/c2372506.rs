// Chaos Magical Hats
use crate::prelude::*;
use crate::testing::{DECK, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 2372506;

pub const LISTED_NAMES: &[u32] = &[CARD_LIGHT_AND_DARKNESS_RITUAL, ID];
// A copy in the hand and one Set, Black Luster Soldier - Soldier of Light and Darkness to Set, three different Spells in
// the Deck, and Spells of the opponent's to respond to.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (70405001, MZONE),
    (53129443, DECK),
    (12580477, DECK),
    (24094653, DECK),
    (53129443, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_POSITION + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    a.rp == 1 - a.tp
        && (re.is_monster_effect(duel) || (re.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && re.get_handler(duel).is_normal_spell_trap(duel)))
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.lists_code(duel, CARD_LIGHT_AND_DARKNESS_RITUAL) && c.is_can_turn_set(duel)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk != 0 {
        return Ok(false);
    }
    let g = duel.get_matching_group(CardId::is_spell_trap, tp, LOCATION_DECK, 0, None);
    let stats = MonsterStats {
        setcode: None,
        card_type: Some(TYPE_MONSTER | TYPE_NORMAL),
        attack: Some(0),
        defense: Some(0),
        level: Some(8),
        race: Some(RACE_SPELLCASTER),
        attribute: Some(ATTRIBUTE_DARK),
    };
    Ok(aux::select_unselect_group_check(duel, g, 3, 3, &aux::dncheck)
        && duel.get_location_count(tp, LOCATION_MZONE) >= 3
        && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
        && duel.is_player_can_special_summon_monster_full(tp, ID, stats, POS_FACEDOWN_DEFENSE, tp, 0)
        && duel.is_existing_matching_card(setfilter, tp, LOCATION_MMZONE, 0, 1, None))
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() || duel.get_location_count(tp, LOCATION_MZONE) < 3 {
        return Ok(());
    }
    let g = duel.get_matching_group(CardId::is_spell_trap, tp, LOCATION_DECK, 0, None);
    let sg = aux::select_unselect_group_full(duel, g, 3, 3, &aux::dncheck, tp, HINTMSG_SPSUMMON, None, false)?;
    if sg.len() != 3 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_POSCHANGE);
    let Some(mc) = duel
        .select_matching_card(tp, setfilter, tp, LOCATION_MMZONE, 0, 1, 1, None)?
        .get_first()
    else {
        return Ok(());
    };
    if mc.is_immune_to_effect(duel, a.e) {
        return Ok(());
    }
    duel.hint_selection(mc);
    for sc in sg {
        let e1 = duel.create_effect(sc);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_CHANGE_TYPE);
        e1.set_value(duel, (TYPE_NORMAL | TYPE_MONSTER) as i32);
        e1.set_reset(
            duel,
            RESET_EVENT | RESET_TOGRAVE | RESET_REMOVE | RESET_TEMP_REMOVE | RESET_TOHAND | RESET_TODECK | RESET_OVERLAY,
        );
        sc.register_effect_full(duel, e1, true);
        let changes = [
            (EFFECT_CHANGE_RACE, RACE_SPELLCASTER as i32),
            (EFFECT_CHANGE_ATTRIBUTE, ATTRIBUTE_DARK as i32),
            (EFFECT_CHANGE_LEVEL, 8),
            (EFFECT_SET_BASE_ATTACK, 0),
            (EFFECT_SET_BASE_DEFENSE, 0),
            (EFFECT_CHANGE_CODE, ID as i32),
        ];
        for (code, value) in changes {
            let e = e1.clone_effect(duel);
            e.set_code(duel, code);
            e.set_value(duel, value);
            sc.register_effect_full(duel, e, true);
        }
    }
    duel.special_summon(sg, 0, tp, tp, true, false, POS_FACEDOWN_DEFENSE)?;
    duel.confirm_cards(1 - tp, sg)?;
    if mc.is_has_effect(duel, EFFECT_LIGHT_OF_INTERVENTION).is_some() {
        duel.change_position(mc, POS_FACEUP_DEFENSE)?;
    } else {
        duel.change_position(mc, POS_FACEDOWN_DEFENSE)?;
        mc.clear_effect_relation(duel);
    }
    let mut sg = sg;
    sg.insert(mc);
    duel.shuffle_set_card(sg)?;
    duel.break_effect()?;
    duel.change_target_card(a.ev as u8, CardSet::default());
    duel.change_chain_operation(a.ev as u8, repop);
    Ok(())
}

fn repop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(tp, CardId::is_facedown, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
