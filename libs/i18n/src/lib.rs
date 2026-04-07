use std::collections::HashMap;
use std::sync::OnceLock;

static LANG: OnceLock<String> = OnceLock::new();
static STRINGS: OnceLock<&'static HashMap<&'static str, &'static str>> = OnceLock::new();

/// Initialisiert die Sprache aus der Windows-Systemsprache.
/// Fallback: "en"
pub fn init() {
    let lang = detect_language();
    LANG.get_or_init(|| lang.clone());
    STRINGS.get_or_init(|| get_strings_for(&lang));
}

/// Übersetzt einen Key. Gibt den Key selbst zurück wenn keine Übersetzung existiert.
pub fn t(key: &str) -> &'static str {
    STRINGS
        .get()
        .and_then(|m| m.get(key).copied())
        .or_else(|| EN.get(key).copied())
        .unwrap_or("???")
}

/// Gibt die aktuelle Sprache zurück (z.B. "de", "en", "fr")
pub fn current_lang() -> &'static str {
    LANG.get().map(|s| s.as_str()).unwrap_or("en")
}

fn detect_language() -> String {
    #[cfg(windows)]
    {
        // GetUserDefaultUILanguage → LANGID → Primary Language
        extern "system" {
            fn GetUserDefaultUILanguage() -> u16;
        }
        let langid = unsafe { GetUserDefaultUILanguage() };
        let primary = langid & 0x3FF; // Primary language ID
        match primary {
            0x07 => return "de".into(), // German
            0x0C => return "fr".into(), // French
            0x10 => return "it".into(), // Italian
            0x0A => return "es".into(), // Spanish
            0x13 => return "nl".into(), // Dutch
            0x16 => return "pt".into(), // Portuguese
            0x15 => return "pl".into(), // Polish
            0x05 => return "cs".into(), // Czech
            0x0E => return "hu".into(), // Hungarian
            0x1B => return "sk".into(), // Slovak
            0x24 => return "sl".into(), // Slovenian
            0x18 => return "ro".into(), // Romanian
            0x02 => return "bg".into(), // Bulgarian
            0x1A => return "hr".into(), // Croatian
            0x25 => return "et".into(), // Estonian
            0x26 => return "lv".into(), // Latvian
            0x27 => return "lt".into(), // Lithuanian
            0x0B => return "fi".into(), // Finnish
            0x1D => return "sv".into(), // Swedish
            0x14 => return "no".into(), // Norwegian
            0x06 => return "da".into(), // Danish
            0x08 => return "el".into(), // Greek
            0x09 => return "en".into(), // English
            _ => {}
        }
    }
    "en".into()
}

fn get_strings_for(lang: &str) -> &'static HashMap<&'static str, &'static str> {
    match lang {
        "de" => &DE,
        "fr" => &FR,
        "it" => &IT,
        "es" => &ES,
        "nl" => &NL,
        "pt" => &PT,
        "pl" => &PL,
        "cs" => &CS,
        "hu" => &HU,
        "sk" => &SK,
        "sl" => &SL,
        "ro" => &RO,
        "bg" => &BG,
        "hr" => &HR,
        "et" => &ET,
        "lv" => &LV,
        "lt" => &LT,
        "fi" => &FI,
        "sv" => &SV,
        "no" => &NO,
        "da" => &DA,
        "el" => &EL,
        _ => &EN,
    }
}

macro_rules! strings {
    ($name:ident, $($key:expr => $val:expr),+ $(,)?) => {
        static $name: std::sync::LazyLock<HashMap<&'static str, &'static str>> =
            std::sync::LazyLock::new(|| HashMap::from([$(($key, $val)),+]));
    };
}

// ─── Keys ───────────────────────────────────────────────
// Tray
// tray.show, tray.reload_config, tray.register_tel, tray.devtools, tray.quit
// Callback Popup
// popup.send_mail, popup.close
// Mailto
// mailto.subject  (template: {name} {number} will be replaced)
// Call History Bulk Delete
// hist.select_multiple, hist.all, hist.selected_count, hist.delete_selected,
// hist.cancel, hist.confirm_delete, hist.preparing, hist.deleting, hist.deleted
// Setup Wizard
// setup.title, setup.prompt, setup.placeholder, setup.error_empty, setup.error_url, setup.save_error, setup.button

