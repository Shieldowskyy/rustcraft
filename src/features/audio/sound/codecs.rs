//! Dekodowanie formatów audio WAV (8/16/24/32-bit PCM/Float), Ogg Opus oraz Ogg Vorbis do `SoundSample`.
//! Całkowicie wolne od `#[cfg]` — limity pamięci PS Vita obsługiwane przez `platform::capabilities()`.

use crate::platform::{capabilities, system::vita_debug_log};

use super::types::SoundSample;

const MAX_VITA_SAMPLES: usize = 6 * 1024 * 1024;

impl SoundSample {
    pub fn from_wav_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 44 {
            return Err("WAV header too short".to_string());
        }

        if &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
            return Err("Invalid RIFF/WAVE magic header".to_string());
        }

        let mut offset = 12;
        let mut audio_format = 1u16;
        let mut channels = 1u16;
        let mut sample_rate = 44100u32;
        let mut bits_per_sample = 16u16;
        let mut data_start = 0usize;
        let mut data_len = 0usize;

        while offset + 8 <= bytes.len() {
            let chunk_id = &bytes[offset..offset + 4];
            let chunk_size = u32::from_le_bytes(
                bytes[offset + 4..offset + 8]
                    .try_into()
                    .map_err(|_| "Malformed chunk size".to_string())?,
            ) as usize;
            offset += 8;

            if chunk_id == b"fmt " {
                if chunk_size < 16 || offset + chunk_size > bytes.len() {
                    return Err("Malformed fmt chunk".to_string());
                }
                audio_format = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]);
                channels = u16::from_le_bytes([bytes[offset + 2], bytes[offset + 3]]);
                sample_rate = u32::from_le_bytes([
                    bytes[offset + 4],
                    bytes[offset + 5],
                    bytes[offset + 6],
                    bytes[offset + 7],
                ]);
                bits_per_sample = u16::from_le_bytes([bytes[offset + 14], bytes[offset + 15]]);
            } else if chunk_id == b"data" {
                data_start = offset;
                data_len = chunk_size.min(bytes.len() - offset);
                break;
            }

            offset += (chunk_size + 1) & !1;
        }

        if data_start == 0 || data_len == 0 {
            return Err("Missing WAV 'data' chunk".to_string());
        }

        let pcm_slice = &bytes[data_start..data_start + data_len];
        let mut samples = Vec::new();

        match (audio_format, bits_per_sample) {
            (1, 8) => {
                samples.reserve(pcm_slice.len());
                for &b in pcm_slice {
                    samples.push((b as f32 - 128.0) / 128.0);
                }
            }
            (1, 16) => {
                let count = pcm_slice.len() / 2;
                samples.reserve(count);
                for i in 0..count {
                    let val = i16::from_le_bytes([pcm_slice[i * 2], pcm_slice[i * 2 + 1]]);
                    samples.push(val as f32 / 32768.0);
                }
            }
            (1, 24) => {
                let count = pcm_slice.len() / 3;
                samples.reserve(count);
                for i in 0..count {
                    let b0 = pcm_slice[i * 3];
                    let b1 = pcm_slice[i * 3 + 1];
                    let b2 = pcm_slice[i * 3 + 2];
                    let val = ((b2 as i8 as i32) << 16) | ((b1 as i32) << 8) | (b0 as i32);
                    samples.push(val as f32 / 8388608.0);
                }
            }
            (1, 32) => {
                let count = pcm_slice.len() / 4;
                samples.reserve(count);
                for i in 0..count {
                    let val = i32::from_le_bytes([
                        pcm_slice[i * 4],
                        pcm_slice[i * 4 + 1],
                        pcm_slice[i * 4 + 2],
                        pcm_slice[i * 4 + 3],
                    ]);
                    samples.push(val as f32 / 2147483648.0);
                }
            }
            (3, 32) => {
                let count = pcm_slice.len() / 4;
                samples.reserve(count);
                for i in 0..count {
                    let val = f32::from_le_bytes([
                        pcm_slice[i * 4],
                        pcm_slice[i * 4 + 1],
                        pcm_slice[i * 4 + 2],
                        pcm_slice[i * 4 + 3],
                    ]);
                    samples.push(val.clamp(-1.0, 1.0));
                }
            }
            _ => {
                return Err(format!(
                    "Unsupported WAV format: format={}, bits={}",
                    audio_format, bits_per_sample
                ));
            }
        }

        Ok(Self::new(samples, sample_rate, channels))
    }

    pub fn from_ogg_opus_bytes(bytes: &[u8]) -> Result<Self, String> {
        let cursor = std::io::Cursor::new(bytes);
        let mut reader = opus_pure::OggOpusReader::new(cursor)
            .map_err(|e| format!("Ogg Opus header error: {:?}", e))?;

        let src_channels = reader.head().channel_count as usize;
        if src_channels == 0 {
            return Err("Invalid Opus channel count: 0".to_string());
        }

        let is_vita = capabilities().is_vita;
        // DLACZEGO: Na PS Vita dekodujemy do 24 kHz mono i nakładamy limit 6M próbek (24 MB),
        // aby nie przekroczyć sterty Newlib (180 MB) przy długich utworach.
        let (decode_rate, decimate_2x, out_channels) = if is_vita {
            match reader.head().decoder(24000) {
                Ok(_) => (24000i32, false, 1usize),
                Err(_) => (48000i32, true, 1usize),
            }
        } else {
            (48000i32, false, src_channels)
        };

        let mut decoder = reader
            .head()
            .decoder(decode_rate)
            .map_err(|e| format!("Opus decoder init error: {:?}", e))?;

        let out_sample_rate = if decimate_2x {
            (decode_rate as u32) / 2
        } else {
            decode_rate as u32
        };

        let mut samples = Vec::new();
        let max_samples = opus_pure::MAX_PACKET_SAMPLES;
        let mut pcm_buf = vec![0.0f32; max_samples * src_channels];

        while let Some(packet) = reader
            .read_packet()
            .map_err(|e| format!("Ogg Opus packet read error: {:?}", e))?
        {
            let samples_per_channel = decoder
                .decode(&packet.data, max_samples, &mut pcm_buf)
                .map_err(|e| format!("Opus packet decode error: {:?}", e))?;

            if is_vita {
                let step = if decimate_2x { 2 } else { 1 };
                let produced = samples_per_channel / step;
                if samples.len() + produced > MAX_VITA_SAMPLES {
                    break;
                }
                if samples.try_reserve(produced).is_err() {
                    vita_debug_log(&format!(
                        "Opus decode stopped early due to low heap (kept {} samples)",
                        samples.len()
                    ));
                    break;
                }
                let mut frame = 0;
                while frame < samples_per_channel {
                    let base = frame * src_channels;
                    let mono = if src_channels >= 2 {
                        (pcm_buf[base] + pcm_buf[base + 1]) * 0.5
                    } else {
                        pcm_buf[base]
                    };
                    samples.push(mono);
                    frame += step;
                }
            } else {
                let total_samples = samples_per_channel * src_channels;
                samples
                    .try_reserve(total_samples)
                    .map_err(|_| "Out of memory while decoding Ogg Opus".to_string())?;
                samples.extend_from_slice(&pcm_buf[..total_samples]);
            }
        }

        if samples.is_empty() {
            return Err("Empty Ogg Opus audio stream".to_string());
        }

        let rate_div = if out_sample_rate < 48000 {
            (48000 / out_sample_rate) as usize
        } else {
            1
        };
        let pre_skip = ((reader.head().pre_skip as usize / rate_div) * out_channels).min(samples.len());
        if pre_skip > 0 {
            samples.drain(0..pre_skip);
        }

        Ok(Self::new(samples, out_sample_rate, out_channels as u16))
    }

    pub fn from_ogg_vorbis_bytes(bytes: &[u8]) -> Result<Self, String> {
        let cursor = std::io::Cursor::new(bytes);
        let mut reader = lewton::inside_ogg::OggStreamReader::new(cursor)
            .map_err(|e| format!("OGG Vorbis header decode error: {:?}", e))?;

        let src_sample_rate = reader.ident_hdr.audio_sample_rate;
        let src_channels = reader.ident_hdr.audio_channels as usize;
        if src_channels == 0 {
            return Err("Invalid OGG Vorbis channel count: 0".to_string());
        }

        let is_vita = capabilities().is_vita;
        let (decimate_2x, out_channels) = if is_vita {
            (src_sample_rate > 24000, 1usize)
        } else {
            (false, src_channels)
        };

        let out_sample_rate = if decimate_2x {
            src_sample_rate / 2
        } else {
            src_sample_rate
        };

        let mut samples = Vec::new();
        while let Some(packet) = reader
            .read_dec_packet_itl()
            .map_err(|e| format!("OGG Vorbis packet decode error: {:?}", e))?
        {
            if is_vita {
                let frames = packet.len() / src_channels;
                let step = if decimate_2x { 2 } else { 1 };
                let produced = frames / step;
                if samples.len() + produced > MAX_VITA_SAMPLES {
                    break;
                }
                if samples.try_reserve(produced).is_err() {
                    vita_debug_log(&format!(
                        "Vorbis decode stopped early due to low heap (kept {} samples)",
                        samples.len()
                    ));
                    break;
                }
                let mut frame = 0;
                while frame < frames {
                    let base = frame * src_channels;
                    let mono = if src_channels >= 2 {
                        (packet[base] as f32 + packet[base + 1] as f32) * (0.5 / 32768.0)
                    } else {
                        packet[base] as f32 / 32768.0
                    };
                    samples.push(mono);
                    frame += step;
                }
            } else {
                samples
                    .try_reserve(packet.len())
                    .map_err(|_| "Out of memory while decoding OGG Vorbis".to_string())?;
                for s in packet {
                    samples.push(s as f32 / 32768.0);
                }
            }
        }

        if samples.is_empty() {
            return Err("Empty OGG Vorbis audio stream".to_string());
        }

        Ok(Self::new(samples, out_sample_rate, out_channels as u16))
    }

    pub fn from_ogg_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.windows(8).take(128).any(|w| w == b"OpusHead") {
            Self::from_ogg_opus_bytes(bytes)
        } else {
            Self::from_ogg_vorbis_bytes(bytes).or_else(|_| Self::from_ogg_opus_bytes(bytes))
        }
    }

    pub fn from_file_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() >= 4 && &bytes[0..4] == b"OggS" {
            Self::from_ogg_bytes(bytes)
        } else if bytes.len() >= 4 && &bytes[0..4] == b"RIFF" {
            Self::from_wav_bytes(bytes)
        } else {
            Self::from_ogg_bytes(bytes).or_else(|_| Self::from_wav_bytes(bytes))
        }
    }
}
