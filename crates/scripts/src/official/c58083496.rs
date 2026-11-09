// Odd-Eyes Override Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 58083496;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(3);

    fn specialchk(duel: &Duel, sg: CardSet, lc: CardId, sumtype: u64, tp: u8) -> bool {
        sg.is_exists(duel, |c, duel| c.is_type_full(duel, TYPE_PENDULUM, Some(lc), sumtype, tp), 1, None)
    }
}

struct ExtraPendulumSummon;

impl ExtraPendulum for ExtraPendulumSummon {
    const DESC: u64 = aux::stringid(ID, 2);
    const OWNER: u32 = ID;

    fn filter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_PERFORMAPAL, SET_ODD_EYES]) || (c.is_set_card(duel, &[SET_MAGICIAN]) && c.is_pendulum_monster(duel))
    }
}

// In the Extra Deck to Link Summon from Flash Knight (Pendulum Normal Monster) and Elfnote Power Patron on the field, on the field
// with Flash Knight to destroy and Power Patron Pendulum Monsters in the Pendulum Zone, and the opponent's Raigeki and
// Mystical Space Typhoon to respond to.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (17390179, MZONE),
    (12375297, MZONE),
    (17473466, SZONE),
    (43871165, SZONE),
    (CARD_POLYMERIZATION, GRAVE),
    (5318639, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];
// The extra Pendulum Summon is granted to the Pendulum Zone cards, so its flag is the check.
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::CardHint(aux::stringid(ID, 3)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY + CATEGORY_DRAW);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DISABLE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, |duel, a| {
        duel.is_turn_player(a.tp) && a.rp == 1 - a.tp && duel.is_chain_disablable(a.ev as u8)
    });
    e2.set_cost(duel, Cost::discard);
    e2.set_target(duel, |duel, a, chk, _| {
        if chk == 0 {
            return Ok(true);
        }
        duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, a.tp, 0);
        Ok(false)
    });
    e2.set_operation(duel, |duel, a| {
        duel.negate_effect(a.ev as u8);
        Ok(())
    });
    c.register_effect(duel, e2);
    let extra_pendulum_effect = Pendulum::create_additional_pendulum_summon_effect::<ExtraPendulumSummon>(duel, c);
    let e3a = duel.create_effect(c);
    e3a.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_GRANT);
    e3a.set_range(duel, LOCATION_MZONE);
    e3a.set_target_range(duel, LOCATION_SZONE, 0);
    e3a.set_target_card(duel, |duel, _, c| c.is_location(duel, LOCATION_PZONE));
    e3a.set_condition(duel, |duel, a| a.e.get_handler(duel).has_flag_effect(duel, ID));
    e3a.set_label_object(duel, Some(extra_pendulum_effect));
    c.register_effect(duel, e3a);
    let harmonic_effect = Pendulum::create_harmonic_oscillation_effect::<ExtraPendulumSummon>(duel, c);
    let e3b = e3a.clone_effect(duel);
    e3b.set_label_object(duel, Some(harmonic_effect));
    c.register_effect(duel, e3b);
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_on_field(duel) && chkc != c);
    }
    if chk == 0 {
        return Ok(duel.is_player_can_draw(tp, 1) && duel.is_existing_target(aux::TRUE, tp, LOCATION_ONFIELD, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, aux::TRUE, tp, LOCATION_ONFIELD, 0, 1, 1, Some(c.into()))?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 1);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && duel.destroy(tc, REASON_EFFECT)? > 0 {
        duel.draw(tp, 1, REASON_EFFECT)?;
        let c = e.get_handler(duel);
        if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) && tc.is_original_type(duel, TYPE_PENDULUM) {
            c.register_flag_effect_full(
                duel,
                ID,
                RESETS_STANDARD_PHASE_END,
                EFFECT_FLAG_CLIENT_HINT,
                1,
                0,
                aux::stringid(ID, 3),
            );
        }
    }
    Ok(())
}
