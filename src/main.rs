use eframe::App;
use eframe::Frame;
use eframe::egui;
use eframe::egui::CentralPanel;
use eframe::egui::Context;
use eframe::egui::RichText;
// use eframe::egui::FontData;
// use eframe::egui::FontDefinitions;
// use eframe::egui::FontFamily;
use eframe::egui::Align2;
use eframe::egui::CollapsingHeader;
use eframe::egui::Direction;
use eframe::egui::Layout;
use eframe::egui::ScrollArea;
use eframe::egui::SidePanel;
use eframe::egui::TextEdit;
use eframe::egui::TopBottomPanel;
use eframe::egui::Window;
use eframe::egui::containers;
use eframe::icon_data;
use egui_extras::Column;
use egui_extras::TableBuilder;
use pistol::scan::TcpUdpScans;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

mod db;
mod scan;

use db::SqliteDB;
use scan::pistol_scan;

static CURRENT_SCAN: LazyLock<Arc<Mutex<Option<TcpUdpScans>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(None)));

static SCAN_HISTORYS: LazyLock<Arc<Mutex<Vec<TcpUdpScans>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(Vec::new())));

static SQLITE_DB: LazyLock<Arc<Mutex<SqliteDB>>> = LazyLock::new(|| {
    Arc::new(Mutex::new(
        SqliteDB::init_db(false).expect("init sqlite db failed"),
    ))
});

fn main() -> eframe::Result {
    // let icon_bytes = include_bytes!("../assets/corgi.png");
    // let icon_bytes = include_bytes!("../assets/hacker.png");
    let icon_bytes = include_bytes!("../assets/R.png");
    let icon = icon_data::from_png_bytes(icon_bytes).expect("failed to load icon");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 600.0])
            .with_icon(icon),
        centered: true,
        ..Default::default()
    };
    // Use MSYH font for other non-English user, but this font will look a bit blurry in my PC.
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
            cc.egui_ctx.set_zoom_factor(1.1);
            // cc.egui_ctx.set_pixels_per_point(1.0);
            // cc.egui_ctx.set_fonts(fonts);
            Ok(Box::<RenmapApp>::default())
        }),
    )
}

#[derive(Debug, Clone, Copy)]
struct ScanOptions {
    fast_mode: bool,
    slow_mode: bool,
    no_ping: bool,
}

impl Default for ScanOptions {
    fn default() -> Self {
        ScanOptions {
            fast_mode: false,
            slow_mode: false,
            no_ping: false,
        }
    }
}

#[derive(Debug, Clone)]
enum AppInterface {
    Scan,
    About,
}

impl Default for AppInterface {
    fn default() -> Self {
        Self::Scan
    }
}

#[derive(Debug, Clone)]
struct ScanParas {
    target_addr: String,
    target_port: String,
    exclued_target_addr: String,
    exclued_target_port: String,
    timeout: String,
    scan_options: ScanOptions,
    in_memory: bool,
    in_memroy_confirm: bool,
}

impl Default for ScanParas {
    fn default() -> Self {
        Self {
            target_addr: "192.168.1.2".to_owned(),
            target_port: "22".to_owned(),
            exclued_target_addr: "".to_owned(),
            exclued_target_port: "".to_owned(),
            timeout: "".to_owned(),
            scan_options: ScanOptions::default(),
            in_memory: false,
            in_memroy_confirm: false,
        }
    }
}

#[derive(Debug, Clone)]
struct RenmapApp {
    interface: AppInterface,
    show_error_message: bool,
    error_message: String,
    status_message: String,
    scan_paras: ScanParas,
}

impl Default for RenmapApp {
    fn default() -> Self {
        Self {
            interface: AppInterface::default(),
            show_error_message: false,
            error_message: "".to_owned(),
            status_message: "ready".to_owned(),
            scan_paras: ScanParas::default(),
        }
    }
}

