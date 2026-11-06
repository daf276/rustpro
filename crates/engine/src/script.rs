// What Lua does for the core: the card scripts' entry points, their parameters, and coroutines.
//
// The core runs cost, target and operation functions as coroutines: a Duel action emplaces its processor unit and
// yields, the processor runs that unit and everything it emplaced, then resumes the function. Here a function runs
// to completion in one call and an action runs its units in a nested processor loop at the call site, which gives the
// same order of state changes and messages. A prompt inside that loop cannot wait for an answer mid-function, so the
// function bails, the prompt is presented on the state the function reached, and once answered the function reruns
// from a snapshot taken before it started, reading the answers so far from a choice log; the messages its earlier
// attempts already wrote are muted.
use crate::buffer::ProgressiveBuffer;
use crate::card::{CardId, CardSet};
use crate::duel::Duel;
use crate::effect::{EffectId, Event};
use crate::list::List;
use crate::processor::{OcgStatus, Unit};

// A Duel action needed a choice the log does not hold, or the duel ended while the function ran.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Interrupt;

pub type Script<T> = Result<T, Interrupt>;

// The parameters (e, tp, eg, ep, ev, re, r, rp) the core passes to condition, cost, target and operation functions;
// `c` is the card a summon procedure's functions receive besides (the condition of EFFECT_SPSUMMON_PROC_G receives
// only e and c), followed by the materials it must use, the only ones it may, and a forced minimum and maximum (0 for
// nil, as the core leaves them out then). The condition (e, g, gc, chkf) and operation (e, tp, eg, ..., gc, chkf) of
// EFFECT_FUSION_MATERIAL get g as `materials` and gc as `must`; the summoning effect passed after them is left out, as
// only cards that are not ported read it.
#[derive(Debug, Clone, Copy, Default)]
pub struct Args {
    pub e: EffectId,
    pub tp: u8,
    pub eg: Option<CardSet>,
    pub ep: u8,
    pub ev: u32,
    pub re: Option<EffectId>,
    pub r: u32,
    pub rp: u8,
    pub c: Option<CardId>,
    pub must: Option<CardSet>,
    pub materials: Option<CardSet>,
    pub min: u16,
    pub max: u16,
    pub chkf: u64,
}

impl Args {
    // A field effect's condition only receives the effect.
    pub(crate) fn effect(e: EffectId) -> Self {
        Self { e, ..Self::default() }
    }

    pub(crate) fn event(e: EffectId, tp: u8, event: &Event) -> Self {
        Self {
            e,
            tp,
            eg: event.event_cards,
            ep: event.event_player,
            ev: event.event_value,
            re: event.reason_effect,
            r: event.reason,
            rp: event.reason_player,
            ..Self::default()
        }
    }
}

// The parameters after the card that the core passes a summon procedure's target and operation.
#[derive(Debug, Clone, Copy, Default)]
pub struct Procedure {
    pub card: CardId,
    pub must: Option<CardSet>,
    pub materials: Option<CardSet>,
    pub min: u16,
    pub max: u16,
    pub chkf: u64,
}

pub type InitialEffect = fn(&mut Duel, CardId);

// A Fusion material check (tp, sg, fc) built by a closure over a card, kept with that card.
pub type FusionCheck = (fn(&Duel, CardId, u8, CardSet, CardId) -> bool, CardId);

// The scripts the oracle loads: utility.lua and the libraries it pulls in (their global effects), and c<code>.lua.
#[derive(Debug, Clone, Copy)]
pub struct Library {
    pub load: fn(&mut Duel),
    pub card_script: fn(u32) -> Option<InitialEffect>,
}

pub type Response = List<u8, 32>;

// The choice log of the outermost running function and what its attempts carry over.
#[derive(Debug, Clone, Default)]
pub struct Replay {
    pub log: Vec<Response>,
    pub cursor: usize,
    // Messages the last attempt wrote, which the next one mutes.
    pub messages: u64,
    // The duel before the pending function's first attempt; set while one of its prompts waits.
    pub snapshot: Option<Box<Duel>>,
    pub spare: Spare,
    // The prompt a nested loop met with no answer logged, carried out to the function's unit.
    pub prompt: Option<Box<Unit>>,
    // The unit stack of a yield's nested run, kept for its capacity.
    pub units: Vec<Unit>,
    pub interrupted: bool,
    // The effects and players of the running functions, innermost last, which an action restores as the reason after
    // its units ran (the core sets them again on every resume).
    pub stack: List<(EffectId, u8), 4>,
}

// The last finished function's snapshot, whose buffers and arenas the next snapshot reuses; a clone of the duel starts
// without one.
#[derive(Debug, Default)]
pub struct Spare(Option<Box<Duel>>);

impl Clone for Spare {
    fn clone(&self) -> Self {
        Self(None)
    }
}

impl Duel {
    // A cost or target with chk=0: a check, which cannot reach a prompt.
    pub(crate) fn check_script(&mut self, call: impl FnOnce(&mut Self) -> Script<bool>) -> bool {
        call(self).unwrap_or(false)
    }

