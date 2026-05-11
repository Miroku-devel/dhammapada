use adw::prelude::*;
use adw::{Application, Window, HeaderBar};
use gtk4::{Box, Button, Label, Orientation, ScrolledWindow, Scale, Adjustment, EventControllerScroll, EventControllerScrollFlags};
use gettextrs::*;
use serde::{Serialize, Deserialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::cell::Cell;
use std::cell::RefCell;
use std::time::Instant;

#[derive(Serialize, Deserialize, Clone)]
struct Config {
    font_size: f64,
    window_width: i32,
    window_height: i32,
    window_maximized: bool,
    last_chapter: Option<String>,
    scroll_pos: f64,
    sidebar_scroll_pos: f64,
    dark_mode: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            font_size: 12.0,
            window_width: 850,
            window_height: 650,
            window_maximized: false,
            last_chapter: None,
            scroll_pos: 0.0,
            sidebar_scroll_pos: 0.0,
            dark_mode: false,
        }
    }
}

fn get_config_path() -> PathBuf {
    let mut path = env::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".config");
    path.push("dhammapada");
    let _ = fs::create_dir_all(&path);
    path.push("config.toml");
    path
}

fn load_config() -> Config {
    fs::read_to_string(get_config_path())
        .and_then(|content| toml::from_str(&content).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)))
        .unwrap_or_default()
}

fn save_config(config: &Config) {
    if let Ok(toml) = toml::to_string(config) {
        let _ = fs::write(get_config_path(), toml);
    }
}

