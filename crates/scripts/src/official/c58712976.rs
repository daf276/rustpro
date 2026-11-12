// Infernal Flame Vixen
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 58712976;

// In the Extra Deck, Xyz Summoned with Level 4 Pyro Normal Monsters (Darkfire Soldier #1, Blazing Inpachi), with Dark Hole in the
// opponent's hand to destroy it, and monsters in the Graveyard to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (5388481, MZONE),
    (5388481, MZONE),
    (5464695, MZONE),
    (5464695, MZONE),
    (5464695, GRAVE),
    (5388481, GRAVE),
    (26202165, MZONE | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_race_full(duel, RACE_PYRO, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_count_limit(duel, 1, 0);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).get_previous_attack_on_field(duel) >= 2500);
    e2.set_target(duel, rmtg);
    e2.set_operation(duel, rmop);
    c.register_effect(duel, e2);
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && c.is_faceup(duel) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_COPY_INHERIT);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, 500);
        e1.set_reset_full(duel, RESETS_STANDARD_DISABLE_PHASE_END, 2);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_able_to_remove(duel) && aux::sp_elim_filter(c, duel, false, false)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE | LOCATION_GRAVE) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(true);
    }
    let locations = LOCATION_MZONE | LOCATION_GRAVE;
    if duel.is_existing_target(filter, tp, locations, locations, 3, None) {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
        let g = duel.select_target(tp, filter, tp, locations, locations, 3, 3, None)?;
        duel.set_operation_info(0, CATEGORY_REMOVE, g, 3, 0, 0);
    }
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(g) = duel.get_chain_info::<Option<CardSet>>(0, CHAININFO_TARGET_CARDS) else {
        return Ok(());
    };
    let rg = g.filter(duel, |c, duel| c.is_relate_to_effect(duel, a.e), None);
    if !rg.is_empty() {
        duel.remove(rg, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}
