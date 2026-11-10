// V-LAN Hydra
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 13536606;

pub const LISTED_NAMES: &[u32] = &[ID + 1];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        !c.is_type_full(duel, TYPE_TOKEN, Some(lc), sumtype, tp)
    }
}

// In the Extra Deck and on the field, hoping for a Link Summon that links two copies with each other.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (ID, EXTRA), (ID, MZONE), (11091375, MZONE), (43096270, MZONE)];
pub const CHECKS: &[Check] = &[Check::SpecialSummonedCode(ID + 1)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, tktg);
    e2.set_operation(duel, tkop);
    c.register_effect(duel, e2);
}

fn token_stats() -> MonsterStats {
    MonsterStats::new(0, TYPES_TOKEN, 0, 0, 1, RACE_CYBERSE, ATTRIBUTE_LIGHT)
}

fn atkval(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    c.get_mutual_linked_group(duel).filter_count(duel, CardId::is_monster, None) as i32 * 300
}

fn rfilter(c: CardId, duel: &Duel, tp: u8, g: CardSet) -> bool {
    let lk = c.get_link(duel) as i32;
    let ft = duel.get_location_count(tp, LOCATION_MZONE);
    let releasable = |duel: &Duel| {
        c.is_faceup(duel) && c.is_link_monster(duel) && c.is_link_below(duel, 3) && c.is_releasable_by_effect(duel) && g.is_contains(c)
    };
    let spirit = |duel: &Duel| ft == 1 || duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none();
    if c.is_controler(duel, tp) {
        releasable(duel) && ((c.get_sequence(duel) > 4 && ft >= lk) || (c.get_sequence(duel) <= 4 && ft + 1 >= lk)) && spirit(duel)
    } else if c.is_controler(duel, 1 - tp) {
        releasable(duel) && ft >= lk && spirit(duel)
    } else {
        false
    }
}

fn tktg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    let lg = c.get_mutual_linked_group(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && rfilter(chkc, duel, tp, lg));
    }
    if chk == 0 {
        return Ok(
            duel.is_existing_target(|c, duel| rfilter(c, duel, tp, lg), tp, LOCATION_MZONE, LOCATION_MZONE, 1, None)
                && duel.is_player_can_special_summon_monster(tp, ID + 1, token_stats()),
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RELEASE);
    let rg = duel.select_target(
        tp,
        |c, duel| rfilter(c, duel, tp, lg),
        tp,
        LOCATION_MZONE,
        LOCATION_MZONE,
        1,
        1,
        None,
    )?;
    let ct = rg.and_then(|rg| rg.get_first()).map_or(0, |tc| tc.get_link(duel)) as u8;
    duel.set_operation_info(0, CATEGORY_TOKEN, None, ct, 0, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, ct, tp, 0);
    Ok(false)
}

fn tkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    let ct = tc.get_link(duel);
    if tc.is_relate_to_effect(duel, e) && !tc.is_immune_to_effect(duel, e) && duel.release(tc, REASON_EFFECT)? > 0 {
        let ft = duel.get_location_count(tp, LOCATION_MZONE);
        if ft < ct as i32 || (ft > 1 && duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some()) {
            return Ok(());
        }
        if !duel.is_player_can_special_summon_monster(tp, ID + 1, token_stats()) {
            return Ok(());
        }
        for _ in 0..ct {
            if let Some(token) = duel.create_token(tp, ID + 1) {
                duel.special_summon_step(token, 0, tp, tp, false, false, POS_FACEUP)?;
            }
        }
        duel.special_summon_complete()?;
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    e1.set_target_range(duel, 1, 0);
    e1.set_label(duel, i64::from(ct));
    e1.set_target_card(duel, |duel, e, c| c.is_link(duel, &[e.get_label(duel) as u32]));
    duel.register_effect(e1, tp);
    Ok(())
}
