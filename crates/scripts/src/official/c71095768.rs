// Bujinki Ahashima
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 71095768;
const GANTETSU: u32 = 10002346;
const CHAIN_ID: u64 = (ID as u64) << 16;
const LOCATION_KEY: u64 = CHAIN_ID | 1;
const SEQUENCE_KEY: u64 = CHAIN_ID | 2;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, _: CardId, _: u64, _: u8) -> bool {
        c.is_level_above(duel, 1)
    }

    fn specialchk(duel: &Duel, sg: CardSet, _: CardId, _: u64, _: u8) -> bool {
        sg.get_class_count(duel, |c, duel| i64::from(c.get_level(duel))) == 1
    }
}

// Link Summoned from two Level 2 Normal Monsters, with Level 2 monsters in the hand and Graveyard and Gachi Gachi Gantetsu in the Extra Deck
// to Xyz Summon. Destroying needs an Xyz monster to pay a cost by detaching, which none of these does.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, MZONE),
    (756652, MZONE),
    (756652, MZONE),
    (2830619, MZONE),
    (3985011, HAND),
    (756652, HAND),
    (2830619, GRAVE),
    (3985011, GRAVE),
    (GANTETSU, EXTRA),
    (GANTETSU, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(GANTETSU), Check::Report(&Check::ResolvesEffect(0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    duel.enable_global_flag(GLOBALFLAG_DETACH_EVENT);
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_CANNOT_BE_LINK_MATERIAL);
    e1.set_value(duel, 1);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_link_summoned(duel));
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_category(duel, CATEGORY_DESTROY);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_CHAINING);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, descon);
    e3.set_target(duel, destg);
    e3.set_operation(duel, stock::destroy_first_target);
    c.register_effect(duel, e3);
    aux::double_snare_validity(duel, c, LOCATION_MZONE);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_DETACH_MATERIAL);
        ge1.set_operation(duel, checkop);
        duel.register_effect(ge1, 0);
    });
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_above(duel, 1) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn xyzchk(c: CardId, duel: &Duel, sg: CardSet, tp: u8) -> bool {
    c.is_xyz_summonable_full(duel, None, Some(sg), 2, 2) && duel.get_location_count_from_ex(tp, tp, Some(sg), c) > 0
}

fn spcheck(duel: &Duel, sg: CardSet, tp: u8) -> (bool, bool) {
    let res = sg.get_class_count(duel, |c, duel| i64::from(c.get_location(duel))) == sg.len()
        && sg.get_class_count(duel, |c, duel| i64::from(c.get_level(duel))) == 1
        && duel.is_existing_matching_card(|c, duel| xyzchk(c, duel, sg, tp), tp, LOCATION_EXTRA, 0, 1, None);
    (res, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE | LOCATION_HAND, 0, None);
        return Ok(duel.is_player_can_special_summon_count(tp, 2)
            && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && duel.get_location_count(tp, LOCATION_MZONE) > 1
            && aux::select_unselect_group_check(duel, g, 2, 2, &|duel, sg, _, _| spcheck(duel, sg, tp)));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, tp, (LOCATION_HAND | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() || duel.get_location_count(tp, LOCATION_MZONE) < 2 {
        return Ok(());
    }
    let g = duel.get_matching_group(
        aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp)),
        tp,
        LOCATION_GRAVE | LOCATION_HAND,
        0,
        None,
    );
    let sg = aux::select_unselect_group(duel, g, 2, 2, &|duel, sg, _, _| spcheck(duel, sg, tp), tp, HINTMSG_SPSUMMON)?;
    if sg.len() != 2 {
        return Ok(());
    }
    for tc in sg {
        duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP)?;
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_DISABLE);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
        let e2 = e1.clone_effect(duel);
        e2.set_code(duel, EFFECT_DISABLE_EFFECT);
        tc.register_effect(duel, e2);
    }
    duel.special_summon_complete()?;
    duel.break_effect()?;
    let xyzg = duel.get_matching_group(|c, duel| xyzchk(c, duel, sg, tp), tp, LOCATION_EXTRA, 0, None);
    if !xyzg.is_empty() {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        if let Some(xyz) = xyzg.select(duel, tp, 1, 1, None)?.get_first() {
            duel.xyz_summon_full(tp, xyz, Some(sg), None, 2, 2)?;
        }
    }
    Ok(())
}

// The chain id, location and sequence of the last cost that detached a material; the sequence is kept plus one so that 0 is the Lua's nil.
fn checkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let cid = duel.get_current_chain();
    if cid > 0 && a.r & REASON_COST == REASON_COST {
        let chain_id = duel.get_chain_info::<u32>(cid, CHAININFO_CHAIN_ID);
        let location = duel.get_chain_info::<u32>(cid, CHAININFO_TRIGGERING_LOCATION);
        duel.set_script_state(CHAIN_ID, chain_id.into());
        duel.set_script_state(LOCATION_KEY, location.into());
        let mut seq = duel.get_chain_info::<u32>(cid, CHAININFO_TRIGGERING_SEQUENCE);
        let Some(te) = duel.get_chain_info::<Option<EffectId>>(cid, CHAININFO_TRIGGERING_EFFECT) else {
            return Ok(());
        };
        let (tc, p) = (te.get_handler(duel), a.e.get_handler(duel).get_controler(duel));
        let opponent = if tc.is_relate_to_effect(duel, te) {
            tc.is_controler(duel, 1 - p)
        } else {
            tc.get_previous_controler(duel) == 1 - p
        };
        if opponent {
            seq += 16;
        }
        duel.set_script_state(SEQUENCE_KEY, i64::from(seq) + 1);
    }
    Ok(())
}

fn descon(duel: &Duel, a: &Args) -> bool {
    let (Some(re), c) = (a.re, a.e.get_handler(duel)) else {
        return false;
    };
    let (loc, seq) = (duel.get_script_state(LOCATION_KEY), duel.get_script_state(SEQUENCE_KEY) - 1);
    if c.is_status(duel, STATUS_BATTLE_DESTROYED) || seq < 0 {
        return false;
    }
    i64::from(duel.get_chain_info::<u32>(a.ev as u8, CHAININFO_CHAIN_ID)) == duel.get_script_state(CHAIN_ID)
        && re.is_active_type(duel, TYPE_XYZ)
        && loc & i64::from(LOCATION_MZONE) != 0
        && i64::from(c.get_linked_zone(duel)) >> seq & 1 != 0
}

fn desfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp) && desfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(desfilter, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, desfilter, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}
