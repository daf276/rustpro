// Port of proc_synchro.lua: Synchro.AddProcedure with its condition, target and operation. The paths for materials
// with EFFECT_SYNCHRO_CHECK or EFFECT_HAND_SYNCHRO (whose checks register and reset effects or assume properties), a
// Synchro Monster's EFFECT_MATERIAL_CHECK operation, the Majestic and Dark Synchro procedures and Synchro.Send come with
// the cards that use them.
use engine::{Args, CardId, CardSet, Duel, EffectId, Operation, Script, Value};

use crate::constant::*;
use crate::utility::aux;

pub struct Synchro;

// The parameters of Synchro.AddProcedure (f1, min1, max1, f2, min2, max2, sub1, sub2, req1, req2, reqm): a filter left
// nil passes every card, a nil substitute none.
pub trait SynchroProcedure {
    const MIN1: usize = 1;
    const MAX1: usize = 1;
    const MIN2: usize;
    const MAX2: usize;
    const REQ1: Option<Requirement> = None;
    const REQ2: Option<Requirement> = None;
    const REQM: Option<Requirement> = None;

    fn f1(_c: CardId, _duel: &Duel, _sc: CardId, _sumtype: u64, _tp: u8) -> bool {
        true
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool;

    fn sub1(_c: CardId, _duel: &Duel, _sc: CardId, _sumtype: u64, _tp: u8) -> bool {
        false
    }

    fn sub2(_c: CardId, _duel: &Duel, _sc: CardId, _sumtype: u64, _tp: u8) -> bool {
        false
    }
}

// req1, req2 or reqm: whether the chosen Tuners, non-Tuners or all materials (g, sc, tp) may be used.
pub type Requirement = fn(&Duel, CardSet, CardId, u8) -> bool;

fn req(requirement: Option<Requirement>, duel: &Duel, g: CardSet, sc: CardId, tp: u8) -> bool {
    requirement.is_none_or(|requirement| requirement(duel, g, sc, tp))
}

const SYNCHRO_MATERIAL: u64 = SUMMON_TYPE_SYNCHRO as u64 | MATERIAL_SYNCHRO;

// What the recursive checks share: the Synchro Monster's level, the monster and its controller, the materials it must
// use, whether only given materials may be used, and the forced minimum and maximum (0 for nil).
struct Search {
    lv: u32,
    sc: CardId,
    tp: u8,
    pg: CardSet,
    mgchk: bool,
    min: usize,
    max: usize,
    levels: Levels,
    plain: bool,
}

// The Synchro levels of up to 16 materials, read once per condition or target call instead of at every CheckP43: neither
// changes a level before its last read, and a repeated read looks up the same effects in the same state. Other cards are
// read when asked.
struct Levels([(Option<CardId>, u32); 16]);

impl Levels {
    fn new(duel: &Duel, g: CardSet, sc: CardId) -> Self {
        let mut levels = [(None, 0); 16];
        levels
            .iter_mut()
            .zip(g)
            .for_each(|(slot, c)| *slot = (Some(c), c.get_synchro_level(duel, sc)));
        Self(levels)
    }

    fn get(&self, duel: &Duel, c: CardId, sc: CardId) -> u32 {
        self.0
            .iter()
            .find(|(card, _)| *card == Some(c))
            .map_or_else(|| c.get_synchro_level(duel, sc), |&(_, level)| level)
    }
}

impl Synchro {
    // Synchro.NonTuner(f, ...): `filter` is f with its arguments.
    pub fn non_tuner(target: CardId, duel: &Duel, scard: CardId, tp: u8, filter: impl Fn(CardId, &Duel) -> bool) -> bool {
        target.is_not_tuner(duel, scard, tp) && filter(target, duel)
    }

    pub fn add_procedure<P: SynchroProcedure>(duel: &mut Duel, c: CardId) -> EffectId {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_description(duel, 1172);
        e1.set_code(duel, EFFECT_SPSUMMON_PROC);
        e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE + EFFECT_FLAG_IGNORE_IMMUNE);
        e1.set_range(duel, LOCATION_EXTRA);
        e1.set_condition(duel, Synchro::condition::<P>);
        e1.set_target(duel, Synchro::target::<P>);
        e1.set_operation(duel, Synchro::operation);
        e1.set_value(duel, SUMMON_TYPE_SYNCHRO as i32);
        c.register_effect(duel, e1);
        e1
    }

