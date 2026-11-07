// Clown Crew Rehearsal
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 20448151;

struct Params;

impl RitualParams for Params {
    const DESC: u64 = aux::stringid(ID, 1);

    fn filter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_CLOWN_CREW])
    }
}

pub const LISTED_NAMES: &[u32] = &[82159583, ID];
// In the hand and the Graveyard, with Clown Crew Biancaviso and Spells/Traps in the Deck to search, and Clown Crew Flair in
// the hand to Ritual Summon by Tributing the Normal Monster Mystical Elf.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (82159583, DECK),
    (84031736, DECK),
    (57847269, DECK),
    (42759961, HAND),
    (15025844, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_cost(duel, thcost);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = Ritual::create_proc::<Params, RITPROC_GREATER>(duel, c);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_cost(duel, Cost::self_banish);
    c.register_effect(duel, e2);
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, aux::TRUE, (1, 1), true, None, None));
    }
    let g = duel.select_release_group_cost(tp, aux::TRUE, (1, 1), true, None, None)?;
    duel.release(g, REASON_COST)?;
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_code(duel, 82159583) || (c.is_set_card(duel, &[SET_CLOWN_CREW]) && c.is_spell_trap(duel) && !c.is_code(duel, ID)))
        && c.is_able_to_hand(duel)
}

fn rescon(duel: &Duel, sg: CardSet, _: CardSet, _: Option<CardId>) -> (bool, bool) {
    (
        sg.is_exists(duel, |c, duel| c.is_code(duel, 82159583), 1, None) && sg.is_exists(duel, CardId::is_spell_trap, 1, None),
        false,
    )
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        let g = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
        return Ok(aux::select_unselect_group_check(duel, g, 2, 2, &rescon));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 2, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(thfilter, tp, LOCATION_DECK, 0, None);
    if g.len() < 2 {
        return Ok(());
    }
    let sg = aux::select_unselect_group(duel, g, 2, 2, &rescon, tp, HINTMSG_ATOHAND)?;
    if sg.len() == 2 {
        duel.send_to_hand(sg, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, sg)?;
    }
    Ok(())
}
