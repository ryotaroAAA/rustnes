/*
    ユニット                 矩形波 三角波 ノイズ  DMC
    ----------------------------------------------------
    ボリューム/エンベロープ     ○             ○
    タイマ                     ○      ○      ○     ○
    長さカウンタ                ○      ○      ○
    スイープユニット            ○
    デューティ                  ○
    線形カウンタ                       ○
    三角ステップ生成器                  ○
    波長コンバータ                            ○     ○
    擬似ランダム生成器                         ○

    Registers	Channel	Units
    $4000-$4003	Pulse 1	Timer, length counter, envelope, sweep
    $4004-$4007	Pulse 2	Timer, length counter, envelope, sweep
    $4008-$400B	Triangle	Timer, length counter, linear counter
    $400C-$400F	Noise	Timer, length counter, envelope, linear feedback shift register
    $4010-$4013	DMC	Timer, memory reader, sample buffer, output unit
    $4015	All	Channel enable and length counter status
    $4017	All	Frame counter
*/

use super::Interrupts;
use super::Ram;

const CYCLE_240HZ: usize = 7457;
const APU_REGISTER_SIZE: usize = 0x18;
const CPU_CLOCK: f32 = 1789772.5;
const GLOBAL_GAIN: f32 = 0.01;
const SAMPLING_FREQUENCY: usize = 44100;
const COUNTER_TABLE: [u8; 32] = [
  0x0A, 0xFE, 0x14, 0x02, 0x28, 0x04, 0x50, 0x06,
  0xA0, 0x08, 0x3C, 0x0A, 0x0E, 0x0C, 0x1A, 0x0E,
  0x0C, 0x10, 0x18, 0x12, 0x30, 0x14, 0x60, 0x16,
  0xC0, 0x18, 0x48, 0x1A, 0x10, 0x1C, 0x20, 0x1E,
];

const NOISE_TIMER_PERIOD_TABLE: [u16; 16] = [
  0x004, 0x008, 0x010, 0x020,
  0x040, 0x060, 0x080, 0x0A0,
  0x0CA, 0x0FE, 0x17C, 0x1FC,
  0x2FA, 0x3F8, 0x7F2, 0xFE4,
];

const DMC_TIMER_PERIOD_TABLE: [u16; 16] = [
  0x1AC, 0x17C, 0x154, 0x140,
  0x11E, 0x0FE, 0x0E2, 0x0D6,
  0x0BE, 0x0A0, 0x08E, 0x080,
  0x06A, 0x054, 0x048, 0x036,
];

#[derive(Debug)]
pub struct Apu {
    cycle: u64,
    step: u32,
    register: Ram,
    envelope_counter: u8,
    is_sequencer_mode: bool,
    is_enable_irq: bool,
}

impl Apu {
    pub fn new () -> Apu {
        Apu {
            cycle: 0,
            step: 0,
            register: Ram::new(APU_REGISTER_SIZE),
            envelope_counter: 0,
            is_sequencer_mode: false,
            is_enable_irq: false,
        }
    }
    pub fn read(&self, interrupts: &mut Interrupts, addr: u16) -> u8{
        match addr {
            0x4015 => {
                interrupts.deassert_irq();
                return self.register.read(0x15)
            },
            _ => panic!("invalid addr: {}", addr),
        }
    }
    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            // square wave 1 control register
            0x4000..=0x4004 => {

            },
            // square wave 2 control register
            0x4004..=0x4008 => {

            },
            // triange wave control register
            0x4008..=0x400C => {

            },
            // noise control register
            0x400C..=0x400F => {

            },
            // DPCM control register
            0x4010..=0x4013 => {

            },
            // audio channel control register
            0x4015 => {

            },
            0x4017 => {

            },
            _ => panic!("invalid addr {} {}", addr, data),
        }
    }

    fn update_mode0_sequence(&mut self, interrupts: &mut Interrupts) {
        self.update_envelope();
        if self.step % 2 > 0 {
            self.update_sweep_and_length_counter();
        }
        self.step += 1;

        if self.step == 4 {
            if self.is_enable_irq {
                interrupts.assert_irq();
            }
            self.step = 0;
        }
    }
    fn update_mode1_sequence(&mut self, interrupts: &mut Interrupts) {
        if self.step % 2 == 0 {
            self.update_sweep_and_length_counter();
        }
        self.step += 1;
        if self.step == 5 {
            self.step = 0;
        } else {
            self.update_envelope();
        }
    }
    fn update_envelope(&self) {
        // self.square1.update_envelope();
        // self.square2.update_envelope();
        // self.triangle.update_envelope();
        // self.noise.update_envelope();
    }

    fn update_sweep_and_length_counter(&self) {
        // self.square1.update_sweep_and_length_counter();
        // self.square2.update_sweep_and_length_counter();
        // self.triangle.update_sweep_and_length_counter();
        // self.noise.update_sweep_and_length_counter();
    }
    pub fn run(&mut self, cycle: u64, interrupts: &mut Interrupts) {
        self.cycle += cycle;
        if self.cycle >= CYCLE_240HZ as u64 {
            self.cycle -= CYCLE_240HZ as u64;
            if self.is_sequencer_mode {
                self.update_mode1_sequence(interrupts);
            } else {
                self.update_mode0_sequence(interrupts);
            }
        }
    }
}