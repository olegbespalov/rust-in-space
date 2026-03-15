#!/usr/bin/env python3

import math
import random
import wave
from pathlib import Path

SAMPLE_RATE = 44100
ROOT = Path(__file__).resolve().parents[2]
SFX_DIR = ROOT / "assets" / "audio" / "sfx"
MUSIC_DIR = ROOT / "assets" / "audio" / "music"
SOURCE_DIR = ROOT / "assets" / "audio" / "source"


def clamp(value: float, lo: float = -1.0, hi: float = 1.0) -> float:
    return max(lo, min(hi, value))


def envelope(t: float, attack: float, decay: float, sustain: float, release: float, total: float) -> float:
    if t < attack:
        return t / attack if attack > 0 else 1.0
    if t < attack + decay:
        if decay <= 0:
            return sustain
        x = (t - attack) / decay
        return 1.0 + (sustain - 1.0) * x
    if t < total - release:
        return sustain
    if release <= 0:
        return 0.0
    x = (t - (total - release)) / release
    return sustain * max(0.0, 1.0 - x)


def sine(freq: float, t: float) -> float:
    return math.sin(2.0 * math.pi * freq * t)


def square(freq: float, t: float) -> float:
    return 1.0 if sine(freq, t) >= 0 else -1.0


def triangle(freq: float, t: float) -> float:
    return (2.0 / math.pi) * math.asin(sine(freq, t))


def noise() -> float:
    return random.uniform(-1.0, 1.0)


def lowpass(samples: list[float], alpha: float) -> list[float]:
    if not samples:
        return []
    out = [samples[0]]
    for sample in samples[1:]:
        out.append(out[-1] + alpha * (sample - out[-1]))
    return out


def normalize(samples: list[float], peak: float = 0.92) -> list[float]:
    max_val = max((abs(s) for s in samples), default=1.0)
    if max_val == 0:
        return samples
    gain = peak / max_val
    return [clamp(s * gain) for s in samples]


def mix(layers: list[list[float]]) -> list[float]:
    length = max((len(layer) for layer in layers), default=0)
    out = [0.0] * length
    for layer in layers:
        for i, sample in enumerate(layer):
            out[i] += sample
    return out