    // sub1 as the condition and target extend it: also a card that can be treated as a Tuner and passes f1.
    fn sub1<P: SynchroProcedure>(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        P::sub1(c, duel, sc, sumtype, tp)
            || (c.is_has_effect(duel, EFFECT_CAN_BE_TUNER).is_some() && P::f1(c, duel, sc, SYNCHRO_MATERIAL, tp))
    }

    fn tuner_filter<P: SynchroProcedure>(c: CardId, duel: &Duel, sc: CardId, tp: u8) -> bool {
        (c.is_type_full(duel, TYPE_TUNER, Some(sc), SYNCHRO_MATERIAL, tp) && P::f1(c, duel, sc, SYNCHRO_MATERIAL, tp))
            || Synchro::sub1::<P>(c, duel, sc, SYNCHRO_MATERIAL, tp)
    }

    fn non_tuner_filter<P: SynchroProcedure>(c: CardId, duel: &Duel, sc: CardId, tp: u8) -> bool {
        P::f2(c, duel, sc, SYNCHRO_MATERIAL, tp) || P::sub2(c, duel, sc, SYNCHRO_MATERIAL, tp)
    }

    // The materials both the condition and the target start from: (dg, g, pg, mgchk) and their levels.
    fn materials(duel: &Duel, c: CardId, tp: u8, mg: Option<CardSet>) -> (CardSet, CardSet, CardSet, bool, Levels) {
        let can_be_material = |mc: CardId, duel: &Duel| mc.is_can_be_synchro_material(duel, c);
        let (dg, mut g, mgchk) = match mg {
            Some(mg) => (mg, mg.filter(duel, can_be_material, Some(c.into())), true),
            None => {
                let synchmatfilter = |mc: CardId, duel: &Duel| {
                    let handmatfilter = mc.is_has_effect(duel, EFFECT_SYNCHRO_MAT_FROM_HAND);
                    (mc.is_location(duel, LOCATION_MZONE)
                        && mc.is_faceup(duel)
                        && (mc.is_controler(duel, tp) || mc.is_can_be_synchro_material(duel, c)))
                        || handmatfilter.is_some_and(|te| {
                            te.check_count_limit(duel, tp)
                                && matches!(te.get_value(duel), Value::MaterialFor(value) if value(duel, te, mc, c))
                        })
                };
                let dg = duel.get_matching_group(synchmatfilter, tp, LOCATION_MZONE | LOCATION_HAND, LOCATION_MZONE, Some(c.into()));
                (dg, dg.filter(duel, can_be_material, None), false)
            }
        };
        let levels = Levels::new(duel, g, c);
        g.remove_matching(duel, |mc, duel| levels.get(duel, mc, c) < 1, None);
        let pg = aux::get_must_be_material_group(duel, tp, Some(dg), tp, Some(c), Some(g), REASON_SYNCHRO);
        (dg, g, pg, mgchk, levels)
    }

    // The materials with EFFECT_SYNCHRO_CHECK or EFFECT_HAND_SYNCHRO, whose search the oracle runs instead.
    fn unported_materials(duel: &Duel, g: CardSet, mgchk: bool) {
        unported!(
            duel,
            g.is_exists(duel, |c, duel| c.is_has_effect(duel, EFFECT_SYNCHRO_CHECK).is_some(), 1, None)
        );
        unported!(
            duel,
            !mgchk && g.is_exists(duel, |c, duel| c.is_has_effect(duel, EFFECT_HAND_SYNCHRO).is_some(), 1, None)
        );
    }

    // The cards of the hand and GY with these effects get them reset at the end; none of the ported ones has them. Not
    // in the Lua: the cards are not searched while no effect has the code.
    fn reset_hand_synchro(duel: &Duel, tp: u8) {
        if !duel.may_have_effect(EFFECT_HAND_SYNCHRO + EFFECT_SYNCHRO_CHECK) {
            return;
        }
        let hg = duel.get_matching_group(
            |c, duel| c.is_has_effect(duel, EFFECT_HAND_SYNCHRO + EFFECT_SYNCHRO_CHECK).is_some(),
            tp,
            LOCATION_HAND + LOCATION_GRAVE,
            0,
            None,
        );
        unported!(duel, !hg.is_empty());
    }