impl App for RenmapApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        TopBottomPanel::top("top_menu_bar").show(ctx, |ui| {
            let menu_bar = containers::menu::Bar::default();
            menu_bar.ui(ui, |ui| {
                ui.menu_button("Utils", |ui| {
                    if ui
                        .button("Scan")
                        .on_hover_text("Scan a port you want to scan")
                        .clicked()
                    {
                        self.interface = AppInterface::Scan;
                    }
                    ui.separator();
                    if ui
                        .button("About")
                        .on_hover_text("Some information about this software")
                        .clicked()
                    {
                        self.interface = AppInterface::About;
                    }
                });
                match &mut self.interface {
                    AppInterface::Scan=> {
                        ui.menu_button("Options", |ui: &mut egui::Ui| {
                            if ui
                                .selectable_label(self.scan_paras.in_memory, "Memory DB Mode")
                                .on_hover_text("Only save the results in memory (means that all previous results will be lost when you reopen the software)")
                                .clicked()
                            {
                                self.scan_paras.in_memory = !self.scan_paras.in_memory;
                                self.scan_paras.in_memroy_confirm = true;
                            }
                        });
                    }
                    AppInterface::About => (),
                }
            });
        });

        match &mut self.interface {
            AppInterface::Scan => {
                if self.scan_paras.in_memory && self.scan_paras.in_memroy_confirm {
                    Window::new("Confirm")
                        .collapsible(false)
                        .resizable(false)
                        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                        .show(ctx, |ui| {
                            ui.label("Modify mode will delete the previously stored local content");
                            ui.horizontal(|ui| {
                                if ui.button("Yes").clicked() {
                                    self.scan_paras.in_memroy_confirm = false;
                                    match SqliteDB::drop_all() {
                                        Ok(_) => (),
                                        Err(e) => {
                                            self.error_message = e.to_string();
                                            self.show_error_message = true;
                                        }
                                    }
                                }
                                if ui.button("No").clicked() {
                                    self.scan_paras.in_memroy_confirm = false;
                                    self.scan_paras.in_memory = false;
                                }
                            });
                        });
                }
            }
            AppInterface::About => (),
        }

        if self.show_error_message {
            Window::new("Error Message")
                .collapsible(false)
                .resizable(false)
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(&self.error_message);
                    ui.horizontal(|ui| {
                        if ui.button("Yes").clicked() {
                            self.show_error_message = false;
                        }
                    });
                });
        }

        match &self.interface {
            AppInterface::Scan => {
                self.scan_interface(ctx);
            }
            AppInterface::About => self.about_interface(ctx),
        }
        // egui::CentralPanel::default().show(ctx, |ui| {
        //     ui.text_edit_singleline(&mut self.error_message);
        // });
    }
}

impl RenmapApp {
    fn set_error_mssage(&mut self, e: &str) {
        self.show_error_message = true;
        self.error_message = e.to_string();
    }
    fn scan_interface(&mut self, ctx: &Context) {
        SidePanel::left("left_sidebar")
            .resizable(true)
            .default_width(160.0)
            .show(ctx, |ui| {
                ui.heading("History");
                ui.separator();
                ScrollArea::vertical().show(ui, |ui| match SCAN_HISTORYS.lock() {
                    Ok(his) => {
                        for h in his.iter() {
                            let item_name = h.stime.format("%Y-%m-%d %H:%M:%S").to_string();
                            if ui.button(item_name).clicked() {
                                match CURRENT_SCAN.lock() {
                                    Ok(mut v) => (*v) = Some(h.clone()),
                                    Err(e) => self.set_error_mssage(&e.to_string()),
                                }
                            }
                        }
                    }
                    Err(e) => self.set_error_mssage(&e.to_string()),
                })
            });

        CentralPanel::default().show(ctx, |ui| {
            ui.columns(2, |columns| {
                columns[0].horizontal(|ui| {
                    ui.label(RichText::new("Target").strong());
                    ui.add_sized(
                        ui.available_size(),
                        TextEdit::singleline(&mut self.scan_paras.target_addr),
                    );
                });
                columns[1].horizontal(|ui| {
                    ui.label(RichText::new("Port").strong());
                    ui.add_sized(
                        ui.available_size(),
                        TextEdit::singleline(&mut self.scan_paras.target_port),
                    );
                });
            });

            CollapsingHeader::new("Advanced Options")
                .default_open(false)
                .show(ui, |ui| {
                    ui.columns(3, |columns| {
                        columns[0].horizontal(|ui| {
                            ui.label("Exclued Target");
                            ui.add_sized(
                                ui.available_size(),
                                TextEdit::singleline(&mut self.scan_paras.exclued_target_addr),
                            );
                        });
                        columns[1].horizontal(|ui| {
                            ui.label("Exclued Port");
                            ui.add_sized(
                                ui.available_size(),
                                TextEdit::singleline(&mut self.scan_paras.exclued_target_port),
                            );
                        });
                        columns[2].horizontal(|ui| {
                            ui.label("Timeout(s)");
                            ui.add_sized(
                                ui.available_size(),
                                TextEdit::singleline(&mut self.scan_paras.timeout),
                            );
                        });
                    });
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.scan_paras.scan_options.fast_mode, "Fast Mode");
                        ui.checkbox(&mut self.scan_paras.scan_options.slow_mode, "Slow Mode");
                        ui.checkbox(&mut self.scan_paras.scan_options.no_ping, "No Ping");
                    });
                });

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Scan").clicked() {
                    // start scan
                    let ctx = ctx.clone();
                    let target_addr = self.scan_paras.target_addr.clone();
                    let target_port = self.scan_paras.target_port.clone();
                    let scan_options = self.scan_paras.scan_options.clone();
                    let (tx, rx) = mpsc::channel();
                    // create new thread to do scan job
                    self.status_message = "Scanning...".to_owned();
                    let start = Instant::now();
                    thread::spawn(move || {
                        match pistol_scan(&target_addr, &target_port, &scan_options) {
                            Ok(ret) => {
                                match SCAN_HISTORYS.lock() {
                                    Ok(mut v) => {
                                        v.push(ret.clone());
                                        tx.send("".to_owned())
                                            .expect("scan fake result send failed");
                                    }
                                    Err(e) => {
                                        tx.send(e.to_string()).expect("scan error send failed");
                                    }
                                }
                                match CURRENT_SCAN.lock() {
                                    Ok(mut r) => *r = Some(ret),
                                    Err(e) => {
                                        tx.send(e.to_string()).expect("scan error send failed");
                                    }
                                }
                            }
                            Err(e) => {
                                tx.send(e.to_string()).expect("scan error send failed");
                            }
                        }
                        ctx.request_repaint();
                    });
                    self.status_message = format!("Time: {:.2}s", start.elapsed().as_secs_f32());