def write_wav(path: Path, samples: list[float]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    pcm = bytearray()
    for sample in normalize(samples):
        value = int(clamp(sample) * 32767.0)
        pcm.extend(value.to_bytes(2, byteorder="little", signed=True))

    with wave.open(str(path), "wb") as wav_file:
        wav_file.setnchannels(1)
        wav_file.setsampwidth(2)
        wav_file.setframerate(SAMPLE_RATE)
        wav_file.writeframes(bytes(pcm))


def render(duration: float, fn) -> list[float]:
    total = int(duration * SAMPLE_RATE)
    return [fn(i / SAMPLE_RATE) for i in range(total)]


def tone_sweep(duration: float, start_freq: float, end_freq: float, wave_fn, amp: float = 1.0) -> list[float]:
    def sample(t: float) -> float:
        progress = min(1.0, t / duration) if duration > 0 else 1.0
        freq = start_freq + (end_freq - start_freq) * progress
        return amp * wave_fn(freq, t)

    return render(duration, sample)


def ui_move() -> list[float]:
    duration = 0.08
    return render(
        duration,
        lambda t: 0.28
        * triangle(900.0 + 300.0 * (t / duration), t)
        * envelope(t, 0.002, 0.02, 0.4, 0.04, duration),
    )


def ui_confirm() -> list[float]:
    duration = 0.18
    return mix(
        [
            render(
                duration,
                lambda t: 0.22 * square(520.0, t) * envelope(t, 0.002, 0.04, 0.5, 0.08, duration),
            ),
            render(
                duration,
                lambda t: 0.18
                * square(780.0 + 220.0 * (t / duration), t)
                * envelope(t, 0.0, 0.03, 0.35, 0.09, duration),
            ),
        ]
    )


def player_shot() -> list[float]:
    duration = 0.14
    body = tone_sweep(duration, 1800.0, 380.0, square, 0.26)
    fizz = render(duration, lambda t: 0.12 * noise() * envelope(t, 0.0, 0.02, 0.16, 0.06, duration))
    return mix([body, lowpass(fizz, 0.18)])


def enemy_shot() -> list[float]:
    duration = 0.16
    body = tone_sweep(duration, 620.0, 220.0, triangle, 0.28)
    fizz = render(duration, lambda t: 0.09 * noise() * envelope(t, 0.0, 0.03, 0.18, 0.06, duration))
    return mix([body, lowpass(fizz, 0.12)])


def explosion_small() -> list[float]:
    duration = 0.55
    boom = render(duration, lambda t: 0.34 * noise() * envelope(t, 0.0, 0.09, 0.22, 0.32, duration))
    thump = tone_sweep(duration, 140.0, 42.0, sine, 0.24)
    return lowpass(mix([boom, thump]), 0.08)


def explosion_big() -> list[float]:
    duration = 0.95
    boom = render(duration, lambda t: 0.4 * noise() * envelope(t, 0.0, 0.14, 0.28, 0.45, duration))
    thump = tone_sweep(duration, 110.0, 28.0, sine, 0.3)
    crack = render(duration, lambda t: 0.1 * square(90.0 + 40.0 * t, t) * envelope(t, 0.0, 0.08, 0.2, 0.3, duration))
    return lowpass(mix([boom, thump, crack]), 0.06)


def pickup_scrap() -> list[float]:
    duration = 0.15
    return mix(
        [
            render(duration, lambda t: 0.16 * square(760.0, t) * envelope(t, 0.0, 0.03, 0.35, 0.06, duration)),
            render(duration, lambda t: 0.14 * square(1140.0, t) * envelope(t, 0.015, 0.02, 0.3, 0.05, duration)),
        ]
    )


def pickup_health() -> list[float]:
    duration = 0.28
    return render(
        duration,
        lambda t: 0.24
        * sine(420.0 + 480.0 * (t / duration), t)
        * envelope(t, 0.0, 0.05, 0.55, 0.12, duration),
    )


def shield_on() -> list[float]:
    duration = 0.42
    shimmer = render(
        duration,
        lambda t: 0.18
        * sine(280.0 + 620.0 * (t / duration), t)
        * envelope(t, 0.0, 0.1, 0.5, 0.14, duration),
    )
    air = render(duration, lambda t: 0.08 * noise() * envelope(t, 0.0, 0.05, 0.2, 0.18, duration))
    return lowpass(mix([shimmer, air]), 0.14)


def shield_hit() -> list[float]:
    duration = 0.18
    ping = render(duration, lambda t: 0.24 * triangle(1200.0 - 500.0 * (t / duration), t) * envelope(t, 0.0, 0.03, 0.28, 0.05, duration))
    crack = render(duration, lambda t: 0.1 * noise() * envelope(t, 0.0, 0.02, 0.1, 0.05, duration))
    return mix([ping, crack])


def ship_hit() -> list[float]:
    duration = 0.24
    impact = render(duration, lambda t: 0.22 * noise() * envelope(t, 0.0, 0.04, 0.16, 0.08, duration))
    alarm = render(duration, lambda t: 0.16 * square(320.0, t) * envelope(t, 0.0, 0.03, 0.25, 0.08, duration))
    return lowpass(mix([impact, alarm]), 0.12)


def mission_success() -> list[float]:
    duration = 0.8
    notes = [523.25, 659.25, 783.99]
    out = [0.0] * int(duration * SAMPLE_RATE)
    note_len = duration / len(notes)
    for idx, freq in enumerate(notes):
        start = int(idx * note_len * SAMPLE_RATE)
        note = render(
            note_len,
            lambda t, freq=freq, note_len=note_len: 0.18
            * square(freq, t)
            * envelope(t, 0.0, 0.03, 0.55, 0.12, note_len),
        )
        for i, sample in enumerate(note):
            out[start + i] += sample
    return out


def game_over() -> list[float]:
    duration = 0.9
    notes = [392.0, 311.13, 233.08]
    out = [0.0] * int(duration * SAMPLE_RATE)
    note_len = duration / len(notes)
    for idx, freq in enumerate(notes):
        start = int(idx * note_len * SAMPLE_RATE)
        note = render(
            note_len,
            lambda t, freq=freq, note_len=note_len: 0.2
            * triangle(freq, t)
            * envelope(t, 0.0, 0.04, 0.5, 0.14, note_len),
        )
        for i, sample in enumerate(note):
            out[start + i] += sample
    return out


def engine_loop() -> list[float]:
    duration = 1.4
    hum = render(duration, lambda t: 0.15 * triangle(90.0 + 4.0 * math.sin(2.0 * math.pi * 1.5 * t), t))
    whine = render(duration, lambda t: 0.06 * sine(420.0 + 14.0 * math.sin(2.0 * math.pi * 2.0 * t), t))
    grit = lowpass(render(duration, lambda t: 0.04 * noise()), 0.03)
    return mix([hum, whine, grit])


def music_loop(name: str, progression: list[float], bass: list[float], duration_per_bar: float, bars: int) -> list[float]:
    total = int(duration_per_bar * bars * SAMPLE_RATE)
    out = [0.0] * total
    for bar in range(bars):
        chord_freq = progression[bar % len(progression)]
        bass_freq = bass[bar % len(bass)]
        start = int(bar * duration_per_bar * SAMPLE_RATE)
        chord = render(
            duration_per_bar,
            lambda t, chord_freq=chord_freq, duration_per_bar=duration_per_bar: (
                0.08 * triangle(chord_freq, t)
                + 0.05 * triangle(chord_freq * 1.25, t)
                + 0.04 * sine(chord_freq * 1.5, t)
            )
            * envelope(t, 0.01, 0.18, 0.55, 0.2, duration_per_bar),
        )
        bass_line = render(
            duration_per_bar,
            lambda t, bass_freq=bass_freq, duration_per_bar=duration_per_bar: 0.1
            * square(bass_freq, t)
            * envelope(t, 0.0, 0.05, 0.45, 0.1, duration_per_bar),
        )
        pulse = render(
            duration_per_bar,
            lambda t, duration_per_bar=duration_per_bar: 0.03
            * noise()
            * envelope((t % 0.25), 0.0, 0.015, 0.12, 0.04, 0.12)
            * 0.6,
        )
        for i, sample in enumerate(mix([chord, bass_line, lowpass(pulse, 0.08)])):
            out[start + i] += sample
    return lowpass(out, 0.16)


def menu_loop() -> list[float]:
    return music_loop(
        "menu_loop",
        progression=[261.63, 329.63, 392.0, 329.63],
        bass=[65.41, 82.41, 98.0, 82.41],
        duration_per_bar=1.2,
        bars=4,
    )


def gameplay_loop() -> list[float]:
    return music_loop(
        "gameplay_loop",
        progression=[220.0, 246.94, 196.0, 174.61],
        bass=[55.0, 61.74, 49.0, 43.65],
        duration_per_bar=0.75,
        bars=8,
    )


def generate() -> None:
    random.seed(42)
    SFX_DIR.mkdir(parents=True, exist_ok=True)
    MUSIC_DIR.mkdir(parents=True, exist_ok=True)
    SOURCE_DIR.mkdir(parents=True, exist_ok=True)

    sfx = {
        "ui_move.wav": ui_move(),
        "ui_confirm.wav": ui_confirm(),
        "player_shot.wav": player_shot(),
        "enemy_shot.wav": enemy_shot(),
        "explosion_small.wav": explosion_small(),
        "explosion_big.wav": explosion_big(),
        "pickup_scrap.wav": pickup_scrap(),
        "pickup_health.wav": pickup_health(),
        "shield_on.wav": shield_on(),
        "shield_hit.wav": shield_hit(),
        "ship_hit.wav": ship_hit(),
        "mission_success.wav": mission_success(),
        "game_over.wav": game_over(),
        "engine_loop.wav": engine_loop(),
    }
    music = {
        "menu_loop.wav": menu_loop(),
        "gameplay_loop.wav": gameplay_loop(),
    }

    for name, samples in sfx.items():
        write_wav(SFX_DIR / name, samples)

    for name, samples in music.items():
        write_wav(MUSIC_DIR / name, samples)


if __name__ == "__main__":
    generate()
