// Dark End Dragon
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE, OPPONENT};

const ID: u32 = 88643579;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_attribute_full(duel, ATTRIBUTE_DARK, Some(sc), sumtype, tp)
    }
}

// In the Extra Deck, Synchro Summoned from Plaguespreader Zombie (Tuner) and Beast of Talwar or Water Spirit (Tuner) and Zoa,
// against the opponent's Sangans.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (33420078, MZONE),
    (11761845, MZONE),
    (487395, MZONE),
    (24311372, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE) && chkc.is_able_to_grave(duel));
    }
    if chk == 0 {
        return Ok(c.get_attack(duel) >= 500
            && c.get_defense(duel) >= 500
            && duel.is_existing_target(CardId::is_able_to_grave, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_target(tp, CardId::is_able_to_grave, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOGRAVE, g, g.map_or(0, |g| g.len()) as u8, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let tc = duel.get_first_target();
    if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) && c.get_attack(duel) >= 500 && c.get_defense(duel) >= 500 {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_COPY_INHERIT);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD_DISABLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, -500);
        c.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_UPDATE_DEFENSE);
        c.register_effect(duel, e2);
        if let Some(tc) = tc
            && tc.is_controler(duel, 1 - tp)
            && tc.is_relate_to_effect(duel, e)
            && c.is_has_effect(duel, EFFECT_REVERSE_UPDATE).is_none()
        {
            duel.send_to_grave(tc, REASON_EFFECT)?;
        }
    }
    Ok(())
}
