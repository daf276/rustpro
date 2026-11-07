// Number 69: Heraldry Crest - Dark Matter Demolition
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 77571454;
pub const XYZ_NUMBER: u32 = 69;

pub const LISTED_NAMES: &[u32] = &[CARD_UNKNOWN];

// In the Extra Deck, Xyz Summoned with five Level 4 monsters.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (2118022, MZONE),
    (2483611, MZONE),
    (4042268, MZONE),
    (5628232, MZONE),
    (14851496, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 5;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_BATTLE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_EFFECT);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_type(duel, EFFECT_TYPE_QUICK_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_FREE_CHAIN);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, 0);
    e3.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e3.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e3.set_target(duel, namechangetg);
    e3.set_operation(duel, namechangeop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e4.set_code(duel, EVENT_CHAIN_SOLVING);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_operation(duel, disop);
    c.register_effect(duel, e4);
}

fn namechangefilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && !c.is_code(duel, CARD_UNKNOWN)
}

fn namechangetg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE) && namechangefilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(namechangefilter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, namechangefilter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    Ok(false)
}

fn namechangeop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) && !tc.is_code(duel, CARD_UNKNOWN) {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_CHANGE_CODE);
        e1.set_value(duel, CARD_UNKNOWN as i32);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
    }
    Ok(())
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    if a.rp == 1 - a.tp
        && let Some(re) = a.re
        && re.is_monster_effect(duel)
    {
        let rc = re.get_handler(duel);
        if rc.is_relate_to_effect(duel, re) && rc.is_code(duel, CARD_UNKNOWN) {
            duel.negate_effect(a.ev as u8);
        }
    }
    Ok(())
}