                    match rx.recv() {
                        Ok(msg) => {
                            if msg.len() > 0 {
                                println!("{}", msg);
                                self.set_error_mssage(&msg);
                            }
                        }
                        Err(e) => self.set_error_mssage(&e.to_string()),
                    }
                };
                // ui.separator();
                if ui.button("Cancel").clicked() {
                    // cancel scan
                };

                ui.separator();
                ui.label(&self.status_message);
            });

            // Added a empty line here.
            ui.separator();

            match CURRENT_SCAN.lock() {
                Ok(v) => {
                    if let Some(show_ret) = (*v).clone() {
                        TableBuilder::new(ui)
                            .id_salt("scan_results_table")
                            .striped(true)
                            .resizable(true)
                            .cell_layout(Layout::centered_and_justified(Direction::BottomUp))
                            .column(Column::remainder())
                            .column(Column::remainder())
                            .column(Column::remainder())
                            .column(Column::remainder())
                            .header(20.0, |mut header| {
                                header.col(|ui| {
                                    ui.label(RichText::new("id").strong());
                                });
                                header.col(|ui| {
                                    ui.label(RichText::new("target").strong());
                                });
                                header.col(|ui| {
                                    ui.label(RichText::new("port").strong());
                                });
                                header.col(|ui| {
                                    ui.label(RichText::new("status").strong());
                                });
                            })
                            .body(|mut body| {
                                let mut i = 1;
                                for (ip, h) in &show_ret.scans {
                                    for (p, s) in h {
                                        body.row(18.0, |mut row| {
                                            row.col(|ui| {
                                                ui.label(format!("{}", i));
                                            });
                                            row.col(|ui| {
                                                ui.label(format!("{}", ip));
                                            });
                                            row.col(|ui| {
                                                ui.label(format!("{}", p));
                                            });
                                            let mut port_status = Vec::new();
                                            for ps in s {
                                                port_status.push(format!("{}", ps.status));
                                            }
                                            let port_status = port_status.join("|");
                                            row.col(|ui| {
                                                ui.label(format!("{}", port_status));
                                            });
                                        });
                                        i += 1;
                                    }
                                }
                            });
                    } else {
                        ui.label("No data");
                    }
                }
                Err(e) => self.set_error_mssage(&e.to_string()),
            }

            // ui.separator();

            // move history panel to left side
            // TopBottomPanel::bottom("bottom_bar")
            //     .resizable(true)
            //     .default_height(100.0)
            //     .show(ctx, |ui| {
            //         ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
            //             ui.label("history");
            //             ui.separator();
            //             ui.label("xxxxxxxx");
            //             ui.label("xxxxxxxx");
            //             ui.label("xxxxxxxx");
            //             ui.label("xxxxxxxx");
            //             ui.label("xxxxxxxx");
            //         });
            //     });
        });
    }
    fn about_interface(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            ui.label("This is free and opensource software.");
        });
    }
}
