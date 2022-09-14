// Iron fish is MIT licensed, (C) Stijn Kuipers

#![allow(unused)]
use {
    std::sync::Arc,
    crate::{
        audio::*,
        makepad_platform::live_atomic::*,
        makepad_platform::thread::*,
        makepad_media::*,
        makepad_media::audio_graph::*,
        makepad_draw_2d::*
    },
};

#[derive(Live, LiveHook, LiveAtomic, Debug, LiveRead)]
pub enum OscType {
    #[pick] DPWSawPulse,
    //  TrivialSaw,
    BlampTri,
    //  Naive,
    Pure
}

#[derive(Live, LiveHook, PartialEq, LiveAtomic, Debug, LiveRead)]
pub enum LFOWave {
    #[pick] Saw,
    Sine,
    Pulse,
    Triangle
}

#[derive(Live, LiveHook, PartialEq, LiveAtomic, Debug, LiveRead)]
pub enum FilterType {
    #[pick] LowPass,
    HighPass,
    BandPass,
    BandReject
}
/*
struct LaddFilterCoefficients {
    k: f64,
    alpha: f64,
    alpha0: f64,
    sigma: f64,
    bass_comp: f64,
    g: f64,
    subfilter_beta: [f64; 4]
}

impl Default for LaddFilterCoefficients {
    fn default() -> Self {
        Self {
            k: 1.0,
            alpha: 0.0,
            alpha0: 1.0,
            sigma: 1.0,
            bass_comp: 1.0,
            g: 1.0,
            
            subfilter_beta: [0.0, 0.0, 0.0, 0.0]
        }
    }
}*/

#[derive(Live, LiveHook, LiveAtomic, Debug, LiveRead, Clone)]
pub struct OscSettings {
    osc_type: U32A<OscType>,
    #[live(0)] transpose: i64a,
    #[live(0.0)] detune: f32a
}

#[derive(Live, LiveHook, LiveAtomic, Debug, LiveRead)]
pub struct EnvelopeSettings {
    #[live(0.0)] predelay: f32a,
    #[live(0.05)] a: f32a,
    #[live(0.0)] h: f32a,
    #[live(0.2)] d: f32a,
    #[live(0.5)] s: f32a,
    #[live(0.2)] r: f32a
}

#[derive(Live, LiveHook, LiveAtomic, Debug, LiveRead)]
pub struct LFOSettings {
    #[live(0.2)] rate: f32a,
    #[live(0)] keysync: u32a,
    waveform: U32A<LFOWave>
}

#[derive(Copy, Clone)]
pub struct LFOState
{
    phase: f32
}

#[derive(Live, LiveHook, LiveAtomic, Debug, LiveRead)]
pub struct FilterSettings {
    filter_type: U32A<FilterType>,
    #[live(0.5)] cutoff: f32a,
    #[live(0.05)] resonance: f32a,
    #[live(0.1)] envelope_amount: f32a,
    #[live(0.1)] lfo_amount: f32a,
    #[live(0.1)] touch_amount: f32a,
    #[live(0.0)] envelope_curvature: f32a
}

#[derive(Live, LiveHook, LiveAtomic, Debug, LiveRead)]
pub struct TouchSettings {
    #[live(0.5)] offset: f32a,
    #[live(1.0)] scale: f32a,
    #[live(0.5)] curve: f32a,
}


#[derive(Live, LiveHook, LiveAtomic, Debug, LiveRead)]
pub struct EffectSettings {
    #[live(0.5)] delaysend: f32a,
    #[live(0.8)] delayfeedback: f32a,
}


#[derive(Live, LiveHook, LiveAtomic, Debug, LiveRead)]
pub struct ArpSettings {
    #[live(true)] enabled: boola,
}

#[derive(Live, LiveHook, LiveAtomic, Debug, LiveRead)]
pub struct SequencerSettings {
    
    #[live(0)] pub step0: u32a,
    #[live(0)] pub step1: u32a,
    #[live(0)] pub step2: u32a,
    #[live(0)] pub step3: u32a,
    #[live(0)] pub step4: u32a,
    #[live(0)] pub step5: u32a,
    #[live(0)] pub step6: u32a,
    #[live(0)] pub step7: u32a,
    #[live(0)] pub step8: u32a,
    #[live(0)] pub step9: u32a,
    #[live(0)] pub step10: u32a,
    #[live(0)] pub step11: u32a,
    #[live(0)] pub step12: u32a,
    #[live(0)] pub step13: u32a,
    #[live(0)] pub step14: u32a,
    #[live(0)] pub step15: u32a,
    /*
    #[live(0)] step0: u32a,
    #[live(1)] step1: u32a,
    #[live(0)] step2: u32a,
    #[live(2)] step3: u32a,
    #[live(0)] step4: u32a,
    #[live(4)] step5: u32a,
    #[live(0)] step6: u32a,
    #[live(8)] step7: u32a,
    #[live(0)] step8: u32a,
    #[live(14)] step9: u32a,
    #[live(0)] step10: u32a,
    #[live(30)] step11: u32a,
    #[live(0)] step12: u32a,
    #[live(1)] step13: u32a,
    #[live(0)] step14: u32a,
    #[live(0)] step15: u32a,
*/
    
