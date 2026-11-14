// Megalith Phul
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 78990927;

pub const LISTED_SERIES: &[u16] = &[SET_MEGALITH];

struct Params;

impl RitualParams for Params {
    const DESC: u64 = aux::stringid(ID, 1);
    const LOCATION: Option<u32> = Some(LOCATION_HAND | LOCATION_DECK);

    fn filter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_MEGALITH])
    }
}

// On the field to Ritual Summon Megalith Och from the hand or Deck with Kojikocy (Level 4), and Ritual Summoned that way from
// a copy, with Ritual Monsters in the GY to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, HAND),
    (36849933, DECK),
    (36849933, HAND),
    (1184620, HAND),
    (1184620, MZONE),
    (36849933, GRAVE),
    (30208479, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_ritual_summoned(duel));
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = Ritual::create_proc::<Params, RITPROC_GREATER>(duel, c);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_hint_timing(duel, 0, TIMING_MAIN_END);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, _| duel.is_main_phase());
    c.register_effect(duel, e2);
}

fn tgfilter(c: CardId, duel: &Duel, lv: u32) -> bool {
    c.is_ritual_monster(duel) && c.is_able_to_hand(duel) && c.get_level(duel) != lv
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let lv = a.e.get_handler(duel).get_level(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && tgfilter(chkc, duel, lv));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| tgfilter(c, duel, lv), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, |c, duel| tgfilter(c, duel, lv), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && c.is_faceup(duel) && c.is_relate_to_effect(duel, e) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CHANGE_LEVEL);
        e1.set_value(duel, tc.get_level(duel) as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e1);
        duel.break_effect()?;
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - a.tp, tc)?;
    }
    Ok(())
}