fn main() {
    let locale_path = if let Ok(appdir) = env::var("APPDIR") {
        PathBuf::from(appdir).join("usr/share/locale")
    } else {
        let local_path = env::current_dir().unwrap().join("locales");
        if local_path.exists() {
            local_path
        } else {
            PathBuf::from("/usr/share/locale")
        }
    };
    if let Ok(lang_env) = env::var("LANG") {
        let lang_code = lang_env.split('.').next().unwrap_or("").split('_').next().unwrap_or("");
        let mo_path = locale_path.join(lang_code).join("LC_MESSAGES/dhammapada.mo");

        if !lang_code.is_empty() && lang_code != "en" && !mo_path.exists() {
            unsafe {
                env::set_var("LANG", "en_US.UTF-8");
                env::set_var("LANGUAGE", "en_US.UTF-8");
                env::set_var("LC_ALL", "en_US.UTF-8");
            }
        }
    }
    glib::set_prgname(Some("dhammapada"));
    glib::set_application_name("Dhammapada");
    
    if setlocale(LocaleCategory::LcAll, "").is_none() {
       setlocale(LocaleCategory::LcAll, "en_US.UTF-8");
    }
    let _ = bindtextdomain("dhammapada", locale_path.to_str().unwrap());
    let _ = bind_textdomain_codeset("dhammapada", "UTF-8");
    let _ = textdomain("dhammapada");

    let app = Application::builder()
        .application_id("org.dhammapada.app")
        .build();
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    let config = load_config();
    let style_manager = adw::StyleManager::default();
    
    if config.dark_mode {
        style_manager.set_color_scheme(adw::ColorScheme::PreferDark);
    } else {
        style_manager.set_color_scheme(adw::ColorScheme::PreferLight);
    }
    
    let font_size = Rc::new(Cell::new((config.font_size * 1024.0) as u32));
    let current_chapter = Rc::new(RefCell::new(config.last_chapter.clone()));
    let main_box = Box::new(Orientation::Vertical, 0);
    let header_bar = HeaderBar::new();
    
    let theme_switch = gtk4::Switch::builder()
        .active(config.dark_mode)
        .valign(gtk4::Align::Center)
        .margin_end(10)
        .build();

    let theme_icon = gtk4::Image::builder()
    .icon_name("night-light-symbolic")
    .margin_start(10)
    .margin_end(5)
    .build();

    let theme_box = Box::new(Orientation::Horizontal, 0);
    theme_box.append(&theme_icon);
    theme_box.append(&theme_switch);
    header_bar.pack_start(&theme_box);
    
    // BUG: Dark/Light mode switching is currently experimental and often 
    // fails to work within an AppImage environment due to sandboxing/portal 
    // communication issues. Hiding the toggle for AppImage builds to avoid 
    // offering a broken feature to the user.
    if std::env::var("APPDIR").is_ok() {
        theme_box.hide();
    }
    
    theme_switch.connect_state_set(move |_, is_dark| {
        let sm = adw::StyleManager::default();
        if is_dark {
            sm.set_color_scheme(adw::ColorScheme::PreferDark);
        } else {
            sm.set_color_scheme(adw::ColorScheme::PreferLight);
        }
        glib::Propagation::Proceed
    });
    
    main_box.append(&header_bar);
    let content_box = Box::new(Orientation::Horizontal, 0);
    main_box.append(&content_box);
    
    let sidebar_container = Box::builder()
        .orientation(Orientation::Vertical)
        .width_request(250)
        .css_classes(["navigation-sidebar"])
        .build();
    let sidebar_title = Label::builder()
        .label(&gettext("chaps"))
        .margin_top(15).margin_bottom(15)
        .css_classes(["title-4"])
        .build();
    sidebar_container.append(&sidebar_title);
    let sidebar_buttons = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(2)
        .build();
    let sidebar_scroll = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .child(&sidebar_buttons)
        .vexpand(true)
        .build();
    sidebar_container.append(&sidebar_scroll);
    
    let font_label = Label::new(Some(&gettext("fontsize")));
    font_label.add_css_class("caption");
    let adj = Adjustment::new(config.font_size, 8.0, 32.0, 1.0, 1.0, 0.0);
    let scale = Scale::builder()
        .adjustment(&adj)
        .digits(0)
        .draw_value(true)
        .build();
    let controls_box = Box::builder()
        .orientation(Orientation::Vertical)
        .margin_top(15).margin_bottom(15).margin_start(15).margin_end(15)
        .spacing(5)
        .build();
    controls_box.append(&font_label);
    controls_box.append(&scale);
    sidebar_container.append(&controls_box);
    
    let content_container = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .valign(gtk4::Align::Start)
        .margin_top(20).margin_bottom(20).margin_start(20).margin_end(20)
        .build();
    let clamp = adw::Clamp::builder()
        .maximum_size(600)
        .tightening_threshold(400)
        .child(&content_container)
        .build();
    let scroll_content = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .child(&clamp)
        .hexpand(true)
        .build();

    let chapter_ends = vec![
        20, 32, 43, 59, 75, 89, 99, 115, 128, 145, 156, 166, 178, 
        196, 208, 220, 234, 255, 272, 289, 305, 319, 333, 359, 382, 423
    ];

    let mut chapters_data: Vec<(String, Vec<String>)> = Vec::new();
    let mut current_start = 1;

    for (i, &end) in chapter_ends.iter().enumerate() {
        let id = format!("chap{}", i + 1);
        let verses = (current_start..=end).map(|n| n.to_string()).collect();
        chapters_data.push((id, verses));
        current_start = end + 1;
    }

    let mut start_button: Option<Button> = None;
    let mut first_button: Option<Button> = None;
    let sidebar_btns_ref = Rc::new(RefCell::new(Vec::<Button>::new()));

    let scroll_to_bottom = Rc::new(Cell::new(false));

    for (i, (id, verses)) in chapters_data.into_iter().enumerate() {
    let chapter_number = i + 1;
    let label_with_number = format!("{}. {}", chapter_number, gettext(&id));

    let btn = Button::builder()
        .label(&label_with_number)
        .css_classes(["flat"])
        .halign(gtk4::Align::Fill)
        .build();

    if let Some(child) = btn.child() {
        if let Ok(label) = child.downcast::<Label>() {
            label.set_xalign(0.0);
        }
    }
        
        let container_ptr = content_container.clone();
        let scroll_ptr = scroll_content.clone();
        let current_font_size = font_size.clone();
        let chapter_tracker = current_chapter.clone();
        let verses_data = verses.clone();
        let chap_id = id.clone();
        let btns_list = sidebar_btns_ref.clone();
        let current_btn = btn.clone();
        let should_bottom = scroll_to_bottom.clone();

        btn.connect_clicked(move |_| {
            *chapter_tracker.borrow_mut() = Some(chap_id.clone());
            for b in btns_list.borrow().iter() {
                b.remove_css_class("suggested-action");
                b.add_css_class("flat");
            }
            current_btn.add_css_class("suggested-action");
            current_btn.remove_css_class("flat");
            
            while let Some(child) = container_ptr.first_child() {
                container_ptr.remove(&child);
            }
            
            for v_id in &verses_data {
                let verse_box = Box::new(Orientation::Vertical, 5);
                verse_box.set_margin_bottom(25);
                let verse_text = gettext(v_id);
                let text_label = Label::builder()
                    .justify(gtk4::Justification::Center)
                    .wrap(true)
                    .use_markup(true)
                    .selectable(true)
                    .build();
                text_label.set_markup(&format!(
                    "<span alpha='50%'>{}</span>\n\n<span size='{}'>{}</span>", 
                    v_id.trim(), current_font_size.get(), verse_text.trim()
                ));
                let copy_btn = Button::builder()
                    .icon_name("edit-copy-symbolic")
                    .halign(gtk4::Align::Center)
                    .css_classes(["flat"])
                    .build();
                let plain_text = format!("{}\n{}", v_id, verse_text);
                copy_btn.connect_clicked(move |b| { b.clipboard().set_text(&plain_text); });
                verse_box.append(&text_label);
                verse_box.append(&copy_btn);
                container_ptr.append(&verse_box);
            }

            let s_ptr = scroll_ptr.clone();
            let go_bottom = should_bottom.get();
            glib::idle_add_local_once(move || {
                let adj = s_ptr.vadjustment();
                if go_bottom {
                    adj.set_value(adj.upper() - adj.page_size());
                } else {
                    adj.set_value(0.0);
                }
            });
            should_bottom.set(false);
        });

        if first_button.is_none() { first_button = Some(btn.clone()); }
        if Some(id.clone()) == config.last_chapter { start_button = Some(btn.clone()); }
        
        sidebar_buttons.append(&btn);
        sidebar_btns_ref.borrow_mut().push(btn);
    }

    let scroll_controller = EventControllerScroll::new(EventControllerScrollFlags::VERTICAL);
    let btns_for_scroll = sidebar_btns_ref.clone();
    let adj_for_scroll = scroll_content.vadjustment();
    let bottom_trigger = scroll_to_bottom.clone();
    let last_change_time = Rc::new(RefCell::new(Instant::now()));

    scroll_controller.connect_scroll(move |_, _dx, dy| {
        let now = Instant::now();
        if now.duration_since(*last_change_time.borrow()).as_millis() < 300 {
            return glib::Propagation::Stop;
        }

        let adj = &adj_for_scroll;
        let value = adj.value();
        let upper = adj.upper();
        let page_size = adj.page_size();
        
        let buttons = btns_for_scroll.borrow();
        let current_index = buttons.iter().position(|b| b.has_css_class("suggested-action"));

        if let Some(idx) = current_index {
            if dy > 0.0 && value >= (upper - page_size - 1.0) {
                if idx + 1 < buttons.len() {
                    *last_change_time.borrow_mut() = now;
                    bottom_trigger.set(false); 
                    buttons[idx + 1].emit_clicked();
                    return glib::Propagation::Stop;
                }
            }
            else if dy < 0.0 && value <= 0.0 {
                if idx > 0 {
                    *last_change_time.borrow_mut() = now;
                    bottom_trigger.set(true); 
                    buttons[idx - 1].emit_clicked();
                    return glib::Propagation::Stop;
                }
            }
        }
        glib::Propagation::Proceed
    });
    scroll_content.add_controller(scroll_controller);

    let final_start_btn = start_button.or(first_button);
    if let Some(btn) = final_start_btn {
        btn.emit_clicked();
        let content_vadj = scroll_content.vadjustment();
        let sidebar_vadj = sidebar_scroll.vadjustment();
        let content_pos = config.scroll_pos;
        let sidebar_pos = config.sidebar_scroll_pos;
        glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
            content_vadj.set_value(content_pos);
            sidebar_vadj.set_value(sidebar_pos);
            glib::ControlFlow::Break
        });
    }

    let container_ptr = content_container.clone();
    let font_size_ptr = font_size.clone();
    scale.connect_value_changed(move |s| {
        let new_size = (s.value() * 1024.0) as u32;
        font_size_ptr.set(new_size);
        let mut next_child = container_ptr.first_child();
        while let Some(child) = next_child {
            if let Ok(v_box) = child.clone().downcast::<Box>() {
                if let Some(label_widget) = v_box.first_child() {
                    if let Ok(label) = label_widget.downcast::<Label>() {
                        let full_text = label.text();
                        let parts: Vec<&str> = full_text.split("\n").map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
                        if parts.len() >= 2 {
                            let v_id = parts[0];
                            let verse_content = parts[1..].join("\n");
                            label.set_markup(&format!("<span alpha='50%'>{}</span>\n\n<span size='{}'>{}</span>", v_id, new_size, verse_content));
                        }
                    }
                }
            }
            next_child = child.next_sibling();
        }
    });

    content_box.append(&sidebar_container);
    content_box.append(&scroll_content);
    
    let window = Window::builder()
        .application(app)
        .title(&gettext("title"))
        .default_width(config.window_width)
        .default_height(config.window_height)
        .maximized(config.window_maximized)
        .content(&main_box)
        .build();

    let font_scale = scale.clone();
    let chapter_to_save = current_chapter.clone();
    let scroll_to_save = scroll_content.clone();
    let sidebar_scroll_to_save = sidebar_scroll.clone();
    let theme_to_save = theme_switch.clone();

    window.connect_close_request(move |w| {
        let (width, height) = (w.width(), w.height());
        save_config(&Config {
            font_size: font_scale.value(),
            window_width: width,
            window_height: height,
            window_maximized: w.is_maximized(),
            last_chapter: chapter_to_save.borrow().clone(),
            scroll_pos: scroll_to_save.vadjustment().value(),
            sidebar_scroll_pos: sidebar_scroll_to_save.vadjustment().value(),
            dark_mode: theme_to_save.is_active(),
        });
        glib::Propagation::Proceed
    });
    window.present();
}