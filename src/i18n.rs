//! Embedded UI strings.
//!
//! Every row of `LOCALES` holds the translations of one interface string in
//! the fixed column order English, Russian, German, French, Spanish. The row
//! index is the string key shared between this table and the GTK UI, the
//! column index is the language index from `geo::LANGUAGE_CODES`.
//!
//! Key index map:
//! 0 window_title      1 welcome_title      2 welcome_subtitle   3 welcome_note
//! 4 start             5 next               6 back               7 apply
//! 8 language_title    9 language_subtitle  10 timezone_title    11 timezone_subtitle
//! 12 user_title       13 user_subtitle     14 label_name        15 label_login
//! 16 label_password   17 label_password_confirm 18 ph_name      19 ph_login
//! 20 device_title     21 device_subtitle   22 label_hostname    23 ph_hostname
//! 24 label_device_password 25 label_device_password_confirm
//! 26 wifi_title       27 wifi_subtitle     28 label_ssid        29 ph_ssid
//! 30 label_wifi_password 31 wifi_note
//! 32 proxy_title      33 proxy_subtitle    34 label_http_proxy  35 label_https_proxy
//! 36 label_no_proxy   37 ph_http_proxy     38 ph_no_proxy
//! 39 err_mismatch     40 err_required      41 err_wifi          42 err_save
//! 43 locating         44 verifying        45 err_verify
//! 46 license_accept   47 err_license       48 license_title      49 license_subtitle
//! 50 err_wrong_creds  51 err_server
pub const LOCALES: &[[&str; 5]] = &[
    ["ColibryOOBE — Initial Setup", "ColibryOOBE — Первоначальная настройка", "ColibryOOBE — Ersteinrichtung", "ColibryOOBE — Configuration initiale", "ColibryOOBE — Configuración inicial"],
    ["Welcome", "Добро пожаловать", "Willkommen", "Bienvenue", "Bienvenido"],
    ["This wizard sets up the language, timezone, your ColibryID and the internet connection.", "Этот мастер настроит язык, часовой пояс, ваш ColibryID и подключение к интернету.", "Dieser Assistent richtet Sprache, Zeitzone, Ihre ColibryID und die Internetverbindung ein.", "Cet assistant configure la langue, le fuseau horaire, votre ColibryID et la connexion Internet.", "Este asistente configura el idioma, la zona horaria, su ColibryID y la conexión a internet."],
    ["Language and timezone are detected automatically from your GPS location.", "Язык и часовой пояс определяются автоматически по GPS-местоположению.", "Sprache und Zeitzone werden automatisch anhand des GPS-Standorts ermittelt.", "La langue et le fuseau horaire sont détectés automatiquement à partir de la position GPS.", "El idioma y la zona horaria se detectan automáticamente a partir de la ubicación GPS."],
    ["Start", "Начать", "Starten", "Démarrer", "Comenzar"],
    ["Next", "Далее", "Weiter", "Suivant", "Siguiente"],
    ["Back", "Назад", "Zurück", "Retour", "Atrás"],
    ["Apply", "Применить", "Übernehmen", "Appliquer", "Aplicar"],
    ["Language", "Язык", "Sprache", "Langue", "Idioma"],
    ["Select the interface language", "Выберите язык интерфейса", "Wählen Sie die Sprache der Oberfläche", "Choisissez la langue de l'interface", "Seleccione el idioma de la interfaz"],
    ["Timezone", "Часовой пояс", "Zeitzone", "Fuseau horaire", "Zona horaria"],
    ["The nearest region was preselected from your GPS location", "Ближайший регион предвыбран по вашему GPS-местоположению", "Die nächste Region wurde anhand Ihres GPS-Standorts vorausgewählt", "La région la plus proche a été présélectionnée selon votre position GPS", "La región más cercana se ha preseleccionado según su ubicación GPS"],
    ["ColibryID", "ColibryID", "ColibryID", "ColibryID", "ColibryID"],
    ["Sign in with your ColibryID", "Войдите с помощью ColibryID", "Mit Ihrer ColibryID anmelden", "Connectez-vous avec votre ColibryID", "Inicie sesión con su ColibryID"],
    ["ColibryID (email)", "ColibryID (электронная почта)", "ColibryID (E-Mail)", "ColibryID (e-mail)", "ColibryID (correo electrónico)"],
    ["Login", "Логин", "Anmeldename", "Identifiant", "Nombre de usuario"],
    ["Password", "Пароль", "Passwort", "Mot de passe", "Contraseña"],
    ["Repeat password", "Повторите пароль", "Passwort wiederholen", "Répétez le mot de passe", "Repita la contraseña"],
    ["you@example.com", "you@example.com", "you@example.com", "you@example.com", "you@example.com"],
    ["john", "ivan", "max", "jean", "juan"],
    ["Device", "Устройство", "Gerät", "Appareil", "Dispositivo"],
    ["Hostname and administrator password", "Имя устройства и пароль администратора", "Hostname und Administratorkennwort", "Nom d'hôte et mot de passe administrateur", "Nombre de host y contraseña de administrador"],
    ["Hostname", "Имя устройства", "Hostname", "Nom d'hôte", "Nombre de host"],
    ["colibry-pc", "colibry-pc", "colibry-pc", "colibry-pc", "colibry-pc"],
    ["Device (root) password", "Пароль устройства (root)", "Gerätepasswort (root)", "Mot de passe de l'appareil (root)", "Contraseña del dispositivo (root)"],
    ["Repeat device password", "Повторите пароль устройства", "Gerätepasswort wiederholen", "Répétez le mot de passe de l'appareil", "Repita la contraseña del dispositivo"],
    ["Wireless Network", "Беспроводная сеть", "Drahtloses Netzwerk", "Réseau sans fil", "Red inalámbrica"],
    ["An internet connection is required", "Требуется подключение к интернету", "Internetverbindung erforderlich", "Une connexion Internet est requise", "Se requiere conexión a Internet"],
    ["Network name (SSID)", "Имя сети (SSID)", "Netzwerkname (SSID)", "Nom du réseau (SSID)", "Nombre de red (SSID)"],
    ["Your Wi-Fi network name", "Имя вашей Wi-Fi сети", "Name Ihres WLAN-Netzwerks", "Nom de votre réseau Wi-Fi", "Nombre de su red Wi-Fi"],
    ["Network password", "Пароль сети", "Netzwerkpasswort", "Mot de passe du réseau", "Contraseña de la red"],
    ["The Wi-Fi password is encrypted with your password.", "Пароль Wi-Fi шифруется вашим паролем.", "Das WLAN-Passwort wird mit Ihrem Passwort verschlüsselt.", "Le mot de passe Wi-Fi est chiffré avec votre mot de passe.", "La contraseña de Wi-Fi se cifra con su contraseña."],
    ["Proxy", "Прокси", "Proxy", "Proxy", "Proxy"],
    ["Network proxy settings (optional)", "Настройки прокси (необязательно)", "Netzwerkproxy-Einstellungen (optional)", "Paramètres de proxy réseau (facultatifs)", "Configuración de proxy de red (opcional)"],
    ["HTTP proxy", "HTTP-прокси", "HTTP-Proxy", "Proxy HTTP", "Proxy HTTP"],
    ["HTTPS proxy", "HTTPS-прокси", "HTTPS-Proxy", "Proxy HTTPS", "Proxy HTTPS"],
    ["Proxy exceptions", "Исключения для прокси", "Proxy-Ausnahmen", "Exceptions de proxy", "Excepciones de proxy"],
    ["http://proxy:8080", "http://proxy:8080", "http://proxy:8080", "http://proxy:8080", "http://proxy:8080"],
    ["localhost, 127.0.0.1", "localhost, 127.0.0.1", "localhost, 127.0.0.1", "localhost, 127.0.0.1", "localhost, 127.0.0.1"],
    ["Passwords do not match", "Пароли не совпадают", "Passwörter stimmen nicht überein", "Les mots de passe ne correspondent pas", "Las contraseñas no coinciden"],
    ["Please fill in all required fields", "Заполните все обязательные поля", "Bitte füllen Sie alle Pflichtfelder aus", "Veuillez remplir tous les champs obligatoires", "Complete todos los campos obligatorios"],
    ["Enter the Wi-Fi password", "Введите пароль Wi-Fi", "Geben Sie das WLAN-Passwort ein", "Entrez le mot de passe Wi-Fi", "Introduzca la contraseña de Wi-Fi"],
    ["Could not save the configuration", "Не удалось сохранить конфигурацию", "Konfiguration konnte nicht gespeichert werden", "Impossible d'enregistrer la configuration", "No se pudo guardar la configuración"],
    ["Detecting your location", "Определяем ваше местоположение", "Ihr Standort wird ermittelt", "Détection de votre position", "Detectando su ubicación"],
    ["Connecting to ColibryID…", "Подключаемся к ColibryID…", "Verbindung mit ColibryID…", "Connexion à ColibryID…", "Conectando con ColibryID…"],
    ["Could not verify your ColibryID", "Не удалось проверить ваш ColibryID", "Ihre ColibryID konnte nicht verifiziert werden", "Impossible de vérifier votre ColibryID", "No se pudo verificar su ColibryID"],
    ["I accept the license terms", "Я принимаю условия лицензии", "Ich akzeptiere die Lizenzbedingungen", "J'accepte les termes de la licence", "Acepto los términos de la licencia"],
    ["Please accept the license to continue", "Примите условия лицензии, чтобы продолжить", "Bitte akzeptieren Sie die Lizenz, um fortzufahren", "Veuillez accepter la licence pour continuer", "Acepte la licencia para continuar"],
    ["License Agreement", "Лицензионное соглашение", "Lizenzvereinbarung", "Contrat de licence", "Acuerdo de licencia"],
    ["Please read and accept the license", "Прочитайте и примите лицензию", "Bitte lesen und akzeptieren Sie die Lizenz", "Veuillez lire et accepter la licence", "Lea y acepte la licencia"],
    ["Invalid email or password", "Неверная почта или пароль", "Ungültige E-Mail oder Passwort", "E-mail ou mot de passe incorrect", "Correo o contraseña incorrectos"],
    ["Cannot reach the ColibryID server", "Не удаётся связаться с сервером ColibryID", "Der ColibryID-Server ist nicht erreichbar", "Impossible de joindre le serveur ColibryID", "No se puede contactar con el servidor de ColibryID"],
];

/// Look up a string by key (row) and language (column).
pub fn t(lang: usize, key: usize) -> &'static str {
    LOCALES[key][lang]
}