    // Runs a cost, target or operation function for `effect` as `player`, the body of the Execute units: a nested
    // call runs in its parent's attempt, an outermost one snapshots the duel first unless it is `promptless`. Returns
    // None if a prompt is now presented or the duel ended.
    pub(crate) fn run_script<T>(
        &mut self,
        effect: EffectId,
        player: u8,
        promptless: bool,
        call: impl Fn(&mut Self) -> Script<T>,
    ) -> Option<T> {
        let nested = !self.core.replay.stack.is_empty();
        if nested || promptless {
            self.core.replay.stack.push((effect, player));
            let result = call(self);
            self.core.replay.stack.pop();
            if result.is_err() && nested {
                self.core.replay.interrupted = true;
            } else if result.is_err() && self.result.is_none() {
                self.reach_unported(concat!(file!(), ":", line!(), " (a promptless function prompted)"));
            }
            return result.ok();
        }
        let snapshot = match self.core.replay.snapshot.take() {
            Some(mut snapshot) => {
                let answer = self.returns.data.clone();
                self.core.replay.log.push(answer);
                self.copy_from(&mut snapshot);
                snapshot
            }
            None => self.snapshot(),
        };
        self.core.replay.cursor = 0;
        self.trace.mute = self.core.replay.messages;
        let start = self.trace.count;
        self.core.replay.stack.push((effect, player));
        let result = call(self);
        self.core.replay.stack.pop();
        self.core.replay.interrupted = false;
        match result {
            Ok(value) => {
                debug_assert_eq!(self.trace.mute, 0, "a rerun wrote fewer messages than its previous attempt");
                self.core.replay.log.clear();
                self.core.replay.messages = 0;
                self.core.replay.spare = Spare(Some(snapshot));
                Some(value)
            }
            Err(Interrupt) => {
                if self.result.is_none() {
                    self.core.replay.messages = self.trace.count - start;
                    self.core.replay.snapshot = Some(snapshot);
                }
                None
            }
        }
    }

    // A copy without the trace and the replay state, which live on across attempts.
    fn snapshot(&mut self) -> Box<Duel> {
        if let Some(mut spare) = self.core.replay.spare.0.take() {
            spare.copy_from(self);
            return spare;
        }
        let trace = std::mem::take(&mut self.trace);
        let replay = std::mem::take(&mut self.core.replay);
        let snapshot = Box::new(self.clone());
        self.trace = trace;
        self.core.replay = replay;
        snapshot
    }

    // Makes this duel equal to `source` apart from the trace and the replay state, reusing its buffers. One must be a
    // snapshot of the other, since the arenas copy only what changed since either synced.
    fn copy_from(&mut self, source: &mut Duel) {
        let Duel {
            cards,
            player,
            infos,
            effects,
            field_effects,
            relations,
            core,
            returns,
            return_cards,
            nil_event,
            random,
            trace: _,
            result,
            library,
            script_state,
            script_groups,
            fusion_check_additional,
            material_cards,
            overlay_cards,
            counters,
            card_targets,
            card_relations,
            unported,
        } = source;
        self.cards.copy_from(cards);
        self.player.clone_from(player);
        self.infos.clone_from(infos);
        self.effects.sync_from(effects);
        self.field_effects.clone_from(field_effects);
        self.relations.clone_from(relations);
        let (replay, own) = (std::mem::take(&mut core.replay), std::mem::take(&mut self.core.replay));
        self.core.clone_from(core);
        (core.replay, self.core.replay) = (replay, own);
        self.returns.clone_from(returns);
        self.return_cards.clone_from(return_cards);
        self.nil_event.clone_from(nil_event);
        self.random.clone_from(random);
        self.result.clone_from(result);
        self.library.clone_from(library);
        self.script_state.clone_from(script_state);
        self.script_groups.clone_from(script_groups);
        self.fusion_check_additional = *fusion_check_additional;
        self.material_cards.clone_from(material_cards);
        self.overlay_cards.clone_from(overlay_cards);
        self.counters.clone_from(counters);
        self.card_targets.clone_from(card_targets);
        self.card_relations.clone_from(card_relations);
        self.unported.clone_from(unported);
    }

    // The resume of a yield: what the processor does between a coroutine's yield and its resume. The units the
    // function emplaced run to completion; a prompt among them takes its answer from the log or interrupts.
    pub(crate) fn yield_script(&mut self) -> Script<()> {
        self.returns.set_i32(0, 0);
        let mut units = std::mem::take(&mut self.core.replay.units);
        let result = self.run_yield(&mut units);
        units.clear();
        self.core.replay.units = units;
        result?;
        if let Some(&(effect, player)) = self.core.replay.stack.last() {
            self.core.reason_effect.set(Some(effect));
            self.core.reason_player.set(player);
        }
        Ok(())
    }

    fn run_yield(&mut self, units: &mut Vec<Unit>) -> Script<()> {
        loop {
            if self.result.is_some() {
                return Err(Interrupt);
            }
            match self.process_unit(units) {
                OcgStatus::End => return Ok(()),
                OcgStatus::Continue => {}
                OcgStatus::Awaiting => {
                    let replay = &mut self.core.replay;
                    match replay.log.get(replay.cursor) {
                        Some(answer) => {
                            self.returns = ProgressiveBuffer { data: answer.clone() };
                            self.core.replay.cursor += 1;
                        }
                        None => {
                            self.core.replay.prompt = units.pop().map(Box::new);
                            self.core.replay.interrupted = true;
                        }
                    }
                }
            }
            if self.core.replay.interrupted {
                return Err(Interrupt);
            }
        }
    }
}
