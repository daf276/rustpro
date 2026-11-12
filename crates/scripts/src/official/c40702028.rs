// DPH Gendamoore
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE, OPPONENT};

const ID: u32 = 40702028;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// In the Extra Deck and Graveyard, Synchro Summoned from Mathmech Sigma (Level 4 Tuner) and Leotron (Level 4 Normal Monster, also
// in the Graveyard to revive) as the Cyberse monsters, with Elfnote Lucina on the opponent's side to attack.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, GRAVE),
    (27182739, MZONE),
    (47226949, MZONE),
    (47226949, MZONE),
    (47226949, GRAVE),
    (13597785, MZONE | OPPONENT),
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
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_synchro_summoned(duel));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DAMAGE);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_BATTLE_DAMAGE);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, damcon);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, damtg);
    e2.set_operation(duel, stock::damage);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_CYBERSE)
        && c.is_level(duel, &[4])
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_GRAVE)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if duel.get_location_count(tp, LOCATION_MZONE) > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
        if let Some(sc) = g.get_first()
            && duel.special_summon_step(sc, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?
        {
            sc.negate_effects(duel, c, 0, false);
        }
        duel.special_summon_complete()?;
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| !c.is_race(duel, RACE_CYBERSE));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    if a.ep == a.tp {
        return false;
    }
    let (ac, bc) = duel.get_battle_monster(a.tp);
    ac.is_some_and(|ac| ac.is_race(duel, RACE_CYBERSE)) && bc.is_some()
}

fn damtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (tp, ev) = (a.tp, a.ev);
    if chk == 0 {
        return Ok(true);
    }
    duel.set_target_player(1 - tp);
    duel.set_target_param(ev);
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - tp, ev as i32);
    Ok(false)
}
