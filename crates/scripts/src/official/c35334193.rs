// S-Force Justify
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 35334193;

// Link Summoned from S-Force Edge Razor and two effect monsters, with many opponent's monsters to negate, attack and banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (91864689, MZONE),
    (91864689, MZONE),
    (3723262, MZONE),
    (42429678, MZONE),
    (32295838, MZONE | OPPONENT),
    (32295838, MZONE | OPPONENT),
    (32295838, MZONE | OPPONENT),
    (36211150, MZONE | OPPONENT),
];
// The banish effect needs it to attack with an opponent's monster in a zone it points to, which random play rarely does.
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Report(&Check::ResolvesEffect(aux::stringid(ID, 1))),
];

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 3;
    const MAX: Option<usize> = Some(3);

    fn f(c: CardId, duel: &Duel, lc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_type_full(duel, TYPE_EFFECT, Some(lc), sumtype, tp)
    }

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(
            duel,
            |c, duel| c.is_set_card_full(duel, &[SET_S_FORCE], Some(lc), sumtype, tp),
            1,
            None,
        )
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_FORCE_MZONE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 1, 0);
    e1.set_value(
        duel,
        Value::Zones(|duel, e, _, _, _| !(e.get_handler(duel).get_linked_zone(duel) as i32)),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DISABLE);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_REMOVE);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_BATTLE_START);
    e3.set_condition(duel, |duel, a| duel.get_attacker() == Some(a.e.get_handler(duel)));
    e3.set_target(duel, rmtg);
    e3.set_operation(duel, rmop);
    c.register_effect(duel, e3);
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_negatable_monster(duel) && chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_MZONE));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_negatable_monster, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, CardId::is_negatable_monster, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && tc.is_faceup(duel) && !tc.is_disabled(duel) && !tc.is_immune_to_effect(duel, e) {
        tc.negate_effects(duel, c, RESET_PHASE | PHASE_END, false);
        if c.is_in_extra_mzone_full(duel, tp) {
            let zone = c.get_linked_zone_full(duel, 1 - tp);
            let (nz, filter) = duel.get_location_count_flag(1 - tp, LOCATION_MZONE, 1 - tp, LOCATION_REASON_TOFIELD, zone);
            if nz > 0 && duel.select_yes_no(tp, aux::stringid(ID, 2))? {
                duel.break_effect()?;
                duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOZONE);
                if let Some(zones) = duel.select_disable_field(tp, 1, 0, LOCATION_MZONE as u16, filter << 16)? {
                    duel.move_sequence(tc, (zones >> 16).trailing_zeros() as u8)?;
                }
            }
        }
    }
    Ok(())
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let lg =
        a.e.get_handler(duel)
            .get_linked_group(duel)
            .filter(duel, CardId::is_able_to_remove, None);
    if chk == 0 {
        return Ok(!lg.is_empty());
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(lg), lg.len() as u8, 0, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let lg =
        a.e.get_handler(duel)
            .get_linked_group(duel)
            .filter(duel, CardId::is_able_to_remove, None);
    duel.remove(lg, POS_FACEUP, REASON_EFFECT)?;
    Ok(())
}
