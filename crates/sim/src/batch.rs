use engine::{Action, ActionError, Duel, PositionError, Prompt};
use fiber::Fiber;
use rayon::prelude::*;

use crate::encoding::{ACTIONS, EncodingError, find, write_mask};
use crate::features::{Vocabulary, features};
use crate::tensor::Tensors;
use crate::tracker::Tracker;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AutoAdvance {
    #[default]
    Off,
    // Only the prompts of the player who acted last: stops at the other player's first prompt.
    Own,
    Both,
}

// Applies the only legal action while there is one, returning how many; `legal` is left with the last prompt's actions.
pub fn auto_advance(duel: &mut Fiber, mode: AutoAdvance, player: u8, legal: &mut Vec<Action>) -> Result<u64, ActionError> {
    let mut applied = 0;
    loop {
        duel.policy_actions_into(legal);
        let allowed = match (mode, duel.prompt()) {
            (AutoAdvance::Off, _) | (_, None) => false,
            (AutoAdvance::Own, Some(prompt)) => prompt.player() == player,
            (AutoAdvance::Both, Some(_)) => true,
        };
        match legal.as_slice() {
            &[only] if allowed => duel.apply(only)?,
            _ => return Ok(applied),
        };
        applied += 1;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Options {
    pub auto_advance: AutoAdvance,
    // A duel cut off after this many policy actions ends without a result.
    pub max_steps: u64,
    // A [Tracker] per player in each slot, for [Batch::features]; costs the duel's message trace.
    pub track: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            auto_advance: AutoAdvance::Off,
            max_steps: u64::MAX,
            track: false,
        }
    }
}

// `episode` numbers the duels of a slot, `steps` counts the policy's actions and `skipped` those auto-advance applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Outcome {
    pub slot: usize,
    pub episode: u64,
    pub result: Option<(u8, u8)>,
    pub steps: u64,
    pub skipped: u64,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum BatchError {
    #[error("{actions} actions for {slots} slots")]
    Count { actions: usize, slots: usize },
    #[error("slot {slot}: index {index} is not a legal action")]
    Illegal { slot: usize, index: usize },
    #[error("slot {slot}: {error}")]
    Action { slot: usize, error: ActionError },
    #[error("slot {slot}: {error}")]
    Encoding { slot: usize, error: EncodingError },
    #[error("slot {slot}: {error}")]
    Setup { slot: usize, error: PositionError },
    #[error("buffers for {rows} observations for {slots} slots")]
    Rows { rows: usize, slots: usize },
    #[error("the batch does not track its duels")]
    Untracked,
}

// No duel once the generator has retired the slot.
struct Slot {
    duel: Option<Fiber>,
    trackers: Option<[Tracker; 2]>,
    legal: Vec<Action>,
    episode: u64,
    steps: u64,
    skipped: u64,
}

// Duels stepped in lockstep on rayon's pool; `generator(slot, episode)` gives unprocessed duels, or None to retire the slot.
pub struct Batch<G> {
    slots: Vec<Slot>,
    masks: Vec<bool>,
    ended: Vec<Option<Outcome>>,
    generator: G,
    options: Options,
}

type Generated = Result<Option<Duel>, PositionError>;

impl<G: Fn(usize, u64) -> Generated + Sync> Batch<G> {
    pub fn new(size: usize, options: Options, generator: G) -> Result<Self, BatchError> {
        let slot = |_| Slot {
            duel: None,
            trackers: None,
            legal: Vec::new(),
            episode: 0,
            steps: 0,
            skipped: 0,
        };
        let mut batch = Self {
            slots: (0..size).map(slot).collect(),
            masks: vec![false; size * ACTIONS],
            ended: vec![None; size],
            generator,
            options,
        };
        let Self {
            slots, masks, generator, ..
        } = &mut batch;
        let starts = slots.par_iter_mut().zip(masks.par_chunks_exact_mut(ACTIONS)).enumerate();
        starts.try_for_each(|(index, (slot, mask))| slot.start(index, &*generator, options, mask))?;
        Ok(batch)
    }

    // Retired slots ignore their action; a slot whose duel ended holds its outcome in the returned slice and its next duel.
    pub fn step(&mut self, actions: &[usize]) -> Result<&[Option<Outcome>], BatchError> {
        let Self {
            slots,
            masks,
            ended,
            generator,
            options,
        } = self;
        let (generator, options) = (&*generator, *options);
        if actions.len() != slots.len() {
            return Err(BatchError::Count {
                actions: actions.len(),
                slots: slots.len(),
            });
        }
        let steps = slots
            .par_iter_mut()
            .zip(masks.par_chunks_exact_mut(ACTIONS))
            .zip(ended.par_iter_mut());
        steps
            .zip(actions)
            .enumerate()
            .try_for_each(|(index, (((slot, mask), ended), &action))| {
                *ended = slot.step(index, action, options)?;
                if ended.is_some() {
                    slot.start(index, generator, options, mask)
                } else {
                    write_mask(&slot.legal, mask).map_err(|error| BatchError::Encoding { slot: index, error })
                }
            })?;
        Ok(&self.ended)
    }