    fn condition<P: SynchroProcedure>(duel: &Duel, a: &Args) -> bool {
        let (smat, mg) = (a.must, a.materials);
        let Some(c) = a.c else { return true };
        if c.is_type(duel, TYPE_PENDULUM) && c.is_faceup(duel) {
            return false;
        }
        let tp = c.get_controler(duel);
        let (_, mut g, mut pg, mgchk, levels) = Synchro::materials(duel, c, tp, mg);
        let can_be_material = |mc: CardId, duel: &Duel| mc.is_can_be_synchro_material(duel, c);
        if !g.includes(pg) || pg.is_exists(duel, |mc, duel| !can_be_material(mc, duel), 1, None) {
            return false;
        }
        if let Some(smat) = smat {
            if smat.is_exists(duel, |mc, duel| !can_be_material(mc, duel), 1, None) {
                return false;
            }
            pg.merge(smat);
            g.merge(smat);
        }
        Synchro::unported_materials(duel, g, mgchk);
        let tg = g.filter(duel, |mc, duel| Synchro::tuner_filter::<P>(mc, duel, c, tp), None);
        let ntg = g.filter(duel, |mc, duel| Synchro::non_tuner_filter::<P>(mc, duel, c, tp), None);
        let search = Search {
            lv: c.get_level(duel),
            sc: c,
            tp,
            pg,
            mgchk,
            min: usize::from(a.min),
            max: usize::from(a.max),
            levels,
            plain: Synchro::is_plain::<P>(duel),
        };
        let none = CardSet::default();
        let res = Synchro::level_reachable::<P>(duel, tg.union(ntg), &search)
            && tg.is_exists(
                duel,
                |tc, duel| Synchro::check_p41::<P>(duel, tc, tg, ntg, none, none, none, &search),
                1,
                None,
            );
        Synchro::reset_hand_synchro(duel, tp);
        res
    }

    // Not in the Lua: CheckP43 needs all of a selection to add up to the level and more materials only raise the sum, so
    // nothing is found from materials that cannot reach the level or from a selection already past it. The search is cut
    // short only when what it skips looks up no effect, since is_available numbers an effect that becomes available: no
    // requirements, and no effect of the codes CheckP41 to CheckP43 look up.
    fn is_plain<P: SynchroProcedure>(duel: &Duel) -> bool {
        let codes = [
            EFFECT_SYNCHRO_MAT_RESTRICTION,
            EFFECT_HAND_SYNCHRO + EFFECT_SYNCHRO_CHECK,
            EFFECT_SYNCHRO_MATERIAL_CUSTOM,
        ];
        P::REQ1.is_none() && P::REQ2.is_none() && P::REQM.is_none() && !codes.iter().any(|&code| duel.may_have_effect(code))
    }

    fn level_reachable<P: SynchroProcedure>(duel: &Duel, g: CardSet, s: &Search) -> bool {
        !s.plain || g.check_with_sum_equal(duel, |c, duel| s.levels.get(duel, c, s.sc), s.lv, P::MIN1 + P::MIN2, g.len())
    }

    fn level_exceeded(duel: &Duel, sg: CardSet, s: &Search) -> bool {
        let least = |level: u32| match level >> 16 {
            0 => level,
            high => high.min(level & 0xffff),
        };
        s.plain && sg.iter().map(|c| least(s.levels.get(duel, c, s.sc))).sum::<u32>() > s.lv
    }

