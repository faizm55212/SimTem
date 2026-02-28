use eframe::egui::{self};
use memmap2::Mmap;
use mimalloc::MiMalloc;
use std::collections::VecDeque;
use std::fs::OpenOptions;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod data;
use data::{Graphics, Physics, Statics};
mod widgets;

const COLOR_GAS: egui::Color32 = egui::Color32::from_rgb(0, 120, 0);
const COLOR_BRAKE: egui::Color32 = egui::Color32::from_rgb(120, 0, 0);
const SCALE_FACTOR: f32 = 1.0;

struct OverlayApp {
    physics_mmap: Mmap,
    graphics_mmap: Mmap,
    statics_mmap: Mmap,

    gas_history: VecDeque<(f64, f64, bool)>,
    brake_history: VecDeque<(f64, f64, bool)>,
    start_time: std::time::Instant,
    last_update: std::time::Instant,
    time_window: f64,

    car_poll_timer: std::time::Instant,
    current_model_name: String,
    cached_shift_rpm: i32,
    cached_bb_offset: f32,

    // Recording / Testing Fields
    rec_last_gear: i32,
    rec_peak_rpm: i32,
    is_recording: bool,

    cache_fuel: String,
    last_fuel: f32,
    cache_bb: String,
    last_bb: f32,
    cache_tc: String,
    last_tc: f32,
    cache_abs: String,
    last_abs: f32,

    cache_gear: String,
    last_gear: i32,
    cache_speed: String,
    last_speed: i32,
    cache_rpm: String,
    last_rpm: i32,
    last_ign: i32,
    cache_gas_text: String,
    last_gas_int: i32,
    cache_brake_text: String,
    last_brake_int: i32,
}

impl OverlayApp {
    fn new(
        physics_mmap: Mmap,
        graphics_mmap: Mmap,
        statics_mmap: Mmap,
        cc: &eframe::CreationContext,
    ) -> Self {
        cc.egui_ctx.set_pixels_per_point(SCALE_FACTOR);

        Self {
            physics_mmap,
            graphics_mmap,
            statics_mmap,
            // Pre-allocate to prevent resizing
            gas_history: VecDeque::with_capacity(5000),
            brake_history: VecDeque::with_capacity(5000),
            start_time: std::time::Instant::now(),
            last_update: std::time::Instant::now(),
            time_window: 15.0,

            // Initialize all caches
            car_poll_timer: std::time::Instant::now(),
            current_model_name: String::new(),
            cached_shift_rpm: 0,
            cached_bb_offset: 0.0,

            // Init Recording defaults
            rec_last_gear: 0,
            rec_peak_rpm: 0,
            is_recording: false,

            cache_fuel: "0.0".to_string(),
            last_fuel: -1.0,
            cache_bb: "50.0".to_string(),
            last_bb: -1.0,
            cache_tc: "0".to_string(),
            last_tc: -1.0,
            cache_abs: "0".to_string(),
            last_abs: -1.0,

            cache_gear: "N".to_string(),
            last_gear: -999,
            cache_speed: "0".to_string(),
            last_speed: -1,
            cache_rpm: "0".to_string(),
            last_rpm: -1,
            last_ign: -1,
            cache_gas_text: "0".to_string(),
            last_gas_int: -1,
            cache_brake_text: "0".to_string(),
            last_brake_int: -1,
        }
    }

    fn get_physics(&self) -> &Physics {
        unsafe { &*(self.physics_mmap.as_ptr() as *const Physics) }
    }
    fn get_graphics(&self) -> &Graphics {
        unsafe { &*(self.graphics_mmap.as_ptr() as *const Graphics) }
    }
    fn get_statics(&self) -> &Statics {
        unsafe { &*(self.statics_mmap.as_ptr() as *const Statics) }
    }
}

impl eframe::App for OverlayApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.car_poll_timer.elapsed().as_secs() >= 2 {
            let s = self.get_statics();
            let raw_name = data::parse_static_string(&s.car_model[..]);

