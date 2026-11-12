// Night's End Administrator
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 3779493;
const NIGHTS_END_SORCERER: u32 = 36107810;

pub const MATERIAL: &[u32] = &[NIGHTS_END_SORCERER];
pub const LISTED_NAMES: &[u32] = &[NIGHTS_END_SORCERER];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_summon_code(duel, Some(sc), sumtype, tp, &[NIGHTS_END_SORCERER])
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// In the Extra Deck, Synchro Summoned with Night's End Sorcerer and Mystical Elf (Level 4), with the opponent's Dark Hole to destroy it
// and a Mystical Elf in the Graveyard to revive, and Sangan on the opponent's side to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (NIGHTS_END_SORCERER, MZONE),
    (NIGHTS_END_SORCERER, MZONE),
    (15025844, MZONE),
    (15025844, MZONE),
    (15025844, GRAVE),
    (15025844, GRAVE),
    (26202165, MZONE | OPPONENT),
    (26202165, GRAVE | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_condition(duel, rmvcon);
    e1.set_target(duel, rmvtg);
    e1.set_operation(duel, rmvop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e2.set_code(duel, EVENT_DESTROYED);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP>);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACE_SPELLCASTER) && c.is_controler(duel, tp)
}

fn rmvcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    let eg = a.eg.unwrap_or_default();
    eg.is_contains(c) || eg.is_exists(duel, |tc, duel| cfilter(tc, duel, a.tp), 1, Some(c.into()))
}

fn rmfilter(c: CardId, duel: &Duel) -> bool {
    c.is_able_to_remove(duel) && aux::sp_elim_filter(c, duel, false, false)
}

fn rmvtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE | LOCATION_GRAVE) && rmfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(rmfilter, tp, 0, LOCATION_MZONE | LOCATION_GRAVE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(tp, rmfilter, tp, 0, LOCATION_MZONE | LOCATION_GRAVE, 1, 1, None)?;
    let location = g.and_then(|g| g.get_first()).map_or(0, |tc| tc.get_location(duel));
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, 0, location as i32);
    Ok(false)
}

fn rmvop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.remove(tc, POS_FACEUP, REASON_EFFECT)?;
    }
    Ok(())
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    (a.rp == 1 - a.tp
        && c.is_reason(duel, REASON_EFFECT)
        && c.is_previous_controler(duel, a.tp)
        && c.is_previous_location(duel, LOCATION_MZONE))
        || c.is_reason(duel, REASON_BATTLE)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_SPELLCASTER) && c.is_level_below(duel, 4) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    Ok(false)
}
