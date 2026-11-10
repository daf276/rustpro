// Pre-Preparation of Rites
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND};

const ID: u32 = 13048472;
const LIGHT_AND_DARKNESS_RITUAL: u32 = 33599853;
const BLACK_CHAOS: u32 = 44001993;
const SOLDIER_OF_LIGHT_AND_DARKNESS: u32 = 70405001;

// Light and Darkness Ritual in the Deck, with the Ritual Monsters it mentions in the Deck and Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (LIGHT_AND_DARKNESS_RITUAL, DECK),
    (LIGHT_AND_DARKNESS_RITUAL, DECK),
    (BLACK_CHAOS, DECK),
    (BLACK_CHAOS, GRAVE),
    (SOLDIER_OF_LIGHT_AND_DARKNESS, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn monsterfilter(c: CardId, duel: &Duel, ritual_spell: CardId) -> bool {
    c.is_ritual_monster(duel) && ritual_spell.lists_code(duel, c.get_code(duel)) && c.is_able_to_hand(duel)
}

fn spellfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_ritual_spell(duel)
        && c.is_able_to_hand(duel)
        && duel.is_existing_matching_card(
            |mc, duel| monsterfilter(mc, duel, c),
            tp,
            LOCATION_DECK | LOCATION_GRAVE,
            0,
            1,
            None,
        )
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| spellfilter(c, duel, tp), tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 2, tp, (LOCATION_DECK | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn rescon(duel: &Duel, sg: CardSet) -> (bool, bool) {
    let ritual_spell = sg.filter(duel, CardId::is_ritual_spell, None).get_first();
    (
        ritual_spell.is_some_and(|spell| sg.is_exists(duel, |c, duel| monsterfilter(c, duel, spell), 1, None)),
        ritual_spell.is_none(),
    )
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let mut g = duel.get_matching_group(|c, duel| spellfilter(c, duel, tp), tp, LOCATION_DECK, 0, None);
    if g.is_empty() {
        return Ok(());
    }
    let monsters = |c: CardId, duel: &Duel| c.is_ritual_monster(duel) && c.is_able_to_hand(duel);
    g.merge(duel.get_matching_group(aux::necro_valley_filter(monsters), tp, LOCATION_DECK | LOCATION_GRAVE, 0, None));
    let sg = aux::select_unselect_group(duel, g, 2, 2, &|duel, sg, _, _| rescon(duel, sg), tp, HINTMSG_ATOHAND)?;
    if sg.len() == 2 {
        duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, sg)?;
    }
    Ok(())
}