strings!(EN,
    "tray.show" => "Show",
    "tray.reload_config" => "Reload config",
    "tray.register_tel" => "Register as tel: handler",
    "tray.devtools" => "DevTools (F12)",
    "tray.open_config" => "Open configuration",
    "tray.quit" => "Quit",
    "popup.send_mail" => "\u{2709}  Send callback mail",
    "mailto.subject" => "Please call back: {name} {number}",
    "hist.select_multiple" => "Select multiple",
    "hist.all" => "All",
    "hist.selected_count" => "selected",
    "hist.delete_selected" => "Delete selected",
    "hist.cancel" => "Cancel",
    "hist.confirm_delete" => "Do you want to delete {count} entries?",
    "hist.preparing" => "Preparing...",
    "hist.deleting" => "Deleting",
    "hist.deleted" => "deleted!",
    "setup.title" => "Set up 4dy Client",
    "setup.prompt" => "Enter the URL of your 3CX Web Client:",
    "setup.placeholder" => "https://my-company.3cx.com",
    "setup.error_empty" => "Please enter a URL.",
    "setup.error_url" => "Please enter a valid URL (e.g. https://my-company.3cx.com)",
    "setup.save_error" => "Error saving:",
    "setup.button" => "Continue",
    "setup.saving" => "Saving...",
    "setup.restart_msg" => "Configuration saved successfully.\nPlease restart the application.",
    "setup.restart_title" => "4dy Client",
);

strings!(DE,
    "tray.show" => "Anzeigen",
    "tray.reload_config" => "Config neu laden",
    "tray.register_tel" => "Als tel: Handler registrieren",
    "tray.devtools" => "DevTools (F12)",
    "tray.open_config" => "Konfiguration öffnen",
    "tray.quit" => "Beenden",
    "popup.send_mail" => "\u{2709}  Rückruf-Mail senden",
    "mailto.subject" => "Bitte zurückrufen: {name} {number}",
    "hist.select_multiple" => "Mehrere auswählen",
    "hist.all" => "Alle",
    "hist.selected_count" => "ausgewählt",
    "hist.delete_selected" => "Ausgewählte löschen",
    "hist.cancel" => "Abbrechen",
    "hist.confirm_delete" => "Möchtest du {count} Einträge löschen?",
    "hist.preparing" => "Vorbereiten...",
    "hist.deleting" => "Lösche",
    "hist.deleted" => "gelöscht!",
    "setup.title" => "4dy Client einrichten",
    "setup.prompt" => "Bitte gib die URL deines 3CX Web-Clients ein:",
    "setup.placeholder" => "https://meine-firma.3cx.de",
    "setup.error_empty" => "Bitte eine URL eingeben.",
    "setup.error_url" => "Bitte eine gültige URL eingeben (z.B. https://meine-firma.3cx.de)",
    "setup.save_error" => "Fehler beim Speichern:",
    "setup.button" => "Weiter",
    "setup.saving" => "Speichere...",
    "setup.restart_msg" => "Konfiguration gespeichert.\nBitte starte die Anwendung neu.",
    "setup.restart_title" => "4dy Client",
);

strings!(FR,
    "tray.show" => "Afficher",
    "tray.reload_config" => "Recharger la config",
    "tray.register_tel" => "Enregistrer comme gestionnaire tel:",
    "tray.devtools" => "DevTools (F12)",
    "tray.open_config" => "Ouvrir la configuration",
    "tray.quit" => "Quitter",
    "popup.send_mail" => "\u{2709}  Envoyer un mail de rappel",
    "mailto.subject" => "Merci de rappeler : {name} {number}",
    "hist.select_multiple" => "Sélection multiple",
    "hist.all" => "Tous",
    "hist.selected_count" => "sélectionné(s)",
    "hist.delete_selected" => "Supprimer la sélection",
    "hist.cancel" => "Annuler",
    "hist.confirm_delete" => "Voulez-vous supprimer {count} entrées ?",
    "hist.preparing" => "Préparation...",
    "hist.deleting" => "Suppression",
    "hist.deleted" => "supprimé(s) !",
    "setup.title" => "Configurer 4dy Client",
    "setup.prompt" => "Entrez l'URL de votre client web 3CX :",
    "setup.placeholder" => "https://mon-entreprise.3cx.fr",
    "setup.error_empty" => "Veuillez entrer une URL.",
    "setup.error_url" => "Veuillez entrer une URL valide",
    "setup.save_error" => "Erreur lors de la sauvegarde :",
    "setup.button" => "Continuer",
    "setup.saving" => "Enregistrement...",
);