    #[live(125.0)] bpm: f32a,
    #[live(false)] playing: boola,
    
    #[live(0)] oneshot: u32a,
    #[live(1)] transposewithmidi: u32a,
    #[live(0)] polyphoniconeshot: u32a,
}


#[derive(Live, LiveHook, LiveAtomic, Debug, LiveRead)]
#[live_ignore]
pub struct IronFishSettings {
    osc1: OscSettings,
    osc2: OscSettings,
    subosc: OscSettings,
    lfo: LFOSettings,
    filter1: FilterSettings,
    volume_envelope: EnvelopeSettings,
    mod_envelope: EnvelopeSettings,
    touch: TouchSettings,
    fx: EffectSettings,
    pub sequencer: SequencerSettings,
    pub arp: ArpSettings,
    #[live(44100.0)] sample_rate: f32a,
    #[live(0.5)] osc_balance: f32a,
    #[live(0.5)] sub_osc: f32a,
    #[live(0.0)] noise: f32a,
}


#[derive(Copy, Clone)]
pub struct SequencerState
{
    
    currentstep: usize,
    samplesleftinstep: usize
}

#[derive(Copy, Clone)]
pub struct ArpState{
    step: usize,
    lastarpnote: u32,
    melody: [u32;128],
    melodylength: usize
}

#[derive(Copy, Clone)]
pub struct OscillatorState {
    phase: f32,
    delta_phase: f32,
    dpw_gain1: f32,
    dpw_gain2: f32,
    dpw_diff_b: [f32; 4],
    dpw_diff_b_write_index: i8, // diffB write index
    dpw_init_countdown: i8
}

#[derive(Copy, Clone)]
pub struct SubOscillatorState {
    phase: f32,
    delta_phase: f32,
}

impl SubOscillatorState {
    fn get(&mut self) -> f32 {
        self.phase += self.delta_phase;
        while self.phase > 1.0 {
            self.phase -= 1.0
        }
        // return self.dpw();
        
        return (self.phase * 6.283).sin();
    }
    
    fn set_note(&mut self, note: u8, samplerate: f32) {
        let freq = 440.0 * f32::powf(2.0, ((note as f32) - 69.0 - 24.0) / 12.0);
        self.delta_phase = ((6.283/2.0) * freq) / samplerate;
        let sampletime = 1.0 / samplerate;
    }
}

impl OscillatorState {
    fn dpw(&mut self) -> f32 {
        
        let triv = 2.0 * self.phase - 1.0;
        
        let sqr = triv * triv;
        
        let poly = sqr * sqr - 2.0 * sqr;
        
        self.dpw_diff_b[self.dpw_diff_b_write_index as usize] = poly;
        self.dpw_diff_b_write_index = self.dpw_diff_b_write_index + 1;
        
        if self.dpw_diff_b_write_index == 4 {self.dpw_diff_b_write_index = 0};
        
        if self.dpw_init_countdown > 0 {
            self.dpw_init_countdown = self.dpw_init_countdown - 1;
            
            return poly;
        }
        
        let mut tmp_a = [0.0, 0.0, 0.0, 0.0];
        let mut dbr = self.dpw_diff_b_write_index - 1;
        if dbr<0 {dbr = 3}
        
        for i in 0..4 {
            tmp_a[i] = self.dpw_diff_b[dbr as usize];
            dbr = dbr - 1;
            if dbr < 0 {
                dbr = 3;
            }
        }
        
        let gain = self.dpw_gain1;
        tmp_a[0] = gain * (tmp_a[0] - tmp_a[1]);
        tmp_a[1] = gain * (tmp_a[1] - tmp_a[2]);
        tmp_a[2] = gain * (tmp_a[2] - tmp_a[3]);
        
        tmp_a[0] = gain * (tmp_a[0] - tmp_a[1]);
        tmp_a[1] = gain * (tmp_a[1] - tmp_a[2]);
        
        tmp_a[0] = gain * (tmp_a[0] - tmp_a[1]);
        
        
        return tmp_a[0] * self.dpw_gain2 * 0.005; //* self.dpw_gain;
    }
    
    fn trivialsaw(self) -> f32 {
        return self.phase * 2.0 - 1.0
    }
    
    fn blamp(&mut self, t: f32, dt: f32) -> f32 {
        let mut y = 0.0;
        if 0.0 <= t && t<2.0 * dt {
            let x = t / dt;
            let u = 2.0 - x;
            let u2 = u * u;
            let u4 = u2 * u2;
            y -= u4;
            if t<dt {
                let v = 1.0 - x;
                let v2 = v * v;
                let v5 = v * v2 * v2;
                y += 4.0 * v5;
            }
        }
        return y * dt / 15.0
    }
    
