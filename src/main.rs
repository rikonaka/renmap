use eframe::App;
use eframe::Frame;
use eframe::egui;
use eframe::egui::CentralPanel;
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
use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::Mutex;

mod db;
mod scan;

use db::SqliteDB;
use scan::pistol_scan;

// start from 1
static SCAN_RETS_ID: LazyLock<Mutex<u32>> = LazyLock::new(|| Mutex::new(1));

static SCAN_RETS: LazyLock<Mutex<HashMap<u32, TcpUdpScans>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static SQLITE_DB: LazyLock<Mutex<SqliteDB>> =
    LazyLock::new(|| Mutex::new(SqliteDB::init_db(false).expect("init sqlite db failed")));

fn main() -> eframe::Result {
    let icon_bytes = include_bytes!("../assets/corgi.png");
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

struct ScanMode {
    fast_mode: bool,
    slow_mode: bool,
    no_ping: bool,
}

impl Default for ScanMode {
    fn default() -> Self {
        ScanMode {
            fast_mode: false,
            slow_mode: false,
            no_ping: false,
        }
    }
}

enum AppInterface {
    Scan,
    About,
}

impl Default for AppInterface {
    fn default() -> Self {
        AppInterface::Scan
    }
}

struct AppMode {
    app_interface: AppInterface,
    in_memory: bool,
    in_memroy_confirm: bool,
    show_error_message: bool,
    error_message: String,
}

impl Default for AppMode {
    fn default() -> Self {
        AppMode {
            app_interface: AppInterface::default(),
            in_memory: false,
            in_memroy_confirm: false,
            show_error_message: false,
            error_message: "".to_owned(),
        }
    }
}

struct RenmapApp {
    target_addr: String,
    target_port: String,
    app_mode: AppMode,
    scan_mode: ScanMode,
}

impl Default for RenmapApp {
    fn default() -> Self {
        Self {
            target_addr: "192.168.7.1".to_owned(),
            target_port: "80".to_owned(),
            app_mode: AppMode::default(),
            scan_mode: ScanMode::default(),
        }
    }
}

impl RenmapApp {
    fn set_error_mssage(&mut self, e: &str) {
        self.app_mode.show_error_message = true;
        self.app_mode.error_message = e.to_string();
    }
}

impl App for RenmapApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            let menu_bar = containers::menu::Bar::default();
            menu_bar.ui(ui, |ui| {
                ui.menu_button("Scan Options", |ui| {
                    if ui
                        .selectable_label(self.app_mode.in_memory, "Memory Mode")
                        .on_hover_text("Only save the results in memory (this means that all previous results will be lost when you reopen the software)")
                        .clicked()
                    {
                        self.app_mode.in_memory = !self.app_mode.in_memory;
                        self.app_mode.in_memroy_confirm = true;
                    }
                    if ui
                        .selectable_label(self.scan_mode.no_ping, "test")
                        .clicked()
                    {
                        self.scan_mode.no_ping = !self.scan_mode.no_ping;
                        // ui.close();
                    }
                });
            });
        });

        SidePanel::left("left_sidebar")
            .resizable(true)
            .default_width(120.0)
            .show(ctx, |ui| {
                ui.heading("App Menu");
                ui.separator();
                if ui.button("Pistol Scan").clicked() {
                    self.app_mode.app_interface = AppInterface::Scan;
                }
                if ui.button("About").clicked() {
                    self.app_mode.app_interface = AppInterface::About;
                }
            });

        if self.app_mode.in_memory && self.app_mode.in_memroy_confirm {
            Window::new("Confirm")
                .collapsible(false)
                .resizable(false)
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Modify mode will delete the previously stored local content");
                    ui.horizontal(|ui| {
                        if ui.button("Yes").clicked() {
                            self.app_mode.in_memroy_confirm = false;
                            match SqliteDB::drop_all() {
                                Ok(_) => (),
                                Err(e) => self.set_error_mssage(&e.to_string()),
                            }
                        }
                        if ui.button("No").clicked() {
                            self.app_mode.in_memroy_confirm = false;
                            self.app_mode.in_memory = false;
                        }
                    });
                });
        }

        if self.app_mode.show_error_message {
            Window::new("Error Message")
                .collapsible(false)
                .resizable(false)
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(&self.app_mode.error_message);
                    ui.horizontal(|ui| {
                        if ui.button("Yes").clicked() {
                            self.app_mode.show_error_message = false;
                        }
                    });
                });
        }

        match self.app_mode.app_interface {
            AppInterface::Scan => {
                CentralPanel::default().show(ctx, |ui| {
                    ui.columns(2, |columns| {
                        columns[0].horizontal(|ui| {
                            ui.label(RichText::new("Target").strong());
                            ui.add_sized(
                                ui.available_size(),
                                TextEdit::singleline(&mut self.target_addr),
                            );
                        });
                        columns[1].horizontal(|ui| {
                            ui.label(RichText::new("Port(s)").strong());
                            ui.add_sized(
                                ui.available_size(),
                                TextEdit::singleline(&mut self.target_port),
                            );
                        });
                    });

                    CollapsingHeader::new("Advanced Options")
                        .default_open(false)
                        .show(ui, |ui| {
                            ui.columns(2, |columns| {
                                columns[0].horizontal(|ui| {
                                    ui.label("Exclued Target");
                                    ui.add_sized(
                                        ui.available_size(),
                                        TextEdit::singleline(&mut self.target_addr),
                                    );
                                });
                                columns[1].horizontal(|ui| {
                                    ui.label("Exclued Port(s)");
                                    ui.add_sized(
                                        ui.available_size(),
                                        TextEdit::singleline(&mut self.target_port),
                                    );
                                });
                            });
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut self.scan_mode.fast_mode, "Fast Mode");
                                ui.checkbox(&mut self.scan_mode.slow_mode, "Slow Mode");
                                ui.checkbox(&mut self.scan_mode.no_ping, "No Ping");
                            });
                        });

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("Scan").clicked() {
                            // start scan
                            match pistol_scan(&self.target_addr, &self.target_port, &self.scan_mode)
                            {
                                Ok(ret) => match SCAN_RETS_ID.lock() {
                                    Ok(mut id) => match SCAN_RETS.lock() {
                                        Ok(mut map) => {
                                            map.insert(*id, ret);
                                            *id += 1;
                                        }
                                        Err(e) => self.set_error_mssage(&e.to_string()),
                                    },
                                    Err(e) => self.set_error_mssage(&e.to_string()),
                                },
                                Err(e) => self.set_error_mssage(&e.to_string()),
                            }
                        };
                        // ui.separator();
                        if ui.button("Cancel").clicked() {
                            // cancel scan
                        };
                        ui.label(format!("Cost 6.66 s",));
                    });

                    // Added a empty line here.
                    ui.separator();

                    match SCAN_RETS.lock() {
                        Ok(map) => {
                            if map.len() > 0 {
                                TableBuilder::new(ui)
                                    .id_salt("scan_results_table")
                                    .striped(true)
                                    .resizable(true)
                                    .cell_layout(Layout::centered_and_justified(
                                        Direction::BottomUp,
                                    ))
                                    .column(Column::auto())
                                    .column(Column::auto())
                                    .column(Column::auto())
                                    .column(Column::auto())
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
                                        for (k, v) in map.iter() {
                                            // start from 1
                                            let mut i: u32 = 1;
                                            for (ip, hmap) in &v.scans {
                                                for (p, s) in hmap {
                                                    body.row(18.0, |mut row| {
                                                        row.col(|ui| {
                                                            ui.label(format!("{}-{}", k, i));
                                                        });
                                                        row.col(|ui| {
                                                            ui.label(format!("{}", ip));
                                                        });
                                                        row.col(|ui| {
                                                            ui.label(format!("{}", p));
                                                        });
                                                        let mut port_status = Vec::new();
                                                        for ps in s {
                                                            port_status
                                                                .push(format!("{}", ps.status));
                                                        }
                                                        let port_status = port_status.join("|");
                                                        row.col(|ui| {
                                                            ui.label(format!("{}", port_status));
                                                        });
                                                    });
                                                    i += 1;
                                                }
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
                });

                TopBottomPanel::bottom("bottom_bar")
                    .resizable(true)
                    .default_height(100.0)
                    .show(ctx, |ui| {
                        ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
                            ui.label("history");
                            ui.separator();
                            ui.label("xxxxxxxx");
                            ui.label("xxxxxxxx");
                            ui.label("xxxxxxxx");
                            ui.label("xxxxxxxx");
                            ui.label("xxxxxxxx");
                        });
                    });
            }
            AppInterface::About => {
                CentralPanel::default().show(ctx, |ui| {
                    ui.label("This is free and opensource software.");
                });
            }
        }
    }
}