strings!(IT,
    "tray.show" => "Mostra", "tray.reload_config" => "Ricarica config", "tray.register_tel" => "Registra come gestore tel:",
    "tray.devtools" => "DevTools (F12)", "tray.open_config" => "Apri configurazione", "tray.quit" => "Esci",
    "popup.send_mail" => "\u{2709}  Invia mail di richiamata", "mailto.subject" => "Si prega di richiamare: {name} {number}",
    "hist.select_multiple" => "Selezione multipla", "hist.all" => "Tutti", "hist.selected_count" => "selezionato/i",
    "hist.delete_selected" => "Elimina selezionati", "hist.cancel" => "Annulla",
    "hist.confirm_delete" => "Vuoi eliminare {count} voci?", "hist.preparing" => "Preparazione...",
    "hist.deleting" => "Eliminazione", "hist.deleted" => "eliminato/i!",
    "setup.title" => "Configura 4dy Client", "setup.prompt" => "Inserisci l'URL del tuo client web 3CX:",
    "setup.placeholder" => "https://mia-azienda.3cx.it", "setup.error_empty" => "Inserisci un URL.",
    "setup.error_url" => "Inserisci un URL valido", "setup.save_error" => "Errore nel salvataggio:",
    "setup.button" => "Continua", "setup.saving" => "Salvataggio...",
);

strings!(ES,
    "tray.show" => "Mostrar", "tray.reload_config" => "Recargar config", "tray.register_tel" => "Registrar como gestor tel:",
    "tray.devtools" => "DevTools (F12)", "tray.open_config" => "Abrir configuración", "tray.quit" => "Salir",
    "popup.send_mail" => "\u{2709}  Enviar correo de devolución", "mailto.subject" => "Por favor devolver llamada: {name} {number}",
    "hist.select_multiple" => "Selección múltiple", "hist.all" => "Todos", "hist.selected_count" => "seleccionado(s)",
    "hist.delete_selected" => "Eliminar seleccionados", "hist.cancel" => "Cancelar",
    "hist.confirm_delete" => "¿Desea eliminar {count} entradas?", "hist.preparing" => "Preparando...",
    "hist.deleting" => "Eliminando", "hist.deleted" => "eliminado(s)!",
    "setup.title" => "Configurar 4dy Client", "setup.prompt" => "Introduce la URL de tu cliente web 3CX:",
    "setup.placeholder" => "https://mi-empresa.3cx.es", "setup.error_empty" => "Introduce una URL.",
    "setup.error_url" => "Introduce una URL válida", "setup.save_error" => "Error al guardar:",
    "setup.button" => "Continuar", "setup.saving" => "Guardando...",
);

strings!(NL,
    "tray.show" => "Tonen", "tray.reload_config" => "Config herladen", "tray.register_tel" => "Registreren als tel: handler",
    "tray.devtools" => "DevTools (F12)", "tray.open_config" => "Configuratie openen", "tray.quit" => "Afsluiten",
    "popup.send_mail" => "\u{2709}  Terugbelmail verzenden", "mailto.subject" => "Graag terugbellen: {name} {number}",
    "hist.select_multiple" => "Meerdere selecteren", "hist.all" => "Alle", "hist.selected_count" => "geselecteerd",
    "hist.delete_selected" => "Selectie verwijderen", "hist.cancel" => "Annuleren",
    "hist.confirm_delete" => "Wilt u {count} items verwijderen?", "hist.preparing" => "Voorbereiden...",
    "hist.deleting" => "Verwijderen", "hist.deleted" => "verwijderd!",
    "setup.title" => "4dy Client instellen", "setup.prompt" => "Voer de URL van uw 3CX-webclient in:",
    "setup.placeholder" => "https://mijn-bedrijf.3cx.nl", "setup.error_empty" => "Voer een URL in.",
    "setup.error_url" => "Voer een geldige URL in", "setup.save_error" => "Fout bij opslaan:",
    "setup.button" => "Doorgaan", "setup.saving" => "Opslaan...",
);

strings!(PT,
    "tray.show" => "Mostrar", "tray.reload_config" => "Recarregar config", "tray.register_tel" => "Registar como gestor tel:",
    "tray.devtools" => "DevTools (F12)", "tray.open_config" => "Abrir configuração", "tray.quit" => "Sair",
    "popup.send_mail" => "\u{2709}  Enviar email de retorno", "mailto.subject" => "Por favor retornar: {name} {number}",
    "hist.select_multiple" => "Selecionar múltiplos", "hist.all" => "Todos", "hist.selected_count" => "selecionado(s)",
    "hist.delete_selected" => "Eliminar selecionados", "hist.cancel" => "Cancelar",
    "hist.confirm_delete" => "Deseja eliminar {count} entradas?", "hist.preparing" => "A preparar...",
    "hist.deleting" => "A eliminar", "hist.deleted" => "eliminado(s)!",
    "setup.title" => "Configurar 4dy Client", "setup.prompt" => "Introduza o URL do seu cliente web 3CX:",
    "setup.placeholder" => "https://minha-empresa.3cx.pt", "setup.error_empty" => "Introduza um URL.",
    "setup.error_url" => "Introduza um URL válido", "setup.save_error" => "Erro ao guardar:",
    "setup.button" => "Continuar", "setup.saving" => "A guardar...",
);

