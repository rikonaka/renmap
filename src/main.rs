use eframe::App;
use eframe::Frame;
use eframe::egui;
use eframe::egui::CentralPanel;
use eframe::egui::ComboBox;
use eframe::egui::TextEdit;
use eframe::egui::containers;
// use eframe::egui::FontData;
// use eframe::egui::FontDefinitions;
// use eframe::egui::FontFamily;
use eframe::egui::Align;
use eframe::egui::Button;
use eframe::egui::Grid;
use eframe::egui::Label;
use eframe::egui::Layout;
use eframe::egui::RichText;
use eframe::egui::Slider;
use eframe::egui::TopBottomPanel;
use eframe::egui::Widget;
use eframe::egui::vec2;
use eframe::icon_data;
use egui_extras::Column;
use egui_extras::TableBuilder;

fn main() -> eframe::Result {
    let icon_bytes = include_bytes!("../assets/corgi.png");
    let icon = icon_data::from_png_bytes(icon_bytes).expect("failed to load icon");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_icon(icon),
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
        Box::new(|cc| {
            // cc.egui_ctx.set_zoom_factor(1.1);
            cc.egui_ctx.set_pixels_per_point(1.1);
            // cc.egui_ctx.set_fonts(fonts);
            Ok(Box::<RenmapApp>::default())
        }),
    )
}

struct RenmapApp {
    target_addr: String,
    target_port: String,
}

impl Default for RenmapApp {
    fn default() -> Self {
        Self {
            target_addr: "127.0.0.1".to_owned(),
            target_port: "8080".to_owned(),
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
                ui.menu_button("edit", |ui| {
                    if ui.button("prev").clicked() {}
                    if ui.button("redo").clicked() {}
                });
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            let label_target = Label::new("Target");
            let label_port = Label::new("Port");
            let input_target = TextEdit::singleline(&mut self.target_addr);
            let input_port = TextEdit::singleline(&mut self.target_port);
            let button_scan = Button::new("Scan");

            TableBuilder::new(ui)
                .column(Column::exact(100.0))
                .column(Column::remainder())
                .column(Column::exact(100.0))
                .body(|mut body| {
                    body.row(20.0, |mut row| {
                        row.col(|ui| {
                            ui.label("固定列");
                        });
                        row.col(|ui| {
                            ui.label("会伸缩的列");
                        });
                    });
                });

            ui.horizontal(|ui| {
                ui.add(label_target);
                ui.add(input_target);
                ui.add(label_port);
                ui.add(input_port);
                ui.add(button_scan);
            });
            // ui.separator();
        });
    }
}
