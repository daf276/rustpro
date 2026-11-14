// Scrap Warrior
use super::c16449363::junk_warrior_support;
use crate::prelude::*;
use crate::testing::{DECK, EXTRA, MZONE};

const ID: u32 = 42711820;

pub const LISTED_SERIES: &[u16] = &[SET_WARRIOR];

const SCRAP_SYNCHRON: u32 = 16449363;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_summon_code(duel, Some(sc), sumtype, tp, &[SCRAP_SYNCHRON]) || c.is_has_effect(duel, 20932152).is_some()
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub const MATERIAL: &[u32] = &[SCRAP_SYNCHRON];
pub const MATERIAL_SETCODE: &[u16] = &[SET_SYNCHRON];
pub const LISTED_NAMES: &[u32] = &[SCRAP_SYNCHRON, CARD_JUNK_SYNCHRON, CARD_JUNK_WARRIOR, ID];
// In the Extra Deck, Synchro Summoned with Scrap Synchron and Atlantean Pikeman, a Level 2 Normal Monster.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (SCRAP_SYNCHRON, MZONE),
    (26976414, MZONE),
    (26976414, MZONE),
    (CARD_JUNK_SYNCHRON, DECK),
    (CARD_JUNK_SYNCHRON, DECK),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SEARCH + CATEGORY_TOHAND + CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_target(duel, thtgtg);
    e1.set_operation(duel, thtgop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_DISEFFECT);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_value(duel, Value::Chain(effval));
    c.register_effect(duel, e2);
}

fn thtgfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_code(duel, CARD_JUNK_SYNCHRON) || c.lists_code(duel, CARD_JUNK_WARRIOR)) && (c.is_able_to_hand(duel) || c.is_able_to_grave(duel))
}

fn thtgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thtgfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thtgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 1));
    let g = duel.select_matching_card(tp, thtgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first() {
        let send = |duel: &mut Duel| duel.send_to_grave(sc, REASON_EFFECT).map(|_| ());
        aux::to_hand_or_else(duel, sc, tp, |c, duel| c.is_able_to_grave(duel), send, 574)?;
    }
    Ok(())
}

fn effval(duel: &Duel, e: EffectId, ct: u8) -> bool {
    let Some(te) = duel.get_chain_info::<Option<EffectId>>(ct, CHAININFO_TRIGGERING_EFFECT) else {
        return false;
    };
    let trig_p: u8 = duel.get_chain_info(ct, CHAININFO_TRIGGERING_PLAYER);
    let trig_loc: u32 = duel.get_chain_info(ct, CHAININFO_TRIGGERING_LOCATION);
    if !(trig_p == e.get_handler_player(duel) && trig_loc & LOCATION_MZONE > 0) {
        return false;
    }
    let trig_c = te.get_handler(duel);
    junk_warrior_support(trig_c, duel) && !trig_c.is_code(duel, ID)
}