strings!(PL,
    "tray.show" => "Pokaż", "tray.reload_config" => "Przeładuj konfigurację", "tray.register_tel" => "Zarejestruj jako obsługę tel:",
    "tray.devtools" => "DevTools (F12)", "tray.open_config" => "Otwórz konfigurację", "tray.quit" => "Zakończ",
    "popup.send_mail" => "\u{2709}  Wyślij mail zwrotny", "mailto.subject" => "Proszę oddzwonić: {name} {number}",
    "hist.select_multiple" => "Zaznacz wiele", "hist.all" => "Wszystkie", "hist.selected_count" => "zaznaczono",
    "hist.delete_selected" => "Usuń zaznaczone", "hist.cancel" => "Anuluj",
    "hist.confirm_delete" => "Czy chcesz usunąć {count} wpisów?", "hist.preparing" => "Przygotowywanie...",
    "hist.deleting" => "Usuwanie", "hist.deleted" => "usunięto!",
    "setup.title" => "Konfiguracja 4dy Client", "setup.prompt" => "Podaj URL klienta webowego 3CX:",
    "setup.placeholder" => "https://moja-firma.3cx.pl", "setup.error_empty" => "Podaj URL.",
    "setup.error_url" => "Podaj prawidłowy URL", "setup.save_error" => "Błąd zapisu:",
    "setup.button" => "Dalej", "setup.saving" => "Zapisywanie...",
);

