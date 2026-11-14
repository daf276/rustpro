// Angelechy Seneschal
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 101402091;

pub const LISTED_SERIES: &[u16] = &[SET_ANGELECHY];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// In the Extra Deck, Synchro Summoned with Angelechy Enlisted, a Level 2 Tuner, and Tri-Horned Dragon, Level 8, with Angelechy
// Destrier in both Extra Decks to summon; on the field with Destrier on the opponent's field to attack it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (ID, MZONE),
    (81797573, MZONE),
    (81797573, MZONE),
    (39111158, MZONE),
    (39111158, MZONE),
    (55393975, EXTRA),
    (55393975, EXTRA | OPPONENT),
    (55393975, MZONE | OPPONENT),
    (55393975, MZONE | OPPONENT),
    (81797573, MZONE | OPPONENT),
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
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_DISABLE + CATEGORY_TOEXTRA);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_CONTROL);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_BE_BATTLE_TARGET);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        a.e.get_handler(duel)
            .get_battle_target(duel)
            .is_some_and(|bc| bc.is_set_card(duel, &[SET_ANGELECHY]))
    });
    e2.set_target(duel, atktg);
    e2.set_operation(duel, atkop);
    c.register_effect(duel, e2);
}

fn sptg(duel: &mut Duel, _: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_possible_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, PLAYER_EITHER, LOCATION_EXTRA as i32);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_race(duel, RACE_FAIRY)
        && c.is_synchro_monster(duel)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && duel.get_location_count_from_ex(tp, tp, None, c) > 0
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let c = e.get_handler(duel);
    let turn_player = duel.get_turn_player();
    for player in [turn_player, 1 - turn_player] {
        let g = duel.get_matching_group(|sc, duel| spfilter(sc, duel, e, player), player, LOCATION_EXTRA, 0, None);
        if !g.is_empty() && duel.select_yes_no(player, aux::stringid(ID, 2))? {
            duel.hint(HINT_SELECTMSG, player, HINTMSG_SPSUMMON);
            if let Some(sc) = g.select(duel, player, 1, 1, None)?.get_first()
                && duel.special_summon_step(sc, 0, player, player, false, false, POS_FACEUP)?
            {
                sc.negate_effects(duel, c, 0, false);
                applyrestrictions(duel, sc, c);
            }
        }
    }
    duel.special_summon_complete()?;
    if c.is_relate_to_effect(duel, e) {
        applyrestrictions(duel, c, c);
    }
    Ok(())
}

fn applyrestrictions(duel: &mut Duel, c: CardId, rc: CardId) {
    let e1 = duel.create_effect(rc);
    e1.set_description(duel, aux::stringid(ID, 3));
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_BE_MATERIAL);
    e1.set_value(
        duel,
        Value::Material(|_, _, _, sumtype, _| aux::cannotmatfilter(&[SUMMON_TYPE_XYZ, SUMMON_TYPE_LINK].map(u64::from), sumtype)),
    );
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    c.register_effect_full(duel, e1, true);
    let e2 = duel.create_effect(rc);
    e2.set_description(duel, aux::stringid(ID, 4));
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
    e2.set_code(duel, EFFECT_LEAVE_FIELD_REDIRECT);
    e2.set_value(duel, LOCATION_DECKBOT as i32);
    e2.set_reset(duel, RESET_EVENT | RESETS_REDIRECT);
    c.register_effect_full(duel, e2, true);
}

fn atktg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let c = a.e.get_handler(duel);
    duel.set_operation_info(0, CATEGORY_CONTROL, Some(c.into()), 1, a.tp, 0);
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    if !duel.negate_attack()? {
        return Ok(());
    }
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) {
        duel.get_control(c, 1 - a.tp)?;
    }
    Ok(())
}
