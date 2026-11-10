// Mitsurugi Mirror
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 49721684;

struct Params;

impl RitualParams for Params {
    const LOCATION: Option<u32> = Some(LOCATION_HAND | LOCATION_GRAVE);

    fn filter(c: CardId, duel: &Duel) -> bool {
        c.is_race(duel, RACE_REPTILE)
    }

    fn matfilter(c: CardId, duel: &Duel, _: EffectId, _: u8) -> bool {
        c.is_race(duel, RACE_REPTILE)
    }
}

pub const LISTED_NAMES: &[u32] = &[19899073, 55397172, 13332685];

// Copies in the hand and the Graveyard, with Ame no Murakumo no Mitsurugi in the hand to Ritual Summon by Tributing Ame no
// Habakiri no Mitsurugi (Level 8) on the field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, GRAVE), (19899073, HAND), (13332685, MZONE), (13332685, MZONE)];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(19899073), Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = Ritual::create_proc::<Params, RITPROC_GREATER>(duel, c);
    e1.set_count_limit(duel, 1, ID);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TODECK);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property_full(duel, EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e2.set_code(duel, EVENT_RELEASE);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, tdcon);
    e2.set_target(duel, tdtg);
    e2.set_operation(duel, tdop);
    c.register_effect(duel, e2);
}

fn tdconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_previous_code_on_field(duel, LISTED_NAMES)
        && c.is_previous_position(duel, POS_FACEUP)
        && c.is_previous_controler(duel, tp)
        && c.is_previous_location(duel, LOCATION_ONFIELD)
}

fn tdcon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    a.eg.is_some_and(|eg| eg.is_exists(duel, |c, duel| tdconfilter(c, duel, tp), 1, None) && !eg.is_contains(a.e.get_handler(duel)))
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_deck(duel));
    }
    duel.set_operation_info(0, CATEGORY_TODECK, Some(c.into()), 1, a.tp, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.hint_selection(c);
        duel.send_to_deck(c, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}
