// Adamancipator Luminous
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 41458361;

pub const LISTED_SERIES: &[u16] = &[SET_ADAMANCIPATOR];

const TOKEN_GLIMMER: u32 = ID + 1;

pub const LISTED_NAMES: &[u32] = &[TOKEN_GLIMMER];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(duel, |c, duel| c.is_type_full(duel, TYPE_SYNCHRO, Some(lc), sumtype, tp), 1, None)
    }
}

// Link Summoned from two Synchro Monsters and two Effect Monsters, which are also the Levels it copies.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (47674738, MZONE),
    (73079836, MZONE),
    (48519867, MZONE),
    (85914562, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1a = duel.create_effect(c);
    e1a.set_description(duel, aux::stringid(ID, 0));
    e1a.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e1a.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1a.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e1a.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1a.set_count_limit(duel, 1, (ID, 0));
    e1a.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e1a.set_target(duel, tkntg);
    e1a.set_operation(duel, tknop);
    c.register_effect(duel, e1a);
    let e1b = e1a.clone_effect(duel);
    e1b.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1b.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DAMAGE_STEP);
    e1b.set_range(duel, LOCATION_MZONE);
    e1b.set_condition(duel, |duel, a| {
        aux::zptgroupcon(duel, a.eg.unwrap_or_default(), aux::TRUE, a.e.get_handler(duel))
    });
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TODECK);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, tdcon);
    e2.set_target(duel, tdtg);
    e2.set_operation(duel, tdop);
    c.register_effect(duel, e2);
}

fn token_stats(level: u32) -> MonsterStats {
    MonsterStats::new(0, TYPES_TOKEN, 0, 0, level, RACE_ROCK, ATTRIBUTE_LIGHT)
}

fn tgfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.has_level(duel)
        && duel.check_mut(|duel| duel.is_player_can_special_summon_monster(tp, TOKEN_GLIMMER, token_stats(c.get_level(duel))))
}

fn tkntg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let location = LOCATION_MZONE | LOCATION_GRAVE;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, location) && chkc.is_controler(duel, tp) && tgfilter(chkc, duel, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| tgfilter(c, duel, tp), tp, location, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, |c, duel| tgfilter(c, duel, tp), tp, location, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, 0);
    Ok(false)
}

fn tknop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e)
        && tc.is_faceup(duel)
        && duel.is_player_can_special_summon_monster(tp, TOKEN_GLIMMER, token_stats(tc.get_level(duel)))
        && let Some(token) = duel.create_token(tp, TOKEN_GLIMMER)
    {
        token.set_level(duel, tc.get_level(duel));
        duel.special_summon(token, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn tdcon(duel: &Duel, a: &Args) -> bool {
    a.re.is_some_and(|re| re.is_monster_effect(duel) && re.get_handler(duel) != a.e.get_handler(duel))
        && chain::is_triggering_location(duel, a.ev as u8, LOCATION_MZONE)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let Some(re) = a.re else { return Ok(false) };
    let rc = re.get_handler(duel);
    if chk == 0 {
        return Ok(rc.is_relate_to_effect(duel, re) && rc.is_able_to_deck(duel));
    }
    duel.set_operation_info(0, CATEGORY_TODECK, Some(rc.into()), 1, a.tp, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    let rc = re.get_handler(duel);
    if rc.is_relate_to_effect(duel, re) {
        duel.send_to_deck(rc, PLAYER_NONE, SEQ_DECKTOP, REASON_EFFECT)?;
    }
    Ok(())
}