            if raw_name != self.current_model_name {
                println!("Car Detected: '{}'", raw_name);
                self.current_model_name = raw_name.clone();

                if let Some(car) = data::get_car_by_name(&raw_name) {
                    println!(" -> Match Found! Shift: {}", car.shift_rpm);
                    self.cached_shift_rpm = car.shift_rpm;
                    self.cached_bb_offset = car.bb_offset;
                } else {
                    println!(" -> No Match in car_data.rs");
                    self.cached_shift_rpm = 0;
                    self.cached_bb_offset = 0.0;
                }
            }
            self.car_poll_timer = std::time::Instant::now();
        }

        let (gas, brake, gear, speed_kmh, rpm, max_rpm, fuel, bb, tc_act, abs_act, ign) = {
            let p = self.get_physics();
            (
                p.gas,
                p.brake,
                p.gear,
                p.speed_kmh,
                p.rpms,
                p.current_max_rpm,
                p.fuel,
                p.brake_bias,
                p.tc == 1.0,
                p.abs == 1.0,
                p.ignition_on,
            )
        };

        let (tc, abs) = {
            let g = self.get_graphics();
            (g.tc, g.abs)
        };

        // --- RECORDING LOGIC (Fixed with Peak Tracking) ---
        if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
            self.is_recording = !self.is_recording;
            println!(
                "RECORDING MODE: {}",
                if self.is_recording { "ON" } else { "OFF" }
            );

            if gear >= 2 {
                self.rec_last_gear = gear;
                self.rec_peak_rpm = rpm;
            }
        }

        if rpm > self.rec_peak_rpm {
            self.rec_peak_rpm = rpm;
        }

        if self.is_recording {
            if gear >= 2 {
                if gear > self.rec_last_gear && self.rec_last_gear >= 2 {
                    let from_gear = self.rec_last_gear - 1;
                    let to_gear = gear - 1;

                    println!(
                        "CAPTURED: Gear {} -> {} at {} RPM",
                        from_gear, to_gear, self.rec_peak_rpm
                    );

                    self.rec_last_gear = gear;
                    self.rec_peak_rpm = rpm;
                } else if gear < self.rec_last_gear {
                    self.rec_last_gear = gear;
                    self.rec_peak_rpm = rpm;
                }
            }
        } else if gear >= 2 && gear != self.rec_last_gear {
            self.rec_last_gear = gear;
            self.rec_peak_rpm = rpm;
        }
        // --------------------------------

        if (fuel - self.last_fuel).abs() > 0.05 {
            self.cache_fuel = format!("{:.1}", fuel);
            self.last_fuel = fuel;
        }
        let bb_disp = if bb == 0.0 {
            0.0
        } else {
            (bb * 100.0) + self.cached_bb_offset
        };

        if (bb_disp - self.last_bb).abs() > 0.05 {
            self.cache_bb = format!("{:.1}", bb_disp);
            self.last_bb = bb_disp;
        }
        if (tc as f32 - self.last_tc).abs() > 0.1 {
            self.cache_tc = format!("{}", tc);
            self.last_tc = tc as f32;
        }
        if (abs as f32 - self.last_abs).abs() > 0.1 {
            self.cache_abs = format!("{}", abs);
            self.last_abs = abs as f32;
        }
        if gear != self.last_gear {
            self.cache_gear = match gear {
                0 => "R".to_string(),
                1 => "N".to_string(),
                g => (g - 1).to_string(),
            };
            self.last_gear = gear;
        }
        let speed_int = speed_kmh as i32;
        if speed_int != self.last_speed {
            self.cache_speed = speed_int.to_string();
            self.last_speed = speed_int;
        }
        if rpm != self.last_rpm || ign != self.last_ign {
            self.cache_rpm = if ign != 1 {
                "IGNITION OFF".to_string()
            } else if rpm <= 0 {
                "ENGINE OFF".to_string()
            } else {
                rpm.to_string()
            };
            self.last_rpm = rpm;
            self.last_ign = ign;
        }
        let gas_pct = (gas * 100.0) as i32;
        if gas_pct != self.last_gas_int {
            self.cache_gas_text = if gas_pct > 99 {
                "F".to_string()
            } else {
                gas_pct.to_string()
            };
            self.last_gas_int = gas_pct;
        }
        let brake_pct = (brake * 100.0) as i32;
        if brake_pct != self.last_brake_int {
            self.cache_brake_text = if brake_pct > 99 {
                "F".to_string()
            } else {
                brake_pct.to_string()
            };
            self.last_brake_int = brake_pct;
        }

        let now = std::time::Instant::now();
        let t = now.duration_since(self.start_time).as_secs_f64();

        if self.last_update.elapsed().as_millis() >= 3 {
            self.gas_history.push_back((t, gas as f64, tc_act));
            self.brake_history.push_back((t, brake as f64, abs_act));
            self.last_update = now;

            if self.gas_history.len() > 3000 {
                self.gas_history.pop_front();
                self.brake_history.pop_front();
            }
        }

        let min_time = t - self.time_window;
        while let Some(&(time, _, _)) = self.gas_history.front() {
            if time < min_time {
                self.gas_history.pop_front();
                self.brake_history.pop_front();
            } else {
                break;
            }
        }

        let panel_frame = egui::Frame::NONE
            .fill(egui::Color32::from_rgba_premultiplied(0, 0, 0, 100))
            .inner_margin(10.0);

        egui::CentralPanel::default()
            .frame(panel_frame)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    widgets::gear_indicator(ui, &self.cache_gear);
                    ui.add_space(20.0);
                    ui.vertical(|ui| {
                        let rpm_pct = if max_rpm > 0 {
                            (rpm as f32 / max_rpm as f32).clamp(0.0, 1.0)
                        } else {
                            0.0
                        };

                        let (is_strobe, is_yellow) = if self.cached_shift_rpm > 0 {
                            (
                                rpm > self.cached_shift_rpm,
                                rpm > (self.cached_shift_rpm - 200),
                            )
                        } else {
                            let f_rpm = rpm as f32;
                            let f_max = max_rpm as f32;
                            (f_rpm > f_max * 0.95, f_rpm > f_max * 0.92)
                        };

                        widgets::rev_strip(
                            ui,
                            rpm_pct,
                            is_yellow,
                            is_strobe,
                            &self.cache_rpm,
                            ign == 1,
                        );
                        ui.add_space(5.0);
                        ui.horizontal(|ui| {
                            widgets::speedometer(ui, &self.cache_speed);
                            widgets::stat_box(ui, "fuel", &self.cache_fuel);
                            widgets::stat_box(ui, "TC", &self.cache_tc);
                            widgets::stat_box(ui, "BB", &self.cache_bb);
                            widgets::stat_box(ui, "ABS", &self.cache_abs);
                        });
                    });
                });
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    widgets::telemetry_graph(
                        ui,
                        &self.gas_history,
                        &self.brake_history,
                        self.time_window,
                        t,
                        COLOR_GAS,
                        COLOR_BRAKE,
                    );
                    widgets::pedal_bar(ui, brake, COLOR_BRAKE, abs_act, &self.cache_brake_text);
                    widgets::pedal_bar(ui, gas, COLOR_GAS, tc_act, &self.cache_gas_text);
                });
            });
        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        vsync: true,
        viewport: egui::ViewportBuilder::default()
            // .with_transparent(true)
            // .with_always_on_top()
            // .with_mouse_passthrough(true)
            .with_decorations(false)
            .with_inner_size([550.0, 250.0]),
        ..Default::default()
    };

    let physics_file = OpenOptions::new()
        .read(true)
        .open("/dev/shm/acpmf_physics")
        .expect("No Physics SHM");
    let graphics_file = OpenOptions::new()
        .read(true)
        .open("/dev/shm/acpmf_graphics")
        .expect("No Graphics SHM");
    let statics_file = OpenOptions::new()
        .read(true)
        .open("/dev/shm/acpmf_static")
        .expect("No Static SHM");
    let physics_mmap = unsafe { Mmap::map(&physics_file).expect("Map Fail") };
    let graphics_mmap = unsafe { Mmap::map(&graphics_file).expect("Map Fail") };
    let statics_mmap = unsafe { Mmap::map(&statics_file).expect("Map Fail") };

    eframe::run_native(
        "AC Overlay",
        options,
        Box::new(|cc| {
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                "formula".to_owned(),
                egui::FontData::from_static(include_bytes!("./assets/Formula1-Regular.ttf")).into(),
            );
            fonts
                .families
                .get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .insert(0, "formula".to_owned());
            cc.egui_ctx.set_fonts(fonts);

            Ok(Box::new(OverlayApp::new(
                physics_mmap,
                graphics_mmap,
                statics_mmap,
                cc,
            )))
        }),
    )
}
