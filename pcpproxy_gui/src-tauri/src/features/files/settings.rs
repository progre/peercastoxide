use std::{io::ErrorKind, path::PathBuf};

use log::error;
use once_cell::sync::Lazy;
use tauri::{api::path, generate_context};
use tokio::fs::{create_dir, read_to_string, rename, write, OpenOptions};

use crate::core::{settings::Settings, utils::dialog::show_dialog};

static APP_DIR: Lazy<PathBuf> = Lazy::new(|| {
    let context = generate_context!();
    path::app_dir(context.config()).unwrap()
});

async fn rename_bak(base_path: &str) {
    let mut i = 0;
    let path = loop {
        let idx = if i == 0 { "".into() } else { format!(".{}", i) };
        let path = format!("{}{}.bak", base_path, idx);
        if OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
            .is_ok()
        {
            break path;
        }
        i += 1;
    };

    if let Err(err) = rename(APP_DIR.join("settings.json"), path).await {
        log::error!("err {}", err);
    }
}

pub async fn load_settings_and_show_dialog_if_error() -> Settings {
    let path = APP_DIR.join("settings.json");
    match read_to_string(&path).await {
        Err(err) => {
            if err.kind() != ErrorKind::NotFound {
                error!("{:?}", err);
                show_dialog(&format!(
                    "設定ファイルの読み込みに失敗しました。({:?})",
                    err
                ));
            }
            Settings::default()
        }
        Ok(str) => match deser_hjson::from_str::<Settings>(&str) {
            Err(err) => {
                error!("{:?}", err);
                show_dialog(&format!(
                    "設定ファイルが破損しています。({:?})\n設定をリセットします。",
                    err
                ));
                rename_bak(&path.to_string_lossy()).await;
                Settings::default()
            }
            Ok(settings) => settings,
        },
    }
}

pub async fn save_settings_and_show_dialog_if_error(settings: &Settings) {
    if let Err(err) = create_dir(APP_DIR.as_path()).await {
        if err.kind() != ErrorKind::AlreadyExists {
            panic!("{:?}", err);
        }
    }
    let opt = write(
        APP_DIR.join("settings.json"),
        serde_json::to_string_pretty(settings).unwrap(),
    )
    .await;
    if let Err(err) = opt {
        error!("{:?}", err);
        show_dialog(&format!("設定ファイルの保存に失敗しました。({:?})", err));
    }
}