    fn blamptriangle(&mut self) -> f32 {
        let mut tri = 2.0 * (2.0 * self.phase - 1.0).abs() - 1.0;
        
        tri += self.blamp(self.phase, self.delta_phase);
        tri += self.blamp(1.0 - self.phase, self.delta_phase);
        let mut t2 = self.phase + 0.5;
        t2 -= t2.floor();
        tri -= self.blamp(t2, self.delta_phase);
        tri -= self.blamp(1.0 - t2, self.delta_phase);
        return tri;
    }
    
    fn get(&mut self, settings: &OscSettings, _samplerate: f32) -> f32 {
        self.phase += self.delta_phase;
        while self.phase > 1.0 {
            self.phase -= 1.0
        }
        // return self.dpw();
        
        match settings.osc_type.get() {
            OscType::Pure => (self.phase * 6.283).sin(),
            OscType::DPWSawPulse => self.dpw(),
            //OscType::TrivialSaw => self.trivialsaw(),
            OscType::BlampTri => self.blamptriangle(),
        }
    }
    
    fn set_note(&mut self, note: u8, samplerate: f32, settings: &OscSettings) {
        let freq = 440.0 * f32::powf(2.0, ((note as f32) - 69.0 + settings.transpose.get() as f32 + settings.detune.get()) / 12.0);
        self.delta_phase = (6.283 * freq) / samplerate;
        //let w = freq * 6.283 / samplerate;
        let sampletime = 1.0 / samplerate;
        let prep = samplerate / freq; // / samplerate;
        //self.dpw_gain1 = (prep*prep*prep);
        //self.dpw_gain2 = 1.0/192.0 ;// (1.0 / 24.0 * (3.1415 / (2.0 * (3.1415 * prep).sin())).powf(3.0)).powf(1.0/3.0);
        self.dpw_gain1 = (1.0 / 24.0 * (3.1415 / (2.0 * (3.1415 / prep).sin())).powf(3.0)).powf(1.0 / 3.0);
        
        // println!("gain: {} {} ", self.dpw_gain, prep);
        //  gain = std::pow(1.f / factorial(dpwOrder) * std::pow(M_PI / (2.f*sin(M_PI*pitch * APP->engine->getSampleTime())),  dpwOrder-1.f), 1.0 / (dpwOrder-1.f));
    }
}

impl Default for SubOscillatorState {
    fn default() -> Self {
        Self {
            phase: 0.0,
            delta_phase: 0.0001,
        }
    }
}

impl Default for ArpState {
    fn default() -> Self {
        Self {
            step: 0,
            lastarpnote: 0,
            melody: [0u32;128],
            melodylength: 0
        }
    }
}


impl Default for OscillatorState {
    fn default() -> Self {
        Self {
            phase: 0.0,
            delta_phase: 0.0001,
            dpw_gain1: 1.0,
            dpw_gain2: 1.0,
            dpw_init_countdown: 4,
            dpw_diff_b: [3.0, 3.0, 3.0, 3.0],
            dpw_diff_b_write_index: 0
        }
    }
}

