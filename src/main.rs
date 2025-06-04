use eframe::App;
use eframe::Frame;
use eframe::egui;
use eframe::egui::CentralPanel;
use eframe::egui::TextEdit;
// use eframe::egui::FontData;
// use eframe::egui::FontDefinitions;
// use eframe::egui::FontFamily;
use eframe::egui::TopBottomPanel;
use eframe::icon_data;

fn main() -> eframe::Result {
    let icon_bytes = include_bytes!("../assets/corgi.png");
    let icon = icon_data::from_png_bytes(icon_bytes).expect("failed to load icon");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_icon(icon),
        centered: true,
        ..Default::default()
    };
    // Use MSYH font for other non-English user, but this font will look a bit blurry.
    // let font_bytes = include_bytes!("../assets/MSYH.TTC");
    // let mut fonts = FontDefinitions::default();
    // fonts.font_data.insert(
    //     "MSYH".to_owned(),
    //     FontData::from_owned(font_bytes.to_vec()).into(),
    // );
    // fonts
    //     .families
    //     .entry(FontFamily::Monospace)
    //     .or_default()
    //     .insert(0, "MSYH".to_owned());
    // fonts
    //     .families
    //     .entry(egui::FontFamily::Proportional)
    //     .or_default()
    //     .insert(0, "MSYH".to_owned());

    let app_name = format!("renmap v{}", env!("CARGO_PKG_VERSION"));
    eframe::run_native(
        &app_name,
        options,
        Box::new(|_cc| {
            // cc.egui_ctx.set_zoom_factor(1.1);
            // cc.egui_ctx.set_pixels_per_point(1.1);
            // cc.egui_ctx.set_fonts(fonts);
            Ok(Box::<RenmapApp>::default())
        }),
    )
}

struct RenmapMode {
    fast_mode: bool,
    no_ping: bool,
}

impl Default for RenmapMode {
    fn default() -> Self {
        RenmapMode {
            fast_mode: false,
            no_ping: false,
        }
    }
}

struct RenmapApp {
    target_addr: String,
    target_port: String,
    mode: RenmapMode,
}

impl Default for RenmapApp {
    fn default() -> Self {
        Self {
            target_addr: "127.0.0.1".to_owned(),
            target_port: "8080".to_owned(),
            mode: RenmapMode::default(),
        }
    }
}

impl App for RenmapApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("File").clicked() {}
                    if ui.button("Save").clicked() {}
                });
                ui.menu_button("Edit", |ui| {
                    if ui.button("prev").clicked() {}
                    if ui.button("redo").clicked() {}
                });
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.columns(3, |columns| {
                columns[0].horizontal(|ui| {
                    ui.label("Target");
                    ui.add_sized(
                        ui.available_size(),
                        TextEdit::singleline(&mut self.target_addr),
                    );
                });
                columns[1].horizontal(|ui| {
                    ui.label("Port(s)");
                    ui.add_sized(
                        ui.available_size(),
                        TextEdit::singleline(&mut self.target_port),
                    );
                });
                columns[2].horizontal(|ui| {
                    if ui.button("Scan").clicked() {
                        // start scan
                    };
                    ui.separator();
                    ui.label(format!(
                        "Test info {}:{}",
                        &self.target_addr, &self.target_port
                    ));
                });
            });
            // Added a empty line here.
            ui.horizontal(|_| {});
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.mode.fast_mode, "Fast Mode");
                ui.checkbox(&mut self.mode.no_ping, "No Ping");
            });

            // ui.separator();
        });
    }
}