// Remaining languages - all follow same pattern
strings!(CS, "tray.show"=>"Zobrazit","tray.reload_config"=>"Znovu načíst konfiguraci","tray.register_tel"=>"Zaregistrovat jako tel: handler","tray.devtools"=>"DevTools (F12)","tray.open_config"=>"Otevřít konfiguraci","tray.quit"=>"Ukončit","popup.send_mail"=>"\u{2709}  Odeslat zpětný e-mail","mailto.subject"=>"Prosím zavolejte zpět: {name} {number}","hist.select_multiple"=>"Vybrat více","hist.all"=>"Vše","hist.selected_count"=>"vybráno","hist.delete_selected"=>"Smazat vybrané","hist.cancel"=>"Zrušit","hist.confirm_delete"=>"Chcete smazat {count} položek?","hist.preparing"=>"Příprava...","hist.deleting"=>"Mazání","hist.deleted"=>"smazáno!","setup.title"=>"Nastavení 4dy Client","setup.prompt"=>"Zadejte URL vašeho 3CX webového klienta:","setup.placeholder"=>"https://moje-firma.3cx.cz","setup.error_empty"=>"Zadejte URL.","setup.error_url"=>"Zadejte platnou URL","setup.save_error"=>"Chyba při ukládání:","setup.button"=>"Pokračovat","setup.saving"=>"Ukládání...");
strings!(HU, "tray.show"=>"Megjelenítés","tray.reload_config"=>"Konfiguráció újratöltése","tray.register_tel"=>"Regisztráció tel: kezelőként","tray.devtools"=>"DevTools (F12)","tray.open_config"=>"Konfiguráció megnyitása","tray.quit"=>"Kilépés","popup.send_mail"=>"\u{2709}  Visszahívás e-mail küldése","mailto.subject"=>"Kérjük hívjon vissza: {name} {number}","hist.select_multiple"=>"Több kijelölése","hist.all"=>"Összes","hist.selected_count"=>"kijelölve","hist.delete_selected"=>"Kijelöltek törlése","hist.cancel"=>"Mégse","hist.confirm_delete"=>"{count} bejegyzés törlése?","hist.preparing"=>"Előkészítés...","hist.deleting"=>"Törlés","hist.deleted"=>"törölve!","setup.title"=>"4dy Client beállítása","setup.prompt"=>"Adja meg a 3CX webes kliens URL-jét:","setup.placeholder"=>"https://cegem.3cx.hu","setup.error_empty"=>"Adjon meg egy URL-t.","setup.error_url"=>"Érvényes URL-t adjon meg","setup.save_error"=>"Mentési hiba:","setup.button"=>"Tovább","setup.saving"=>"Mentés...");
strings!(SK, "tray.show"=>"Zobraziť","tray.reload_config"=>"Znovu načítať konfiguráciu","tray.register_tel"=>"Zaregistrovať ako tel: handler","tray.devtools"=>"DevTools (F12)","tray.open_config"=>"Otvoriť konfiguráciu","tray.quit"=>"Ukončiť","popup.send_mail"=>"\u{2709}  Odoslať spätný e-mail","mailto.subject"=>"Prosím zavolajte späť: {name} {number}","hist.select_multiple"=>"Vybrať viacero","hist.all"=>"Všetko","hist.selected_count"=>"vybraných","hist.delete_selected"=>"Zmazať vybrané","hist.cancel"=>"Zrušiť","hist.confirm_delete"=>"Chcete zmazať {count} položiek?","hist.preparing"=>"Príprava...","hist.deleting"=>"Mazanie","hist.deleted"=>"zmazaných!","setup.title"=>"Nastavenie 4dy Client","setup.prompt"=>"Zadajte URL vášho 3CX webového klienta:","setup.placeholder"=>"https://moja-firma.3cx.sk","setup.error_empty"=>"Zadajte URL.","setup.error_url"=>"Zadajte platnú URL","setup.save_error"=>"Chyba pri ukladaní:","setup.button"=>"Pokračovať","setup.saving"=>"Ukladanie...");
strings!(SL, "tray.show"=>"Prikaži","tray.reload_config"=>"Ponovno naloži konfiguracijo","tray.register_tel"=>"Registriraj kot tel: handler","tray.devtools"=>"DevTools (F12)","tray.open_config"=>"Odpri konfiguracijo","tray.quit"=>"Izhod","popup.send_mail"=>"\u{2709}  Pošlji povratno e-pošto","mailto.subject"=>"Prosimo pokličite nazaj: {name} {number}","hist.select_multiple"=>"Izberi več","hist.all"=>"Vse","hist.selected_count"=>"izbranih","hist.delete_selected"=>"Izbriši izbrane","hist.cancel"=>"Prekliči","hist.confirm_delete"=>"Ali želite izbrisati {count} vnosov?","hist.preparing"=>"Priprava...","hist.deleting"=>"Brisanje","hist.deleted"=>"izbrisanih!","setup.title"=>"Nastavitev 4dy Client","setup.prompt"=>"Vnesite URL vašega 3CX spletnega odjemalca:","setup.placeholder"=>"https://moje-podjetje.3cx.si","setup.error_empty"=>"Vnesite URL.","setup.error_url"=>"Vnesite veljavni URL","setup.save_error"=>"Napaka pri shranjevanju:","setup.button"=>"Nadaljuj","setup.saving"=>"Shranjevanje...");
strings!(RO, "tray.show"=>"Afișare","tray.reload_config"=>"Reîncarcă config","tray.register_tel"=>"Înregistrează ca handler tel:","tray.devtools"=>"DevTools (F12)","tray.open_config"=>"Deschide configurația","tray.quit"=>"Ieșire","popup.send_mail"=>"\u{2709}  Trimite email de reapelare","mailto.subject"=>"Vă rugăm sunați înapoi: {name} {number}","hist.select_multiple"=>"Selectare multiplă","hist.all"=>"Toate","hist.selected_count"=>"selectate","hist.delete_selected"=>"Șterge selectate","hist.cancel"=>"Anulare","hist.confirm_delete"=>"Doriți să ștergeți {count} intrări?","hist.preparing"=>"Pregătire...","hist.deleting"=>"Ștergere","hist.deleted"=>"șterse!","setup.title"=>"Configurare 4dy Client","setup.prompt"=>"Introduceți URL-ul clientului web 3CX:","setup.placeholder"=>"https://firma-mea.3cx.ro","setup.error_empty"=>"Introduceți un URL.","setup.error_url"=>"Introduceți un URL valid","setup.save_error"=>"Eroare la salvare:","setup.button"=>"Continuare","setup.saving"=>"Salvare...");
strings!(BG, "tray.show"=>"Покажи","tray.reload_config"=>"Презареди конфигурацията","tray.register_tel"=>"Регистрирай като tel: handler","tray.devtools"=>"DevTools (F12)","tray.open_config"=>"Отвори конфигурацията","tray.quit"=>"Изход","popup.send_mail"=>"\u{2709}  Изпрати имейл за обратно обаждане","mailto.subject"=>"Моля обадете се обратно: {name} {number}","hist.select_multiple"=>"Избери няколко","hist.all"=>"Всички","hist.selected_count"=>"избрани","hist.delete_selected"=>"Изтрий избраните","hist.cancel"=>"Отказ","hist.confirm_delete"=>"Искате ли да изтриете {count} записа?","hist.preparing"=>"Подготовка...","hist.deleting"=>"Изтриване","hist.deleted"=>"изтрити!","setup.title"=>"Настройка на 4dy Client","setup.prompt"=>"Въведете URL на вашия 3CX уеб клиент:","setup.placeholder"=>"https://moiata-firma.3cx.bg","setup.error_empty"=>"Въведете URL.","setup.error_url"=>"Въведете валиден URL","setup.save_error"=>"Грешка при запазване:","setup.button"=>"Продължи","setup.saving"=>"Запазване...");
strings!(HR, "tray.show"=>"Prikaži","tray.reload_config"=>"Ponovno učitaj konfiguraciju","tray.register_tel"=>"Registriraj kao tel: handler","tray.devtools"=>"DevTools (F12)","tray.open_config"=>"Otvori konfiguraciju","tray.quit"=>"Izlaz","popup.send_mail"=>"\u{2709}  Pošalji povratni e-mail","mailto.subject"=>"Molimo nazovite: {name} {number}","hist.select_multiple"=>"Odaberi više","hist.all"=>"Sve","hist.selected_count"=>"odabrano","hist.delete_selected"=>"Obriši odabrane","hist.cancel"=>"Odustani","hist.confirm_delete"=>"Želite li obrisati {count} unosa?","hist.preparing"=>"Priprema...","hist.deleting"=>"Brisanje","hist.deleted"=>"obrisano!","setup.title"=>"Postavljanje 4dy Client","setup.prompt"=>"Unesite URL svog 3CX web klijenta:","setup.placeholder"=>"https://moja-tvrtka.3cx.hr","setup.error_empty"=>"Unesite URL.","setup.error_url"=>"Unesite valjani URL","setup.save_error"=>"Greška pri spremanju:","setup.button"=>"Nastavi","setup.saving"=>"Spremanje...");
strings!(ET, "tray.show"=>"Kuva","tray.reload_config"=>"Laadi konfiguratsioon uuesti","tray.register_tel"=>"Registreeri tel: käitlejana","tray.devtools"=>"DevTools (F12)","tray.open_config"=>"Ava konfiguratsioon","tray.quit"=>"Välju","popup.send_mail"=>"\u{2709}  Saada tagasihelistamise e-kiri","mailto.subject"=>"Palun helistage tagasi: {name} {number}","hist.select_multiple"=>"Vali mitu","hist.all"=>"Kõik","hist.selected_count"=>"valitud","hist.delete_selected"=>"Kustuta valitud","hist.cancel"=>"Tühista","hist.confirm_delete"=>"Kas soovite kustutada {count} kirjet?","hist.preparing"=>"Ettevalmistamine...","hist.deleting"=>"Kustutamine","hist.deleted"=>"kustutatud!","setup.title"=>"4dy Client seadistamine","setup.prompt"=>"Sisestage oma 3CX veebikliendi URL:","setup.placeholder"=>"https://minu-ettevote.3cx.ee","setup.error_empty"=>"Sisestage URL.","setup.error_url"=>"Sisestage kehtiv URL","setup.save_error"=>"Salvestamise viga:","setup.button"=>"Jätka","setup.saving"=>"Salvestamine...");
strings!(LV, "tray.show"=>"Rādīt","tray.reload_config"=>"Pārlādēt konfigurāciju","tray.register_tel"=>"Reģistrēt kā tel: apstrādātāju","tray.devtools"=>"DevTools (F12)","tray.open_config"=>"Atvērt konfigurāciju","tray.quit"=>"Iziet","popup.send_mail"=>"\u{2709}  Sūtīt atzvana e-pastu","mailto.subject"=>"Lūdzu atzvaniet: {name} {number}","hist.select_multiple"=>"Atlasīt vairākus","hist.all"=>"Visi","hist.selected_count"=>"atlasīts","hist.delete_selected"=>"Dzēst atlasītos","hist.cancel"=>"Atcelt","hist.confirm_delete"=>"Vai vēlaties dzēst {count} ierakstus?","hist.preparing"=>"Sagatavošana...","hist.deleting"=>"Dzēšana","hist.deleted"=>"dzēsts!","setup.title"=>"4dy Client iestatīšana","setup.prompt"=>"Ievadiet sava 3CX tīmekļa klienta URL:","setup.placeholder"=>"https://mans-uznemums.3cx.lv","setup.error_empty"=>"Ievadiet URL.","setup.error_url"=>"Ievadiet derīgu URL","setup.save_error"=>"Saglabāšanas kļūda:","setup.button"=>"Turpināt","setup.saving"=>"Saglabāšana...");
strings!(LT, "tray.show"=>"Rodyti","tray.reload_config"=>"Perkrauti konfigūraciją","tray.register_tel"=>"Registruoti kaip tel: tvarkyklę","tray.devtools"=>"DevTools (F12)","tray.open_config"=>"Atidaryti konfigūraciją","tray.quit"=>"Išeiti","popup.send_mail"=>"\u{2709}  Siųsti atgalinio skambučio el. laišką","mailto.subject"=>"Prašome perskambinti: {name} {number}","hist.select_multiple"=>"Pasirinkti kelis","hist.all"=>"Visi","hist.selected_count"=>"pasirinkta","hist.delete_selected"=>"Ištrinti pasirinktus","hist.cancel"=>"Atšaukti","hist.confirm_delete"=>"Ar norite ištrinti {count} įrašų?","hist.preparing"=>"Ruošiama...","hist.deleting"=>"Trinama","hist.deleted"=>"ištrinta!","setup.title"=>"4dy Client nustatymas","setup.prompt"=>"Įveskite savo 3CX žiniatinklio kliento URL:","setup.placeholder"=>"https://mano-imone.3cx.lt","setup.error_empty"=>"Įveskite URL.","setup.error_url"=>"Įveskite galiojantį URL","setup.save_error"=>"Išsaugojimo klaida:","setup.button"=>"Tęsti","setup.saving"=>"Saugoma...");
strings!(FI, "tray.show"=>"Näytä","tray.reload_config"=>"Lataa asetukset uudelleen","tray.register_tel"=>"Rekisteröi tel: käsittelijäksi","tray.devtools"=>"DevTools (F12)","tray.open_config"=>"Avaa asetukset","tray.quit"=>"Lopeta","popup.send_mail"=>"\u{2709}  Lähetä takaisinsoittosähköposti","mailto.subject"=>"Soita takaisin: {name} {number}","hist.select_multiple"=>"Valitse useita","hist.all"=>"Kaikki","hist.selected_count"=>"valittu","hist.delete_selected"=>"Poista valitut","hist.cancel"=>"Peruuta","hist.confirm_delete"=>"Haluatko poistaa {count} merkintää?","hist.preparing"=>"Valmistellaan...","hist.deleting"=>"Poistetaan","hist.deleted"=>"poistettu!","setup.title"=>"4dy Client -asetukset","setup.prompt"=>"Anna 3CX-verkkoasiakkaan URL:","setup.placeholder"=>"https://yritykseni.3cx.fi","setup.error_empty"=>"Anna URL.","setup.error_url"=>"Anna kelvollinen URL","setup.save_error"=>"Tallennusvirhe:","setup.button"=>"Jatka","setup.saving"=>"Tallennetaan...");
strings!(SV, "tray.show"=>"Visa","tray.reload_config"=>"Ladda om konfiguration","tray.register_tel"=>"Registrera som tel: hanterare","tray.devtools"=>"DevTools (F12)","tray.open_config"=>"Öppna konfiguration","tray.quit"=>"Avsluta","popup.send_mail"=>"\u{2709}  Skicka återuppringningsmail","mailto.subject"=>"Vänligen ring tillbaka: {name} {number}","hist.select_multiple"=>"Välj flera","hist.all"=>"Alla","hist.selected_count"=>"valda","hist.delete_selected"=>"Ta bort valda","hist.cancel"=>"Avbryt","hist.confirm_delete"=>"Vill du ta bort {count} poster?","hist.preparing"=>"Förbereder...","hist.deleting"=>"Tar bort","hist.deleted"=>"borttagna!","setup.title"=>"Konfigurera 4dy Client","setup.prompt"=>"Ange URL för din 3CX-webbklient:","setup.placeholder"=>"https://mitt-foretag.3cx.se","setup.error_empty"=>"Ange en URL.","setup.error_url"=>"Ange en giltig URL","setup.save_error"=>"Fel vid sparning:","setup.button"=>"Fortsätt","setup.saving"=>"Sparar...");
strings!(NO, "tray.show"=>"Vis","tray.reload_config"=>"Last inn konfigurasjon på nytt","tray.register_tel"=>"Registrer som tel: handler","tray.devtools"=>"DevTools (F12)","tray.open_config"=>"Åpne konfigurasjon","tray.quit"=>"Avslutt","popup.send_mail"=>"\u{2709}  Send tilbakeringingsmail","mailto.subject"=>"Vennligst ring tilbake: {name} {number}","hist.select_multiple"=>"Velg flere","hist.all"=>"Alle","hist.selected_count"=>"valgt","hist.delete_selected"=>"Slett valgte","hist.cancel"=>"Avbryt","hist.confirm_delete"=>"Vil du slette {count} oppføringer?","hist.preparing"=>"Forbereder...","hist.deleting"=>"Sletter","hist.deleted"=>"slettet!","setup.title"=>"Konfigurer 4dy Client","setup.prompt"=>"Skriv inn URL-en til din 3CX-webklient:","setup.placeholder"=>"https://mitt-firma.3cx.no","setup.error_empty"=>"Skriv inn en URL.","setup.error_url"=>"Skriv inn en gyldig URL","setup.save_error"=>"Feil ved lagring:","setup.button"=>"Fortsett","setup.saving"=>"Lagrer...");
strings!(DA, "tray.show"=>"Vis","tray.reload_config"=>"Genindlæs konfiguration","tray.register_tel"=>"Registrer som tel: handler","tray.devtools"=>"DevTools (F12)","tray.open_config"=>"Åbn konfiguration","tray.quit"=>"Afslut","popup.send_mail"=>"\u{2709}  Send tilbagekaldsmail","mailto.subject"=>"Ring venligst tilbage: {name} {number}","hist.select_multiple"=>"Vælg flere","hist.all"=>"Alle","hist.selected_count"=>"valgt","hist.delete_selected"=>"Slet valgte","hist.cancel"=>"Annuller","hist.confirm_delete"=>"Vil du slette {count} poster?","hist.preparing"=>"Forbereder...","hist.deleting"=>"Sletter","hist.deleted"=>"slettet!","setup.title"=>"Konfigurer 4dy Client","setup.prompt"=>"Indtast URL til din 3CX webklient:","setup.placeholder"=>"https://min-virksomhed.3cx.dk","setup.error_empty"=>"Indtast en URL.","setup.error_url"=>"Indtast en gyldig URL","setup.save_error"=>"Fejl ved gemning:","setup.button"=>"Fortsæt","setup.saving"=>"Gemmer...");
strings!(EL, "tray.show"=>"Εμφάνιση","tray.reload_config"=>"Επαναφόρτωση ρυθμίσεων","tray.register_tel"=>"Εγγραφή ως χειριστής tel:","tray.devtools"=>"DevTools (F12)","tray.open_config"=>"Άνοιγμα ρυθμίσεων","tray.quit"=>"Έξοδος","popup.send_mail"=>"\u{2709}  Αποστολή email επιστροφής κλήσης","mailto.subject"=>"Παρακαλώ καλέστε πίσω: {name} {number}","hist.select_multiple"=>"Επιλογή πολλαπλών","hist.all"=>"Όλα","hist.selected_count"=>"επιλεγμένα","hist.delete_selected"=>"Διαγραφή επιλεγμένων","hist.cancel"=>"Ακύρωση","hist.confirm_delete"=>"Θέλετε να διαγράψετε {count} εγγραφές;","hist.preparing"=>"Προετοιμασία...","hist.deleting"=>"Διαγραφή","hist.deleted"=>"διαγράφηκαν!","setup.title"=>"Ρύθμιση 4dy Client","setup.prompt"=>"Εισάγετε το URL του 3CX web client σας:","setup.placeholder"=>"https://i-etairia-mou.3cx.gr","setup.error_empty"=>"Εισάγετε ένα URL.","setup.error_url"=>"Εισάγετε έγκυρο URL","setup.save_error"=>"Σφάλμα αποθήκευσης:","setup.button"=>"Συνέχεια","setup.saving"=>"Αποθήκευση...");