    // The restrictions EFFECT_SYNCHRO_MAT_RESTRICTION puts on c and on the cards already chosen: false if one rules out
    // adding c to sg, else the cards the restrictions of c rule out are taken out of tg and ntg. Then the checks of hand
    // materials.
    fn restrictions(duel: &Duel, c: CardId, sg: CardSet, tg: &mut CardSet, ntg: &mut CardSet, mgchk: bool) -> bool {
        // Auxiliary.HarmonizingMagFilter(c, f, f:GetValue()).
        let rejects = |f: EffectId| move |c: CardId, duel: &Duel| matches!(f.get_value(duel), Value::Card(value) if value(duel, f, c) == 0);
        // Not in the Lua: the lookups are skipped while no effect has the code, as they would find nothing.
        if duel.may_have_effect(EFFECT_SYNCHRO_MAT_RESTRICTION) {
            let (mut trg, mut ntrg) = (CardSet::default(), CardSet::default());
            for f in c.get_card_effect(duel, EFFECT_SYNCHRO_MAT_RESTRICTION) {
                if sg.is_exists(duel, rejects(f), 1, Some(c.into())) {
                    return false;
                }
                trg.merge(tg.filter(duel, rejects(f), None));
                ntrg.merge(ntg.filter(duel, rejects(f), None));
            }
            for tc in sg {
                if tc
                    .get_card_effect(duel, EFFECT_SYNCHRO_MAT_RESTRICTION)
                    .iter()
                    .any(|&f| rejects(f)(c, duel))
                {
                    return false;
                }
            }
            (*tg, *ntg) = (tg.difference(trg), ntg.difference(ntrg));
        }
        let hand = |c: CardId, duel: &Duel| c.is_has_effect(duel, EFFECT_HAND_SYNCHRO + EFFECT_SYNCHRO_CHECK).is_some();
        unported!(duel, !mgchk && (hand(c, duel) || sg.is_exists(duel, hand, 1, None)));
        true
    }

    fn check_p41<P: SynchroProcedure>(
        duel: &Duel,
        c: CardId,
        tg: CardSet,
        ntg: CardSet,
        tsg: CardSet,
        ntsg: CardSet,
        sg: CardSet,
        s: &Search,
    ) -> bool {
        let (mut tg, mut ntg) = (tg, ntg);
        if !Synchro::restrictions(duel, c, sg, &mut tg, &mut ntg, s.mgchk) {
            return false;
        }
        let (tsg, sg) = (tsg.union(c.into()), sg.union(c.into()));
        let tsg_count = tsg.len();
        let p41 = |tc, duel: &Duel| Synchro::check_p41::<P>(duel, tc, tg, ntg, tsg, ntsg, sg, s);
        let p42 = |tc, duel: &Duel| Synchro::check_p42::<P>(duel, tc, ntg, tsg, ntsg, sg, s);
        if s.max != 0 && (tsg_count > s.max || s.max - tsg_count < P::MIN2) {
            false
        } else if s.max != 0 && s.max - tsg_count == P::MIN2 {
            req(P::REQ1, duel, tsg, s.sc, s.tp) && ntg.is_exists(duel, p42, 1, Some(sg))
        } else if tsg_count < P::MIN1 {
            tg.is_exists(duel, p41, 1, Some(sg))
        } else if tsg_count < P::MAX1 {
            tg.is_exists(duel, p41, 1, Some(sg)) || (req(P::REQ1, duel, tsg, s.sc, s.tp) && ntg.is_exists(duel, p42, 1, Some(sg)))
        } else {
            req(P::REQ1, duel, tsg, s.sc, s.tp) && ntg.is_exists(duel, p42, 1, Some(sg))
        }
    }

    fn check_p42<P: SynchroProcedure>(duel: &Duel, c: CardId, ntg: CardSet, tsg: CardSet, ntsg: CardSet, sg: CardSet, s: &Search) -> bool {
        let mut ntg = ntg;
        if !Synchro::restrictions(duel, c, sg, &mut CardSet::default(), &mut ntg, s.mgchk) {
            return false;
        }
        let (ntsg, sg) = (ntsg.union(c.into()), sg.union(c.into()));
        if Synchro::level_exceeded(duel, sg, s) {
            return false;
        }
        let (tsg_count, ntsg_count) = (tsg.len(), ntsg.len());
        let p42 = |tc, duel: &Duel| Synchro::check_p42::<P>(duel, tc, ntg, tsg, ntsg, sg, s);
        let complete = || {
            (s.min == 0 || tsg_count + ntsg_count >= s.min)
                && req(P::REQ2, duel, ntsg, s.sc, s.tp)
                && req(P::REQM, duel, sg, s.sc, s.tp)
                && sg.includes(s.pg)
                && Synchro::check_p43(duel, tsg, ntsg, sg, s)
        };
        if s.max != 0 && tsg_count + ntsg_count > s.max {
            false
        } else if ntsg_count < P::MIN2 {
            ntg.is_exists(duel, p42, 1, Some(sg))
        } else if ntsg_count < P::MAX2 {
            ntg.is_exists(duel, p42, 1, Some(sg)) || complete()
        } else {
            complete()
        }
    }

