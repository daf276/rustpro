// Inzektor Exa-Stag
use crate::prelude::*;
use crate::testing::{EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 26211048;

// In the Extra Deck, Xyz Summoned from Level 5 Insect Normal Monsters (Dungeon Worm, Hercules Beetle, Hunter Spider), with Sangan on
// the opponent's side, on the field and in the Graveyard, to equip.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (51228280, MZONE),
    (51228280, MZONE),
    (52584282, MZONE),
    (52584282, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, GRAVE | OPPONENT),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_race_full(duel, RACE_INSECT, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(5);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_EQUIP);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, eqtg);
    e1.set_operation(duel, eqop);
    c.register_effect(duel, e1);
    aux::add_er_equip_limit(duel, c, None, e1);
}

fn eqfilter(c: CardId, duel: &Duel) -> bool {
    c.is_location(duel, LOCATION_MZONE) || (c.is_monster(duel) && !c.is_forbidden(duel))
}

fn eqtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE | LOCATION_MZONE) && chkc.is_controler(duel, 1 - tp) && eqfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_SZONE) > 0
            && duel.is_existing_target(eqfilter, tp, 0, LOCATION_GRAVE | LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_EQUIP);
    let g = duel.select_target(tp, eqfilter, tp, 0, LOCATION_GRAVE | LOCATION_MZONE, 1, 1, None)?;
    if g.and_then(|g| g.get_first()).is_some_and(|tc| tc.is_location(duel, LOCATION_GRAVE)) {
        duel.set_operation_info(0, CATEGORY_LEAVE_GRAVE, g, 1, 0, 0);
    }
    Ok(false)
}

pub fn equipop(duel: &mut Duel, c: CardId, e: EffectId, tp: u8, tc: CardId) -> Script<()> {
    if !c.equip_by_effect_and_limit_register(duel, e, tp, tc, None, None)? {
        return Ok(());
    }
    if tc.is_faceup(duel) {
        for (code, text) in [
            (EFFECT_UPDATE_ATTACK, tc.get_text_attack(duel)),
            (EFFECT_UPDATE_DEFENSE, tc.get_text_defense(duel)),
        ] {
            let e2 = duel.create_effect(c);
            e2.set_type(duel, EFFECT_TYPE_EQUIP);
            e2.set_code(duel, code);
            e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            e2.set_value(duel, ((text + 1) / 2).max(0));
            tc.register_effect(duel, e2);
        }
    }
    Ok(())
}

fn eqop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(tc) = duel.get_first_target()
        && tc.is_monster(duel)
    {
        equipop(duel, c, e, tp, tc)?;
    }
    Ok(())
}
