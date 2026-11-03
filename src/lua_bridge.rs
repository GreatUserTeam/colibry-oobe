use anyhow::{anyhow, Context, Result};
use mlua::{Lua, LuaSerdeExt, Value};
use std::path::Path;

use crate::config::Config;

/// Путь к Lua-скрипту по умолчанию; можно переопределить OOBE_LUA_SCRIPT.
pub fn script_path() -> String {
    std::env::var("OOBE_LUA_SCRIPT").unwrap_or_else(|_| "oobe.lua".to_string())
}

/// Передаёт собранную конфигурацию в Lua-скрипт. Скрипт должен определить
/// функцию `write_config(cfg)` — она получает таблицу с полями Config
/// и записывает oobe.config.json. Возвращает путь, который вернул скрипт.
pub fn write_config_via_lua(cfg: &Config) -> Result<String> {
    let script = script_path();
    let source = std::fs::read_to_string(&script)
        .with_context(|| format!("чтение Lua-скрипта {script}"))?;

    let lua = Lua::new();
    let globals = lua.globals();

    // Каталог скрипта — скрипт может использовать для относительных путей
    if let Some(dir) = Path::new(&script).parent() {
        let _ = globals.set("OOBE_SCRIPT_DIR", dir.to_string_lossy().to_string());
    }

    // Загружаем скрипт
    lua.load(&source)
        .set_name(&script)
        .exec()
        .map_err(|e| anyhow!("выполнение {script}: {e}"))?;

    // Передаём конфиг как таблицу Lua
    let cfg_value: Value = lua
        .to_value(cfg)
        .map_err(|e| anyhow!("сериализация конфигурации в Lua-значение: {e}"))?;
    globals
        .set("OOBE_CONFIG", cfg_value.clone())
        .map_err(|e| anyhow!("globals.set: {e}"))?;

    let write_config: mlua::Function = globals
        .get("write_config")
        .map_err(|_| anyhow!("в {script} нет функции write_config(cfg)"))?;

    let result: Value = write_config
        .call(cfg_value)
        .map_err(|e| anyhow!("вызов write_config(cfg) в {script}: {e}"))?;

    // Скрипт может вернуть путь к файлу; если ничего не вернул — считаем
    // что он записал oobe.config.json в текущий каталог
    let path = match result {
        Value::String(s) => s
            .to_str()
            .map_err(|e| anyhow!("некорректная строка результата: {e}"))?
            .to_string(),
        _ => "oobe.config.json".to_string(),
    };
    Ok(path)
}