impl Default for FilterState {
    fn default() -> Self {
        Self {
            
            hp: 0.0,
            bp: 0.0,
            lp: 0.0,
            gamma: 0.0,
            fc: 0.0,
            phi: 0.0,
            damp: 0.0
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
enum EnvelopePhase {
    Idle,
    Predelay,
    Attack,
    Hold,
    Decay,
    Sustain,
    Release,
    FastRelease
}

#[derive(Copy, Clone)]
struct EnvelopeState {
    phase: EnvelopePhase,
    delta_value: f32,
    current_value: f32,
    target_value: f32,
    state_time: i32
}

impl Default for EnvelopeState {
    fn default() -> Self {
        Self {
            phase: EnvelopePhase::Idle,
            delta_value: 0.0,
            current_value: 0.0,
            target_value: 0.0,
            state_time: 0
        }
    }
}

impl EnvelopeState {
    
    fn get(&mut self, settings: &EnvelopeSettings, samplerate: f32) -> f32 {
        
        self.current_value = self.current_value + self.delta_value;
        self.state_time = self.state_time - 1;
        //println!("st {}", self.state_time);
        if self.state_time < -1 {self.state_time = -1;}
        if self.state_time == 0 {
            match self.phase {
                
                EnvelopePhase::Attack => {
                    if settings.h.get() != 0.0 {
                        self.phase = EnvelopePhase::Hold;
                        self.delta_value = 0.0;
                        self.current_value = 1.0;
                        self.target_value = 1.0;
                        self.state_time = EnvelopeState::nicerange(settings.h.get(), samplerate) as i32;
                        
                    }
                    else {
                        self.phase = EnvelopePhase::Decay;
                        let sustainlevel = settings.s.get() * settings.s.get();
                        self.delta_value = -(1.0 - sustainlevel) / EnvelopeState::nicerange(settings.d.get(), samplerate);
                        self.current_value = 1.0;
                        self.target_value = sustainlevel;
                        self.state_time = EnvelopeState::nicerange(settings.d.get(), samplerate) as i32;
                    }
                }
                
                EnvelopePhase::Hold => {
                    
                    let sustainlevel = settings.s.get() * settings.s.get();
                    self.phase = EnvelopePhase::Decay;
                    self.delta_value = -(1.0 - settings.s.get()) / EnvelopeState::nicerange(settings.d.get(), samplerate);
                    self.current_value = 1.0;
                    self.target_value = sustainlevel;
                    self.state_time = EnvelopeState::nicerange(settings.d.get(), samplerate) as i32;
                }
                
                EnvelopePhase::Decay => {
                    self.phase = EnvelopePhase::Sustain;
                    self.delta_value = 0.0;
                    let sustainlevel = settings.s.get() * settings.s.get();
                    self.current_value = sustainlevel;
                    self.target_value = sustainlevel;
                    self.state_time = -1;
                }
                
                EnvelopePhase::FastRelease |
                EnvelopePhase::Release => {
                    self.phase = EnvelopePhase::Idle;
                    self.delta_value = 0.0;
                    self.current_value = 0.0;
                    self.target_value = 0.0;
                    self.state_time = -1;
                }
                EnvelopePhase::Predelay => {
                    self.phase = EnvelopePhase::Attack;
                    self.delta_value = (1.0 - self.current_value) / EnvelopeState::nicerange(settings.a.get(), samplerate);
                    self.state_time = EnvelopeState::nicerange(settings.a.get(), samplerate) as i32;
                    self.target_value = 1.0;
                }
                
                _ => {}
            }
        }
        else
        {
            if self.phase == EnvelopePhase::Sustain {
                let sustainlevel = settings.s.get() * settings.s.get();
                self.current_value = sustainlevel;
                self.target_value = sustainlevel;
            }
        }
        return self.current_value;
    }
    
    fn trigger_on(&mut self, velocity: f32, settings: &EnvelopeSettings, samplerate: f32) {
        if settings.predelay.get() != 0.0 {
            if self.phase != EnvelopePhase::Idle {
                self.phase = EnvelopePhase::FastRelease;
                self.target_value = 0.0;
                self.delta_value = -self.current_value / 100.0;
                self.state_time = 100;
                return;
            }
            self.delta_value = 0.0;
            self.current_value = 0.0;
            self.target_value = 0.0;
            self.phase = EnvelopePhase::Predelay;
            self.state_time = EnvelopeState::nicerange(settings.predelay.get(), samplerate) as i32;
            return;
        };
        
        self.phase = EnvelopePhase::Attack;
        self.delta_value = (1.0 - self.current_value) / (EnvelopeState::nicerange(settings.a.get(), samplerate));
        self.state_time = EnvelopeState::nicerange(settings.a.get(), samplerate) as i32;
        self.target_value = 1.0;
        //  println!("attacking with {} {} {} {} {}",self.state_time,  settings.a, samplerate, settings.a * samplerate, self.delta_value);
    }
    
    fn nicerange(input: f32, samplerate: f32) -> f32 {
        return 1.0 + input * input * samplerate * 5.0;
    }
    
    fn trigger_off(&mut self, velocity: f32, settings: &EnvelopeSettings, samplerate: f32) {
        
        match self.phase {
            EnvelopePhase::Attack |
            EnvelopePhase::Decay |
            EnvelopePhase::Hold |
            EnvelopePhase::Sustain => {
                self.phase = EnvelopePhase::Release;
                self.target_value = 0.0;
                self.delta_value = -self.current_value / EnvelopeState::nicerange(settings.r.get(), samplerate);
                self.state_time = EnvelopeState::nicerange(settings.r.get(), samplerate) as i32;
            }
            _ => {}
        }
    }
}

#[derive(Copy, Clone)]
struct FilterState {
    
    hp: f32,
    bp: f32,
    lp: f32,
    phi: f32,
    gamma: f32,
    fc: f32,
    damp: f32
}

impl FilterState {
    fn pump(&mut self, input: f32) {
        self.bp = self.phi * self.hp + self.bp;
        self.lp = self.phi * self.bp + self.lp;
        self.hp = input - self.lp - self.gamma * self.bp;
    }
    
    fn get_lp(&mut self, input: f32) -> f32 {
        self.pump(input);
        return self.lp;
    }
    
    fn get_bp(&mut self, input: f32) -> f32 {
        self.pump(input);
        return self.bp;
    }
    
    fn get_br(&mut self, input: f32) -> f32 {
        self.pump(input);
        return input - self.bp;
    }
    
    fn get_hp(&mut self, input: f32) -> f32 {
        self.pump(input);
        return self.hp;
    }
    
    fn get(&mut self, input: f32, settings: &FilterSettings) -> f32 {
        match settings.filter_type.get() {
            FilterType::LowPass => self.get_lp(input),
            FilterType::HighPass => self.get_hp(input),
            FilterType::BandPass => self.get_bp(input),
            FilterType::BandReject => self.get_br(input)
        }
    }
    
    fn set_cutoff(&mut self, settings: &FilterSettings, envelope: f32, sample_rate: f32, touch: f32) {
        self.fc = (settings.cutoff.get() + touch * settings.touch_amount.get() + envelope * settings.envelope_amount.get() * 0.5).clamp(0.0, 1.0);
        self.fc *= self.fc * 0.5;
        self.damp = 1.0 - settings.resonance.get();
        let preclamp = 2.0 * ((3.1415 * self.fc).sin());
        self.phi = (preclamp).clamp(0.0, 1.0);
        self.gamma = (2.0 * self.damp).clamp(0.0, 1.0);
    }
}

struct GriesingerReverb {
    lp_: f32,
    diffusion_: f32,
    dphase1: f32,
    dphase2: f32,
    writepos: i32,
    buffer: [f32; 96000]
}
/*
impl GriesingerReverb {
    pub fn init(&mut self, sample_rate: f32) {
        self.dphase1 = 0.5 / sample_rate;
        self.dphase2 = 0.3 / sample_rate;
        self.lp_ = 0.7;
        self.diffusion_ = 0.625;
    }
    
    pub fn fill_buffer(&mut self, buffer: &mut AudioBuffer) {
        
        for i in 0..buffer.frame_count() {
            let output = 0.0;
            buffer.left[i] += output as f32;
            buffer.right[i] += output as f32;
        }
    }
}
*/

#[derive(Copy, Clone)]
pub struct IronFishVoice {
    osc1: OscillatorState,
    osc2: OscillatorState,
    subosc: SubOscillatorState,
    filter1: FilterState,
    volume_envelope: EnvelopeState,
    mod_envelope: EnvelopeState,
    current_note: i16,
    seed: u32,
    sequencer: SequencerState
}

fn random_bit(seed: &mut u32) -> u32 {
    *seed = seed.overflowing_add((seed.overflowing_mul(*seed)).0 | 5).0;
    return *seed >> 31;
}

fn random_f32(seed: &mut u32) -> f32 {
    let mut out = 0;
    for _ in 0..32 {
        out |= random_bit(seed);
        out <<= 1;
    }
    out as f32 / std::u32::MAX as f32
}

impl IronFishVoice {
    pub fn active(&mut self) -> i16 {
        if self.volume_envelope.phase == EnvelopePhase::Idle {
            return -1;
        }
        
        return self.current_note;
    }
    
    pub fn note_off(&mut self, b1: u8, b2: u8, settings: &IronFishSettings) {
        let velocity = (b2 as f32) / 127.0;
        self.volume_envelope.trigger_off(velocity, &settings.volume_envelope, settings.sample_rate.get());
        self.mod_envelope.trigger_off(velocity, &settings.mod_envelope, settings.sample_rate.get());
    }
    
    pub fn update_note(&mut self, settings: &IronFishSettings) {
        self.osc1.set_note(self.current_note as u8, settings.sample_rate.get(), &settings.osc1);
        self.osc2.set_note(self.current_note as u8, settings.sample_rate.get(), &settings.osc2);
    }
    
    pub fn note_on(&mut self, b1: u8, b2: u8, settings: &IronFishSettings) {
        
        let velocity = (b2 as f32) / 127.0;
        self.osc1.set_note(b1, settings.sample_rate.get(), &settings.osc1);
        self.osc2.set_note(b1, settings.sample_rate.get(), &settings.osc2);
        self.subosc.set_note(b1, settings.sample_rate.get());
        self.volume_envelope.trigger_on(velocity, &settings.volume_envelope, settings.sample_rate.get());
        self.mod_envelope.trigger_on(velocity, &settings.mod_envelope, settings.sample_rate.get());
        self.current_note = b1 as i16;
    }
    
    pub fn one(&mut self, settings: &IronFishSettings, touch: f32) -> f32 {
        
        let sub = self.subosc.get();
        
        let osc1 = self.osc1.get(&settings.osc1, settings.sample_rate.get());
        let osc2 = self.osc2.get(&settings.osc2, settings.sample_rate.get());
        let volume_envelope = self.volume_envelope.get(&settings.volume_envelope, settings.sample_rate.get());
        let mod_envelope = self.mod_envelope.get(&settings.mod_envelope, settings.sample_rate.get());
        
        self.filter1.set_cutoff(&settings.filter1, mod_envelope, settings.sample_rate.get(), touch);
        
        let noise = random_f32(&mut self.seed) * 2.0 - 1.0;
        
        let oscinput = osc2 * settings.osc_balance.get() + osc1 * (1.0 - settings.osc_balance.get()) + settings.sub_osc.get() * sub + settings.noise.get() * noise;
        let filter = self.filter1.get(oscinput, &settings.filter1);
        
        let output = volume_envelope * filter;
        
        return output * 0.006; //* 1000.0;
    }
    
    pub fn fill_buffer(&mut self, mix_buffer: &mut AudioBuffer, startidx: usize, frame_count: usize, display_buffer: Option<&mut AudioBuffer>, settings: &IronFishSettings, touch: f32) {
        
//        log!("blah {} {}", startidx, frame_count);
        let (left, right) = mix_buffer.stereo_mut();
        
        if let Some(display_buffer) = display_buffer {
            let (left_disp, right_disp) = display_buffer.stereo_mut();
            for i in startidx..frame_count+startidx {
                let output = self.one(&settings, touch) * 8.0;
                left_disp[i] = output as f32;
                right_disp[i] = output as f32;
                left[i] += output as f32;
                right[i] += output as f32;
            }
        }
        else {
            for i in startidx..frame_count+startidx {
                let output = self.one(&settings, touch) * 8.0;
                left[i] += output as f32;
                right[i] += output as f32;
            }
        }
    }
}

pub struct IronFishState {
    from_ui: FromUIReceiver<FromUI>,
    to_ui: ToUISender<ToUI>,
    display_buffers: Vec<AudioBuffer>,
    settings: Arc<IronFishSettings>,
    voices: [IronFishVoice; 16],
    activemidinotes: [bool; 256],
    activemidinotecount: usize,
    osc1cache: OscSettings,
    osc2cache: OscSettings,
    touch: f32,
    delayline: Vec<f32>,
    delayreadpos: usize,
    delaywritepos: usize,
    sequencer: SequencerState,
    arp: ArpState,
    lastplaying: bool,
    old_step: u32
}

impl IronFishState {
    
    pub fn note_off(&mut self, b1: u8, b2: u8) {
        if (self.settings.arp.enabled.get())
        {
            self.activemidinotes[b1 as usize] = false;
            if (self.activemidinotecount >0){   
                     self.activemidinotecount =  self.activemidinotecount  - 1;
            }
            self.rebuildarp();
        }
        else{
            self.internal_note_off(b1, b2);
        }
        
    }
    pub fn internal_note_off(&mut self, b1: u8, b2: u8){
        for i in 0..self.voices.len() {
            if self.voices[i].active() == b1 as i16 {
                self.voices[i].note_off(b1, b2, &self.settings)
            }
        }

    }
    pub fn note_on(&mut self, b1: u8, b2: u8) {
        if (self.settings.arp.enabled.get())
        {
            self.activemidinotes[b1 as usize] = true;
            self.activemidinotecount = self.activemidinotecount  + 1;
            self.rebuildarp();
      
        }
        else{
            self.internal_note_on(b1, b2);
        }
    }

    pub fn internal_note_on(&mut self, b1: u8, b2: u8) {
        for i in 0..self.voices.len() {
            if self.voices[i].active() == -1 {
                self.voices[i].note_on(b1, b2, &self.settings);
                return;
            }
        }
       
    }

    pub fn rebuildarp(&mut self)
    {
        log!("rebuilding arp");
        let mut current = 0;
        for i in 0..128 {
            if (self.activemidinotes[i] ) {
                self.arp.melody[current] = i as u32;
                current +=1;
                //   self.activemidinotecount += 1;
            }
        }
        self.arp.melodylength = current;
    }
    
   
    
    pub fn one(&mut self) -> f32 {
        let mut output: f32 = 0.0;
        for i in 0..self.voices.len() {
            output += self.voices[i].one(&self.settings, self.touch);
        }
        return output; //* 1000.0;
    }
    pub fn apply_delay(&mut self, buffer: &mut AudioBuffer) {
        let frame_count = buffer.frame_count();
        let (left, right) = buffer.stereo_mut();
        
        for i in 0..frame_count {
            let mut r = self.delayline[self.delayreadpos];
            r *= self.settings.fx.delayfeedback.get() * 0.9;
            r += self.settings.fx.delaysend.get() * (left[i] + right[i]);
            self.delayline[self.delaywritepos] = r;
            left[i] += r;
            right[i] += r;
            self.delaywritepos += 1;
            if (self.delaywritepos >= 44100) {self.delaywritepos = 0;}
            self.delayreadpos += 1;
            if (self.delayreadpos >= 44100) {self.delayreadpos = 0;}
        }
    }

    pub fn get_sequencer_step(&mut self, step: usize) -> u32
    {
        match step {
            0 => return self.settings.sequencer.step0.get(),
            1 => return self.settings.sequencer.step1.get(),
            2 => return self.settings.sequencer.step2.get(),
            3 => return self.settings.sequencer.step3.get(),
            4 => return self.settings.sequencer.step4.get(),
            5 => return self.settings.sequencer.step5.get(),
            6 => return self.settings.sequencer.step6.get(),
            7 => return self.settings.sequencer.step7.get(),
            8 => return self.settings.sequencer.step8.get(),
            9 => return self.settings.sequencer.step9.get(),
            10 => return self.settings.sequencer.step10.get(),
            11 => return self.settings.sequencer.step11.get(),
            12 => return self.settings.sequencer.step12.get(),
            13 => return self.settings.sequencer.step13.get(),
            14 => return self.settings.sequencer.step14.get(),
            15 => return self.settings.sequencer.step15.get(),
            _ => 0
            
        };
        return 0;
    }
    
    pub fn fill_buffer(&mut self, buffer: &mut AudioBuffer, display: &mut DisplayAudioGraph) {
        
        let mut disp_buffers = Vec::new();
        for i in 0..self.voices.len() {
            let mut dp = display.pop_buffer_resize(buffer.frame_count(), buffer.channel_count());
            if let Some(dp) = &mut dp{
                dp.zero();
            }
            disp_buffers.push(dp);
        }

        buffer.zero();
        //        if (self.settings.osc1.transpose != )
        let mut pitchdirty: bool = false;
        if (self.osc1cache.transpose.get() != self.settings.osc1.transpose.get()) {pitchdirty = true;}
        if (self.osc1cache.detune.get() != self.settings.osc1.detune.get()) {pitchdirty = true;}
        if (self.osc2cache.transpose.get() != self.settings.osc2.transpose.get()) {pitchdirty = true;}
        if (self.osc2cache.detune.get() != self.settings.osc2.detune.get()) {pitchdirty = true;}
        if (pitchdirty)
        {
            self.osc1cache.transpose.set(self.settings.osc1.transpose.get());
            self.osc1cache.detune.set(self.settings.osc1.detune.get());
            self.osc2cache.transpose.set(self.settings.osc2.transpose.get());
            self.osc2cache.detune.set(self.settings.osc2.detune.get());
            
            for i in 0..self.voices.len() {
                if self.voices[i].active() > -1 {
                    self.voices[i].update_note(&self.settings);
                }
            } 
        }
        let mut remaining = buffer.frame_count();
        let mut bufferidx = 0;
     //   log!("s{}", remaining);
            
        while (remaining > 0)
        {
            //log!("b{}", remaining);
            let mut toprocess = remaining;
           


                if (self.sequencer.samplesleftinstep == 0) {

                    if (self.settings.sequencer.playing.get())
                    {
                        if (self.lastplaying == false)
                        {
                            self.lastplaying = true;
                            self.sequencer.currentstep = 15;
                            self.old_step = 0;
                        }

                        // process notes!
                        let newstepidx = (self.sequencer.currentstep + 1) % 16;
                        let new_step = self.get_sequencer_step(newstepidx);
                        
                        //log!("tick! {:?} {:?}",newstepidx, new_step);
                        // minor scale..
                        let scale = [
                        
                                36 - 24, 38 - 24, 39 - 24, 41 - 24, 43 - 24, 44 - 24, 46 - 24, 
                                36 - 12, 38 - 12, 39 - 12, 41 - 12, 43 - 12, 44 - 12, 46 - 12, 
                                36     , 38     , 39     , 41     , 43     , 44     , 46     , 
                                36 + 12, 38 + 12, 39 + 12, 41 + 12, 43 + 12, 44 + 12, 46 + 12, 
                                36 + 24, 38 + 24, 39 + 24, 41 + 24, 43 + 24, 44 + 24, 46 + 24];
                    
                        for i in 0..32 {
                            if self.old_step & (1 << (31-i)) != 0 {
                                if (new_step & (1 << (31-i))) == 0 {
                                            //  log!("note off {:?}",scale[i]);
                                    self.internal_note_off(scale[i], 127);
                                }
                            } else {
                                if (new_step & (1 << (31-i)) != 0) {
                                        // log!("note on {:?}",scale[i]);
                                    self.internal_note_on(scale[i], 127);
                                }
                            }
                        }
                        self.old_step = new_step;
                    
                        self.sequencer.currentstep = newstepidx;
                    }

                    if  (self.settings.arp.enabled.get())
                    {   
                        self.internal_note_off(self.arp.lastarpnote as u8,127);
                                            
                        if  self.activemidinotecount > 0{
                            self.arp.lastarpnote = self.arp.melody[self.arp.step];
                            self.internal_note_on(self.arp.lastarpnote as u8,127);
                            self.arp.step = (self.arp.step + 1) % self.arp.melodylength.max(1);
                        }
                    }
                    self.sequencer.samplesleftinstep = ((self.settings.sample_rate.get() * 60.0) / (self.settings.sequencer.bpm.get() * 4.0)) as usize;
                }
               
                
                toprocess = toprocess.min(self.sequencer.samplesleftinstep);
                self.sequencer.samplesleftinstep -= toprocess;
            

            if (self.lastplaying && self.settings.sequencer.playing.get() == false)
            {
                self.all_notes_off();
                self.lastplaying = false;
            }
        
            for i in 0..self.voices.len() {
                if self.voices[i].active() > -1 {
                    self.voices[i].fill_buffer(buffer, bufferidx, toprocess, disp_buffers[i].as_mut(), &self.settings, self.touch);                   
                }
            }
            bufferidx += toprocess;
            remaining -= toprocess;
        }

        for (i,dp) in disp_buffers.into_iter().enumerate(){
            if let Some(dp) = dp{
                display.send_buffer(true, i, dp);
            }
        }

        self.apply_delay(buffer);
    }
}

impl Default for SequencerState {
    fn default() -> Self {
        Self {
            samplesleftinstep: 10,
            currentstep: 0
        }
    }
}

impl Default for IronFishVoice {
    fn default() -> Self {
        Self {
            osc1: OscillatorState::default(),
            osc2: OscillatorState::default(),
            subosc: SubOscillatorState::default(),
            filter1: FilterState::default(),
            volume_envelope: EnvelopeState::default(),
            mod_envelope: EnvelopeState::default(),
            sequencer: SequencerState::default(),
            current_note: -1,
            seed: 1234,
        }
    }
}

live_register!{
    IronFish: {{IronFish}} {
        settings: {}
    }
}

enum ToUI {
    DisplayAudio {
        voice: usize,
        buffer: AudioBuffer
    },
}

enum FromUI {
    DisplayAudio(AudioBuffer),
}

#[derive(Live, LiveHook)]
#[live_register(audio_component!(IronFish))]
pub struct IronFish {
    pub settings: Arc<IronFishSettings>,
    #[rust] to_ui: ToUIReceiver<ToUI>,
    #[rust] from_ui: FromUISender<FromUI>,
}

impl AudioGraphNode for IronFishState {
    fn all_notes_off(&mut self) {
        for i in 0..self.voices.len() {
            self.voices[i].volume_envelope.phase = EnvelopePhase::Idle;
            self.activemidinotes[i] = false;
        }
        self.activemidinotecount = 0;
        self.rebuildarp();
    }
    
    fn handle_midi_1_data(&mut self, data: Midi1Data) {
        match data.decode() {
            Midi1Event::Note(note) => {
                if note.is_on {
                    self.note_on(note.note_number, note.velocity);
                }
                else {
                    self.note_off(note.note_number, note.velocity);
                }
            }
            _ => ()
        }
        
        if (data.data0 == 0xb0 && data.data1 == 1) {
            self.touch = (data.data2 as f32 - 40.0) / (127.0 - 40.0);
            self.touch += self.settings.touch.offset.get();
            self.touch *= self.settings.touch.scale.get();
            self.touch = self.touch.powf(self.settings.touch.curve.get() * 3.0).min(1.0).max(-1.0);
        }
    }
    
    fn render_to_audio_buffer(&mut self, _time: AudioTime, outputs: &mut [&mut AudioBuffer], _inputs: &[&AudioBuffer], display: &mut DisplayAudioGraph) {
        self.fill_buffer(outputs[0], display)
    }
}

impl AudioComponent for IronFish {
    fn get_graph_node(&mut self, cx: &mut Cx) -> Box<dyn AudioGraphNode + Send> {
        self.from_ui.new_channel();
        let mut buffers = Vec::new();
        for i in 0..12 * 16 {
            buffers.push(AudioBuffer::default());
        }
        self.settings.osc1.clone();
        Box::new(IronFishState {
            display_buffers: buffers,
            settings: self.settings.clone(),
            voices: Default::default(),
            activemidinotes: [false;256],
            to_ui: self.to_ui.sender(),
            from_ui: self.from_ui.receiver(),
            osc1cache: self.settings.osc1.clone(),
            osc2cache: self.settings.osc2.clone(),
            touch: 0.0,
            delayline: vec![0.0f32; 44100],
            delaywritepos: 15000,
            delayreadpos: 0,
            sequencer: SequencerState::default(),
            lastplaying: false,
            old_step: 0,
            arp: Default::default(),
            activemidinotecount : 0
        })
    }
    
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, dispatch_action: &mut dyn FnMut(&mut Cx, AudioComponentAction)) {
    }
    // we dont have inputs
    fn audio_query(&mut self, _query: &AudioQuery, _callback: &mut Option<AudioQueryCb>) -> AudioResult {
        AudioResult::not_found()
    }
}