    fn check_p43(duel: &Duel, tsg: CardSet, ntsg: CardSet, sg: CardSet, s: &Search) -> bool {
        let (lv, sc, tp) = (s.lv, s.sc, s.tp);
        unported!(
            duel,
            sg.is_exists(
                duel,
                |c, duel| c.is_has_effect(duel, EFFECT_HAND_SYNCHRO + EFFECT_SYNCHRO_CHECK).is_some(),
                1,
                None
            )
        );
        let mut lvchk = false;
        let custom = if duel.may_have_effect(EFFECT_SYNCHRO_MATERIAL_CUSTOM) {
            sg
        } else {
            CardSet::default()
        };
        for tc in custom {
            for te in tc.get_card_effect(duel, EFFECT_SYNCHRO_MATERIAL_CUSTOM) {
                let Some(Operation::SynchroMaterial(op)) = te.get_operation(duel) else {
                    continue;
                };
                let (ok, tlvchk) = op(duel, te, tsg, ntsg, sg, lv, sc, tp);
                if !ok {
                    return false;
                }
                lvchk |= tlvchk;
            }
        }
        (lvchk || sg.check_with_sum_equal(duel, |c, duel| s.levels.get(duel, c, sc), lv, sg.len(), sg.len()))
            && ((sc.is_location(duel, LOCATION_EXTRA) && duel.get_location_count_from_ex(tp, tp, Some(sg), sc) > 0)
                || (!sc.is_location(duel, LOCATION_EXTRA) && duel.get_mzone_count(tp, Some(sg)) > 0))
    }

