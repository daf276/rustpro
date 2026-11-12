// Evigishki Merrowgeist
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE, OPPONENT};

const ID: u32 = 76372778;

// Xyz Summoned from the Extra Deck with two Level 4 monsters (Barrel Rock), battling the opponent's weak monsters (Key Mace).
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (10476868, MZONE),
    (10476868, MZONE),
    (10476868, MZONE),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BATTLED);
    e1.set_condition(duel, condition);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    if c.is_status(duel, STATUS_BATTLE_DESTROYED) {
        return false;
    }
    c.get_battle_target(duel).is_some_and(|bc| {
        bc.is_status(duel, STATUS_BATTLE_DESTROYED) && !bc.is_type(duel, TYPE_TOKEN) && bc.get_leave_field_dest(duel) == 0
    })
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if let Some(bc) = c.get_battle_target(duel)
        && bc.is_relate_to_battle(duel)
    {
        let e1 = duel.create_effect(c);
        e1.set_code(duel, EFFECT_SEND_REPLACE);
        e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_CONTINUOUS);
        e1.set_target(duel, reptg);
        e1.set_operation(duel, repop);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_DAMAGE);
        bc.register_effect(duel, e1);
    }
    Ok(())
}

fn reptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.get_destination(duel) == LOCATION_GRAVE && c.is_reason(duel, REASON_BATTLE));
    }
    Ok(true)
}

fn repop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.send_to_deck(a.e.get_handler(duel), PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    Ok(())
}
