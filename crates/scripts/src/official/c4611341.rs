// S-Force Mystify
use crate::prelude::*;
use crate::testing::{DECK, EXTRA, MZONE, OPPONENT};

const ID: u32 = 4611341;

pub const LISTED_SERIES: &[u16] = &[SET_S_FORCE];
// Link Summoned from S-Force Edge Razor and a Cyberse monster, with opponent's monsters to fight.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (91864689, MZONE),
    (91864689, MZONE),
    (47226949, MZONE),
    (32295838, MZONE),
    (91864689, DECK),
    (89631139, MZONE | OPPONENT),
    (32295838, MZONE | OPPONENT),
];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(3);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(
            duel,
            |c, duel| c.is_set_card_full(duel, &[SET_S_FORCE], Some(lc), sumtype, tp),
            1,
            None,
        )
    }
}

pub const MATERIAL_SETCODE: &[u16] = &[SET_S_FORCE];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1a = duel.create_effect(c);
    e1a.set_type(duel, EFFECT_TYPE_SINGLE);
    e1a.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1a.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e1a.set_range(duel, LOCATION_MZONE);
    e1a.set_condition(duel, columncon);
    e1a.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e1a);
    let e1b = duel.create_effect(c);
    e1b.set_type(duel, EFFECT_TYPE_FIELD);
    e1b.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1b.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1b.set_range(duel, LOCATION_MZONE);
    e1b.set_target_range(duel, 0, 1);
    e1b.set_condition(duel, columncon);
    e1b.set_target_spsummon(duel, |duel, _, _, _, _, _, _, se| {
        se.is_some_and(|se| se.is_activated(duel) && se.is_monster_effect(duel))
    });
    c.register_effect(duel, e1b);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, mvtg);
    e2.set_operation(duel, mvop);
    e2.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e2);
}

fn columncon(duel: &Duel, a: &Args) -> bool {
    let e = a.e;
    e.get_handler(duel).get_column_group(duel).is_exists(
        duel,
        |c, duel| c.is_location(duel, LOCATION_MZONE) && c.is_controler(duel, 1 - e.get_handler_player(duel)),
        1,
        None,
    )
}

fn mvtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let own_loc = if duel.get_location_count(tp, LOCATION_MZONE) > 0 {
        LOCATION_MZONE
    } else {
        0
    };
    let opp_loc = if duel.get_location_count(1 - tp, LOCATION_MZONE) > 0 {
        LOCATION_MZONE
    } else {
        0
    };
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && ((chkc.is_controler(duel, tp) && own_loc > 0) || opp_loc > 0));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(aux::TRUE, tp, own_loc, opp_loc, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 1));
    duel.select_target(tp, aux::TRUE, tp, own_loc, opp_loc, 1, 1, None)?;
    Ok(false)
}

fn mvop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) || tc.is_immune_to_effect(duel, e) {
        return Ok(());
    }
    let player = tc.get_controler(duel);
    if duel.get_location_count(player, LOCATION_MZONE) > 0 {
        let is_own = player == tp;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOZONE);
        let (own, opp) = if is_own { (LOCATION_MZONE, 0) } else { (0, LOCATION_MZONE) };
        let Some(mut zone) = duel.select_disable_field(tp, 1, own as u16, opp as u16, 0)? else {
            return Ok(());
        };
        if !is_own {
            zone >>= 16;
        }
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOZONE);
        duel.move_sequence(tc, zone.trailing_zeros() as u8)?;
    }
    Ok(())
}
