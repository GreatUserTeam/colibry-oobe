-- oobe.lua — записывает oobe.config.json по таблице cfg,
-- переданной из Rust. Ожидается функция write_config(cfg) -> path.

local function json_escape(s)
    s = s:gsub('\\', '\\\\')
    s = s:gsub('"', '\\"')
    s = s:gsub('\n', '\\n')
    s = s:gsub('\r', '\\r')
    s = s:gsub('\t', '\\t')
    return '"' .. s .. '"'
end

local function write_json(path, cfg)
    local f, err = io.open(path, "w")
    if not f then return nil, err end
    local lines = {
        "{",
        string.format('  "config_version": %s,', json_escape(cfg.config_version or "1.0")),
        string.format('  "language": %s,', json_escape(cfg.language or "")),
        string.format('  "keyboard_layout": %s,', json_escape(cfg.keyboard_layout or "")),
        string.format('  "timezone": %s,', json_escape(cfg.timezone or "")),
        string.format('  "user_name": %s,', json_escape(cfg.user_name or "")),
        string.format('  "user_login": %s,', json_escape(cfg.user_login or "")),
        string.format('  "user_password_hash": %s,', json_escape(cfg.user_password_hash or "")),
        string.format('  "hostname": %s,', json_escape(cfg.hostname or "")),
        string.format('  "device_password_hash": %s,', json_escape(cfg.device_password_hash or "")),
        string.format('  "wifi_ssid": %s,', json_escape(cfg.wifi_ssid or "")),
        string.format('  "wifi_password_encrypted": %s,', json_escape(cfg.wifi_password_encrypted or "")),
        string.format('  "wifi_key_salt": %s,', json_escape(cfg.wifi_key_salt or "")),
        string.format('  "dns_servers": %s,', json_escape(cfg.dns_servers or "")),
        string.format('  "http_proxy": %s,', json_escape(cfg.http_proxy or "")),
        string.format('  "https_proxy": %s,', json_escape(cfg.https_proxy or "")),
        string.format('  "no_proxy": %s,', json_escape(cfg.no_proxy or "")),
        string.format('  "configured_at": %d', tonumber(cfg.configured_at) or 0),
        "}",
    }
    f:write(table.concat(lines, "\n"), "\n")
    f:close()
    os.execute(string.format("chmod 600 %q 2>/dev/null", path))
    return path
end

-- Скрипт пытается записать в /etc (если есть права), затем рядом со скриптом.
function write_config(cfg)
    local paths = { "/etc/oobe.config.json", "oobe.config.json" }
    local last_err
    for _, p in ipairs(paths) do
        local ok, err = write_json(p, cfg)
        if ok then return ok end
        last_err = err
    end
    error("не удалось записать oobe.config.json: " .. tostring(last_err))
end