    // Steps until every slot is retired; the outcomes come in the order the duels ended.
    pub fn run(&mut self, mut policy: impl FnMut(&Self, &mut [usize])) -> Result<Vec<Outcome>, BatchError> {
        let (mut actions, mut outcomes) = (vec![0; self.len()], Vec::new());
        while self.slots.iter().any(|slot| slot.duel.is_some()) {
            policy(self, &mut actions);
            outcomes.extend(self.step(&actions)?.iter().flatten());
        }
        Ok(outcomes)
    }
}

impl<G> Batch<G> {
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    // ACTIONS entries per slot.
    pub fn masks(&self) -> &[bool] {
        &self.masks
    }

    pub fn duel(&self, slot: usize) -> Option<&Duel> {
        self.slots.get(slot)?.duel.as_deref()
    }

    pub fn legal(&self, slot: usize) -> &[Action] {
        self.slots.get(slot).map_or(&[], |slot| &slot.legal)
    }

    // Each slot's observation for the player its prompt waits for, empty rows for a retired slot.
    pub fn observe(&mut self, tensors: &mut Tensors) -> Result<(), BatchError> {
        if tensors.len() != self.len() {
            return Err(BatchError::Rows {
                rows: tensors.len(),
                slots: self.len(),
            });
        }
        tensors
            .par_write(self.slots.par_iter_mut().map(|slot| slot.duel.as_mut()))
            .map_err(|(slot, error)| BatchError::Encoding { slot, error })
    }

    pub fn tracker(&self, slot: usize, player: u8) -> Option<&Tracker> {
        Some(of(self.slots.get(slot)?.trackers.as_ref()?, player))
    }

    // Appends each slot's [features] for the player its prompt waits for and pushes where they start; empty for a retired slot.
    pub fn features(&mut self, vocabulary: &Vocabulary, out: &mut Vec<u32>, starts: &mut Vec<usize>) -> Result<(), BatchError> {
        if !self.options.track {
            return Err(BatchError::Untracked);
        }
        let bags: Vec<Vec<u32>> = (self.slots.par_iter_mut())
            .map(|slot| {
                let mut bag = Vec::new();
                if let (Some(duel), Some(trackers)) = (&slot.duel, &slot.trackers)
                    && let Some(prompt) = duel.prompt()
                {
                    features(duel, of(trackers, prompt.player()), vocabulary, &mut bag);
                }
                bag
            })
            .collect();
        for bag in bags {
            starts.push(out.len());
            out.extend(bag);
        }
        Ok(())
    }
}

fn track(duel: &mut Fiber, trackers: &mut Option<[Tracker; 2]>) {
    if let Some(trackers) = trackers {
        let messages = duel.take_trace();
        trackers.iter_mut().for_each(|tracker| tracker.update(&messages, duel));
    }
}

fn of(trackers: &[Tracker; 2], player: u8) -> &Tracker {
    let [first, second] = trackers;
    if player == 0 { first } else { second }
}

impl Slot {
    fn start(
        &mut self,
        index: usize,
        generator: &impl Fn(usize, u64) -> Generated,
        options: Options,
        mask: &mut [bool],
    ) -> Result<(), BatchError> {
        loop {
            self.legal.clear();
            let generated = generator(index, self.episode).map_err(|error| BatchError::Setup { slot: index, error })?;
            self.trackers = None;
            self.duel = generated.map(|mut duel| {
                if options.track {
                    duel.set_trace(true);
                    self.trackers = Some([0, 1].map(|player| Tracker::new(&duel, player)));
                }
                Fiber::new(duel)
            });
            let Some(duel) = &mut self.duel else { break };
            duel.process();
            let player = duel.prompt().map_or(0, Prompt::player);
            self.skipped = auto_advance(duel, options.auto_advance, player, &mut self.legal)
                .map_err(|error| BatchError::Action { slot: index, error })?;
            track(duel, &mut self.trackers);
            self.steps = 0;
            if !self.legal.is_empty() {
                break;
            }
            self.episode += 1;
        }
        write_mask(&self.legal, mask).map_err(|error| BatchError::Encoding { slot: index, error })
    }

    fn step(&mut self, slot: usize, index: usize, options: Options) -> Result<Option<Outcome>, BatchError> {
        let Some(duel) = &mut self.duel else { return Ok(None) };
        let action = find(&self.legal, index).ok_or(BatchError::Illegal { slot, index })?;
        let player = duel.prompt().map_or(0, Prompt::player);
        let advanced = duel
            .apply(action)
            .and_then(|_| auto_advance(duel, options.auto_advance, player, &mut self.legal));
        self.skipped += advanced.map_err(|error| BatchError::Action { slot, error })?;
        track(duel, &mut self.trackers);
        self.steps += 1;
        if !self.legal.is_empty() && self.steps < options.max_steps {
            return Ok(None);
        }
        let outcome = Outcome {
            slot,
            episode: self.episode,
            result: duel.result(),
            steps: self.steps,
            skipped: self.skipped,
        };
        self.episode += 1;
        Ok(Some(outcome))
    }
}