#[cfg(test)]
mod tests {
    use super::*;

    /// All language codes that `get_strings_for` recognises.
    const SUPPORTED_LANGS: &[&str] = &[
        "en", "de", "fr", "it", "es", "nl", "pt", "pl", "cs", "hu", "sk", "sl", "ro", "bg", "hr",
        "et", "lv", "lt", "fi", "sv", "no", "da", "el",
    ];

    #[test]
    fn unknown_language_falls_back_to_english() {
        let unknown = get_strings_for("xx");
        let english = get_strings_for("en");
        // pointer equality: the unknown branch must return the EN map
        assert!(std::ptr::eq(unknown, english));
    }

    #[test]
    fn german_and_english_translations_differ() {
        let de = get_strings_for("de");
        let en = get_strings_for("en");
        assert_ne!(
            de.get("mailto.subject"),
            en.get("mailto.subject"),
            "DE and EN should provide distinct mailto.subject translations"
        );
    }

    #[test]
    fn every_supported_language_has_core_keys() {
        // These keys are required by every translation set; if a language
        // is missing one we want to know at test time, not at runtime.
        let required = [
            "mailto.subject",
            "tray.show",
            "tray.quit",
            "setup.title",
            "setup.button",
        ];
        for lang in SUPPORTED_LANGS {
            let map = get_strings_for(lang);
            for key in &required {
                assert!(
                    map.contains_key(key),
                    "language `{}` is missing required key `{}`",
                    lang,
                    key
                );
            }
        }
    }

    #[test]
    fn t_returns_question_marks_for_unknown_key() {
        // STRINGS may or may not be initialised depending on test order;
        // either way, the EN fallback also doesn't know this key.
        assert_eq!(t("definitely.not.a.real.key.zzz"), "???");
    }

    #[test]
    fn t_returns_english_translation_when_strings_uninitialised() {
        // EN is the hard-coded fallback inside `t()`, so this works
        // regardless of whether init() has been called.
        let value = t("tray.show");
        // Either the German "Anzeigen" (if init ran on a German Windows
        // host) or the English "Show". Both are valid; "???" is not.
        assert_ne!(value, "???");
    }
}