    fn target<P: SynchroProcedure>(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
        let (e, tp, smat, mg) = (a.e, a.tp, a.must, a.materials);
        let Some(c) = a.c else { return Ok(false) };
        let mut sg = CardSet::default();
        let (_, mut g, mut pg, mgchk, levels) = Synchro::materials(duel, c, tp, mg);
        if let Some(smat) = smat {
            pg.merge(smat);
            g.merge(smat);
        }
        let tg = g.filter(duel, |mc, duel| Synchro::tuner_filter::<P>(mc, duel, c, tp), None);
        let ntg = g.filter(duel, |mc, duel| Synchro::non_tuner_filter::<P>(mc, duel, c, tp), None);
        Synchro::unported_materials(duel, g, mgchk);
        let s = Search {
            lv: c.get_level(duel),
            sc: c,
            tp,
            pg,
            mgchk,
            min: usize::from(a.min),
            max: usize::from(a.max),
            levels,
            plain: Synchro::is_plain::<P>(duel),
        };
        let mut tsg = CardSet::default();
        let mut selectedastuner = CardSet::default();
        let mut ntsg = CardSet::default();
        let mut tune = true;
        while ntsg.len() < P::MAX2 {
            let p41 = |tc, duel: &Duel| Synchro::check_p41::<P>(duel, tc, tg, ntg, tsg, ntsg, sg, &s);
            let p42 = |tc, duel: &Duel| Synchro::check_p42::<P>(duel, tc, ntg, tsg, ntsg, sg, &s);
            if tune {
                let cancel = !mgchk && duel.is_summon_cancelable() && tsg.is_empty();
                let g3 = ntg.filter(duel, p42, Some(sg));
                let mut g2 = tg.filter(duel, p41, Some(sg));
                if !g3.is_empty() && tsg.len() >= P::MIN1 && req(P::REQ1, duel, tsg, c, tp) {
                    g2.merge(g3);
                }
                duel.hint(HINT_SELECTMSG, tp, HINTMSG_SMATERIAL);
                match g2.select_unselect(duel, Some(sg), tp, false, cancel, 1, 1)? {
                    None => {
                        if tsg.len() >= P::MIN1 && req(P::REQ1, duel, tsg, c, tp) && ntg.filter_count(duel, p42, Some(sg)) > 0 {
                            tune = false;
                        } else {
                            return Ok(false);
                        }
                    }
                    Some(tc) if !sg.is_contains(tc) => {
                        if g3.is_contains(tc) {
                            ntsg.insert(tc);
                            tune = false;
                        } else {
                            tsg.insert(tc);
                        }
                        selectedastuner.insert(tc);
                        sg.insert(tc);
                    }
                    Some(tc) => {
                        selectedastuner.remove(tc);
                        tsg.remove(tc);
                        sg.remove(tc);
                    }
                }
                let p41 = |tc, duel: &Duel| Synchro::check_p41::<P>(duel, tc, tg, ntg, tsg, ntsg, sg, &s);
                if tg.filter_count(duel, p41, Some(sg)) == 0 || tsg.len() >= P::MAX1 {
                    tune = false;
                }
            } else {
                let complete = |duel: &Duel| {
                    ntsg.len() >= P::MIN2
                        && req(P::REQ2, duel, ntsg, c, tp)
                        && req(P::REQM, duel, sg, c, tp)
                        && sg.includes(pg)
                        && Synchro::check_p43(duel, tsg, ntsg, sg, &s)
                };
                let finish = complete(duel);
                let cancel = !mgchk && duel.is_summon_cancelable() && sg.is_empty();
                let mut g2 = ntg.filter(duel, p42, Some(sg));
                if g2.is_empty() {
                    break;
                }
                let g3 = tg.filter(duel, p41, Some(sg));
                if !g3.is_empty() && ntsg.difference(selectedastuner).is_empty() && tsg.len() < P::MAX1 {
                    g2.merge(g3);
                }
                duel.hint(HINT_SELECTMSG, tp, HINTMSG_SMATERIAL);
                let Some(tc) = g2.select_unselect(duel, Some(sg), tp, finish, cancel, 1, 1)? else {
                    if complete(duel) {
                        break;
                    }
                    return Ok(false);
                };
                if !selectedastuner.is_contains(tc) {
                    if !sg.is_contains(tc) {
                        ntsg.insert(tc);
                        sg.insert(tc);
                    } else {
                        ntsg.remove(tc);
                        sg.remove(tc);
                    }
                } else if ntsg.difference(selectedastuner).is_empty() {
                    tune = true;
                    selectedastuner.remove(tc);
                    ntsg.remove(tc);
                    tsg.remove(tc);
                    sg.remove(tc);
                }
            }
        }
        Synchro::reset_hand_synchro(duel, tp);
        let subtsg = tsg.filter(
            duel,
            |sc, duel| {
                Synchro::sub1::<P>(sc, duel, c, SYNCHRO_MATERIAL, tp)
                    && (!P::f1(sc, duel, c, SYNCHRO_MATERIAL, tp) || !sc.is_type(duel, TYPE_TUNER))
            },
            None,
        );
        for subc in subtsg {
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_ADD_TYPE);
            e1.set_value(duel, TYPE_TUNER as i32);
            e1.set_reset(duel, RESET_EVENT + RESETS_STANDARD);
            subc.register_effect_full(duel, e1, true);
        }
        e.set_label_object_group(duel, Some(sg));
        Ok(true)
    }

    fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
        let (e, c) = (a.e, a.c);
        let Some(c) = c else { return Ok(()) };
        let g = e.get_label_object_group(duel).unwrap_or_default();
        c.set_material(duel, Some(g));
        let mat_check_eff = c.is_has_effect(duel, EFFECT_MATERIAL_CHECK);
        unported!(duel, mat_check_eff.is_some_and(|eff| eff.get_operation(duel).is_some()));
        // GetValue with arguments returns the function itself, so every hand material uses up its count.
        for mc in g {
            if let Some(handmatfilter) = mc.is_has_effect(duel, EFFECT_SYNCHRO_MAT_FROM_HAND) {
                handmatfilter.use_count_limit(duel, a.tp);
            }
        }
        unported!(duel, g.is_exists(duel, |mc, duel| mc.get_flag_effect(duel, 3096468) != 0, 1, None));
        duel.send_to_grave(g, REASON_MATERIAL + REASON_SYNCHRO)?;
        Ok(())
    }
}
