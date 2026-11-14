// Throne of the Archfiends
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, PZONE, SZONE};

const ID: u32 = 63679166;
const EMPEROR: u32 = 48469380;

pub const LISTED_NAMES: &[u32] = &[EMPEROR];
pub const LISTED_SERIES: &[u16] = &[SET_ARCHFIEND];

struct Params;

impl RitualParams for Params {
    const LOCATION: Option<u32> = Some(LOCATION_HAND | LOCATION_GRAVE | LOCATION_EXTRA | LOCATION_REMOVED);

    fn filter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_ARCHFIEND]) && c.is_ritual_monster(duel) && (c.is_faceup(duel) || c.is_location(duel, LOCATION_HAND))
    }
}

// Archfiend Emperor (Level 7) with Level 4 Archfiend Soldiers in hand as the Tribute, and one in a Pendulum Zone with Toon World
// to destroy itself into the Extra Deck face-up for the return to hand of a copy in the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (EMPEROR, HAND),
    (EMPEROR, PZONE),
    (49881766, HAND),
    (49881766, HAND),
    (15259703, SZONE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(EMPEROR), Check::ResolvesEffect(aux::stringid(ID, 1))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Ritual::add_proc_greater::<Params>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_category(duel, CATEGORY_TOHAND);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_PHASE + PHASE_STANDBY);
    e1.set_range(duel, LOCATION_GRAVE | LOCATION_REMOVED);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| {
        let filter = aux::faceup_filter(|c, duel| c.is_code(duel, EMPEROR));
        duel.is_existing_matching_card(filter, a.tp, LOCATION_EXTRA, 0, 1, None)
    });
    e1.set_target(duel, thtg);
    e1.set_operation(duel, stock::self_to_hand);
    c.register_effect(duel, e1);
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_hand(duel));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, Some(c.into()), 1, a.tp, 0);
    Ok(false)
}
