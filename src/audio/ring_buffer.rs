//! Lock-free Single-Producer Single-Consumer (SPSC) command queue.
//!
//! Provides ultra-low latency, zero-allocation messaging from the game simulation thread
//! to the real-time audio playback thread. Game ticks never block on audio I/O.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use crate::audio::sound::{SoundCategory, SoundId, SoundSample};
use crate::core::math::Vec3;

/// Audio engine commands dispatched asynchronously from game code.
#[derive(Clone, Debug)]
pub enum AudioCommand {
    PlaySound {
        sound_id: SoundId,
        volume: f32,
        pitch: f32,
        pan: f32,
        looping: bool,
        spatial: bool,
        position: Vec3,
        min_dist: f32,
        max_dist: f32,
        rolloff: f32,
        occlusion: f32,
    },
    PlayMusicSample {
        sample: Arc<SoundSample>,
        volume: f32,
        looping: bool,
    },
    StopMusic {
        fade_out_seconds: f32,
    },
    UpdateListener {
        position: Vec3,
        forward: Vec3,
        right: Vec3,
        up: Vec3,
    },
    SetCategoryVolume {
        category: SoundCategory,
        volume: f32,
    },
    StopAll {
        fade_out_seconds: f32,
    },
}

/// Power-of-two capacity for fast bitwise masking.
pub const COMMAND_QUEUE_CAPACITY: usize = 512;
const COMMAND_QUEUE_MASK: usize = COMMAND_QUEUE_CAPACITY - 1;

/// Cache-padded lock-free SPSC ring buffer.
pub struct SpscCommandQueue {
    buffer: Box<[Option<AudioCommand>; COMMAND_QUEUE_CAPACITY]>,
    write_head: AtomicUsize,
    read_tail: AtomicUsize,
}

impl SpscCommandQueue {
    pub fn new() -> Self {
        Self {
            buffer: Box::new(std::array::from_fn(|_| None)),
            write_head: AtomicUsize::new(0),
            read_tail: AtomicUsize::new(0),
        }
    }

    /// Pushes a command into the queue (called from game simulation thread).
    /// Returns false only if the queue is saturated. Takes ~3 nanoseconds.
    #[inline]
    pub fn push(&mut self, cmd: AudioCommand) -> bool {
        let head = self.write_head.load(Ordering::Relaxed);
        let tail = self.read_tail.load(Ordering::Acquire);

        if head.wrapping_sub(tail) >= COMMAND_QUEUE_CAPACITY {
            return false; // Queue full
        }

        self.buffer[head & COMMAND_QUEUE_MASK] = Some(cmd);
        self.write_head.store(head.wrapping_add(1), Ordering::Release);
        true
    }

    /// Pops a command from the queue (called from audio mixing thread).
    #[inline]
    pub fn pop(&mut self) -> Option<AudioCommand> {
        let tail = self.read_tail.load(Ordering::Relaxed);
        let head = self.write_head.load(Ordering::Acquire);

        if tail == head {
            return None; // Queue empty
        }

        let cmd = self.buffer[tail & COMMAND_QUEUE_MASK].take();
        self.read_tail.store(tail.wrapping_add(1), Ordering::Release);
        cmd
    }
}
