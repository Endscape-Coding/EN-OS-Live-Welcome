use std::process::Command;
use std::env::{self, set_var, var};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;
use serde::{Serialize, Deserialize};
use sysinfo::{Components, System};

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    lang: String,
    theme: String,
    lightmode: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            lang: system_lang(),
            theme: "default".to_string(),
            lightmode: false,
        }
    }
}

//Открытие программ

#[tauri::command]
fn startprog(mode: &str) -> String {
    let mut cmd = match mode {
        "calamares-offline" => Command::new("calamares"),
        "calamares-online" => {
            let mut c = Command::new("calamares");
            c.args(&["-c", "/etc/calamares/online/"]);
            c
        }
        _ => return "Что то тут не так..".to_string()
    };

    match cmd.spawn() {
        Ok(..) => "OK".to_string().to_string(),
        Err(e) => format!("Ошибка {e}").to_string()
    }
    // ЭТО НЕ ИИШКА
}

#[tauri::command]
fn startlink(link: String) {
    let mut cmd = Command::new("xdg-open");
    cmd.arg(&link);

    match cmd.spawn() {
        Ok(..) => println!("Открываю ссылку: {}", &link),
        Err(e) => println!("Ошибка {e}")
    }
}

//Работа с темами
#[tauri::command]
fn theme_path(path: String) -> Result<String, String> {
    match fs::read_to_string(path) {
        Ok(content) => {
            println!("Подгружаем css стили.. Полет нормальный");
            Ok(String::from(data::DATA))
        }
        Err(e) => {
            eprintln!("Ошибка чтения CSS: {}", e);
            Ok(String::from(data::DATA))
        }
    }
}

//Работа с языками
fn system_lang() -> String {
    let syslang = env::var("LANG").expect("Ошибка получения языка").to_string().chars().take(2).collect();
    syslang
}

#[tauri::command]
fn set_lang(lang: String) -> Result<Config, String> {
    let mut config = config_read()?;
    config.lang = lang;
    println!("Ставим язык: {}", config.lang);
    config_write(config)
}

#[tauri::command]
fn curr_lang() -> String {
    let config = config_read();
    config.unwrap().lang
}


//Работа с конфигами
#[tauri::command]
fn config_read() -> Result<Config, String> {
    let path = format!("{}/.config/enos_manager/settings.json",var("HOME").unwrap());
    let path2 = format!("{}/.config/enos_manager/",var("HOME").unwrap());
    let path = Path::new(&path);
    let path2 = Path::new(&path2);

    match path.exists() {
        true => {
            println!("Читаем конфиг");
            let file = File::open(path).map_err(|e| e.to_string())?;
            let reader = BufReader::new(file);
            let config: Config = serde_json::from_reader(reader)
            .map_err(|e| format!("Ошибка парсинга конфига: {}", e))?;
            Ok(config)
            }

        false => {
            println!("Ошибка чтения конфига, может он просто еще не создан?");
            fs::create_dir_all(path2).map_err(|e| e.to_string())?;

            let default_config = Config::default();
            let file = fs::File::create(path).map_err(|e| e.to_string())?;
            let mut writer = BufWriter::new(file);

            serde_json::to_writer_pretty(&mut writer, &default_config).map_err(|e| e.to_string())?;
            writer.flush().map_err(|e| e.to_string())?;

            Ok(default_config)
        }


    }
}

#[tauri::command]
fn config_write(config: Config) -> Result<Config, String> {
    println!("Запись в конфиг..");

    let path = format!("{}/.config/enos_manager/settings.json",var("HOME").unwrap());
    let path2 = format!("{}/.config/enos_manager/",var("HOME").unwrap());
    let path = Path::new(&path);
    let path2 = Path::new(&path2);

    fs::create_dir_all(path2).map_err(|e| format!("Ошибка создания директории для конфига: {}", e))?;

    let file = File::create(path)
    .map_err(|e| format!("Ошибка создания конфига: {}", e))?;
    let writer = BufWriter::new(file);

    serde_json::to_writer_pretty(writer, &config)
    .map_err(|e| format!("Ошибка записи конфига: {}", e))?;

    Ok(config)
}

#[tauri::command]
fn get_home_dir() -> String {
    env::var("HOME").unwrap_or_else(|_| "/home/user".to_string())
}


fn mbwayland() -> bool {
    let output = env::var("XDG_SESSION_TYPE").unwrap_or_default();

    match &*output {
        "wayland" => true,
        "x11" => false,
        _ => panic!("You dont have x11 or wayland"),
    }
}

fn check_memory(sys: &mut System) -> f64 {
    sys.refresh_memory();
    sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0
}

fn main() {
    println!("Надеюсь, что вы не ИИ-фоб, и понимаете, что вся эта отладка написана вручную, просто, чтобы вы могли понять, на каком месте программа застряла. Удачного просмотра!");
    let mut sys = System::new();
    let _components = Components::new_with_refreshed_list();
    sys.refresh_all();

    let memory_size = check_memory(&mut sys);
    if memory_size < 1.0 {
        println!("У вас мало оперативной памяти!")
    }
    match mbwayland() {
        true => {
            set_var("XDG_RUNTIME_DIR", "/tmp/runtime-root");
            set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
            println!("Работаем с вяленым");
        }
        false => println!("Хорг запущен, еще посидим")
    }

    tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![startprog, theme_path, config_read, config_write, get_home_dir, set_lang, curr_lang, startlink])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
