/** WebAudio 合成音效，无需外部资源文件 */

let ctx: AudioContext | null = null;
let enabled = true;

export function setSoundEnabled(v: boolean) {
  enabled = v;
}

function ac(): AudioContext | null {
  if (!enabled) return null;
  if (!ctx) {
    try {
      ctx = new AudioContext();
    } catch {
      return null;
    }
  }
  if (ctx.state === "suspended") {
    void ctx.resume();
  }
  return ctx;
}

function tone(freq: number, dur: number, type: OscillatorType = "sine", gain = 0.16, delay = 0, freqEnd?: number) {
  const c = ac();
  if (!c) return;
  const t0 = c.currentTime + delay;
  const osc = c.createOscillator();
  const g = c.createGain();
  osc.type = type;
  osc.frequency.setValueAtTime(freq, t0);
  if (freqEnd) osc.frequency.exponentialRampToValueAtTime(freqEnd, t0 + dur);
  g.gain.setValueAtTime(0.0001, t0);
  g.gain.exponentialRampToValueAtTime(gain, t0 + 0.008);
  g.gain.exponentialRampToValueAtTime(0.0001, t0 + dur);
  osc.connect(g).connect(c.destination);
  osc.start(t0);
  osc.stop(t0 + dur + 0.05);
}

function noise(dur: number, gain = 0.1, delay = 0, lowpass = 1200) {
  const c = ac();
  if (!c) return;
  const t0 = c.currentTime + delay;
  const len = Math.floor(c.sampleRate * dur);
  const buf = c.createBuffer(1, len, c.sampleRate);
  const data = buf.getChannelData(0);
  for (let i = 0; i < len; i++) {
    data[i] = (Math.random() * 2 - 1) * (1 - i / len);
  }
  const src = c.createBufferSource();
  src.buffer = buf;
  const filter = c.createBiquadFilter();
  filter.type = "lowpass";
  filter.frequency.value = lowpass;
  const g = c.createGain();
  g.gain.value = gain;
  src.connect(filter).connect(g).connect(c.destination);
  src.start(t0);
}

export const sfx = {
  select() {
    tone(660, 0.04, "triangle", 0.05);
  },
  move() {
    tone(520, 0.06, "triangle", 0.18, 0, 440);
    noise(0.03, 0.06, 0, 2000);
  },
  capture() {
    tone(220, 0.09, "square", 0.16, 0, 140);
    noise(0.08, 0.14, 0, 900);
  },
  check() {
    tone(880, 0.07, "sawtooth", 0.1);
    tone(1170, 0.09, "sawtooth", 0.1, 0.08);
  },
  illegal() {
    tone(180, 0.12, "square", 0.12);
  },
  win() {
    [523, 659, 784, 1046].forEach((f, i) => tone(f, 0.16, "triangle", 0.14, i * 0.11));
  },
  lose() {
    [440, 370, 294, 220].forEach((f, i) => tone(f, 0.18, "sine", 0.13, i * 0.13));
  },
  draw() {
    [440, 523].forEach((f, i) => tone(f, 0.14, "sine", 0.11, i * 0.12));
  },
  puzzleOk() {
    [660, 880].forEach((f, i) => tone(f, 0.1, "triangle", 0.12, i * 0.08));
  },
  puzzleBad() {
    tone(240, 0.14, "square", 0.12, 0, 160);
  },
};
