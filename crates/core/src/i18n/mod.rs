use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Language {
    English,
    Spanish,
}

impl Default for Language {
    fn default() -> Self {
        Language::English
    }
}

impl Language {
    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Spanish => "es",
        }
    }

    pub fn from_code(code: &str) -> Option<Language> {
        match code {
            "en" => Some(Language::English),
            "es" => Some(Language::Spanish),
            _ => None,
        }
    }
}

lazy_static! {
    pub static ref CURRENT_LANGUAGE: RwLock<Language> = RwLock::new(Language::English);
}

pub fn set_language(lang: Language) {
    *CURRENT_LANGUAGE
        .write()
        .expect("CURRENT_LANGUAGE lock poisoned") = lang;
}

pub fn get_language() -> Language {
    *CURRENT_LANGUAGE
        .read()
        .expect("CURRENT_LANGUAGE lock poisoned")
}

pub fn t(key: &str) -> String {
    let lang = get_language();
    match lang {
        Language::English => ENGLISH
            .get(key)
            .map(|&s| s.to_string())
            .unwrap_or_else(|| key.to_string()),
        Language::Spanish => SPANISH
            .get(key)
            .map(|&s| s.to_string())
            .unwrap_or_else(|| key.to_string()),
    }
}

type TranslationMap = HashMap<&'static str, &'static str>;

lazy_static! {
    pub static ref ENGLISH: TranslationMap = {
        let mut t = HashMap::new();

        // General
        t.insert("close", "Close");
        t.insert("save", "Save");
        t.insert("cancel", "Cancel");
        t.insert("ok", "OK");
        t.insert("yes", "Yes");
        t.insert("no", "No");
        t.insert("settings", "Settings");
        t.insert("loading", "Loading...");

        // Menu
        t.insert("menu", "Menu");
        t.insert("about", "About");
        t.insert("system_info", "System Info");
        t.insert("applications", "Applications");
        t.insert("rotate", "Rotate");

        // Settings Editor
        t.insert("frontlight", "Frontlight");
        t.insert("wifi", "WiFi");
        t.insert("inverted", "Inverted");
        t.insert("sleep_cover", "Sleep Cover");
        t.insert("auto_suspend", "Auto Suspend (min)");
        t.insert("auto_power_off", "Auto Power Off (h)");
        t.insert("auto_dual_page", "Auto Dual Page");
        t.insert("finished_action", "Finished Action");
        t.insert("language", "Language");
        t.insert("on", "On");
        t.insert("off", "Off");

        // Reader
        t.insert("table_of_contents", "Table of Contents");
        t.insert("go_to_page", "Go to page");
        t.insert("bookmarks", "Bookmarks");
        t.insert("annotations", "Annotations");
        t.insert("search", "Search");
        t.insert("no_results", "No results found");
        t.insert("no_next_page", "No next page.");
        t.insert("no_next_file", "No next file.");
        t.insert("previous_page", "Previous page");
        t.insert("next_page", "Next page");

        // Library
        t.insert("library", "Library");
        t.insert("all_books", "All Books");
        t.insert("recent_books", "Recent Books");
        t.insert("authors", "Authors");
        t.insert("series", "Series");
        t.insert("folders", "Folders");
        t.insert("no_books", "No books found");
        t.insert("import", "Import");
        t.insert("delete", "Delete");
        t.insert("rename", "Rename");

        // Dictionary
        t.insert("dictionary", "Dictionary");
        t.insert("enter_word", "Enter a word");
        t.insert("not_found", "Word not found");

        // Calculator
        t.insert("calculator", "Calculator");

        // Sketch
        t.insert("sketch", "Sketch");
        t.insert("clear", "Clear");
        t.insert("pen", "Pen");
        t.insert("eraser", "Eraser");

        // Messages
        t.insert("confirm_delete", "Confirm delete?");
        t.insert("processing", "Processing...");
        t.insert("error", "Error");

        t
    };

    pub static ref SPANISH: TranslationMap = {
        let mut t = HashMap::new();

        // General
        t.insert("close", "Cerrar");
        t.insert("save", "Guardar");
        t.insert("cancel", "Cancelar");
        t.insert("ok", "Aceptar");
        t.insert("yes", "Sí");
        t.insert("no", "No");
        t.insert("settings", "Configuración");
        t.insert("loading", "Cargando...");

        // Menu
        t.insert("menu", "Menú");
        t.insert("about", "Acerca de");
        t.insert("system_info", "Info del sistema");
        t.insert("applications", "Aplicaciones");
        t.insert("rotate", "Rotar");

        // Settings Editor
        t.insert("frontlight", "Luz frontal");
        t.insert("wifi", "WiFi");
        t.insert("inverted", "Invertido");
        t.insert("sleep_cover", "Cubierta de suspensión");
        t.insert("auto_suspend", "Suspensión auto (min)");
        t.insert("auto_power_off", "Apagado auto (h)");
        t.insert("auto_dual_page", "Doble página auto");
        t.insert("finished_action", "Acción al finalizar");
        t.insert("language", "Idioma");
        t.insert("on", "Activado");
        t.insert("off", "Desactivado");

        // Reader
        t.insert("table_of_contents", "Tabla de contenidos");
        t.insert("go_to_page", "Ir a página");
        t.insert("bookmarks", "Marcadores");
        t.insert("annotations", "Anotaciones");
        t.insert("search", "Buscar");
        t.insert("no_results", "No se encontraron resultados");
        t.insert("no_next_page", "No hay página siguiente.");
        t.insert("no_next_file", "No hay siguiente archivo.");
        t.insert("previous_page", "Página anterior");
        t.insert("next_page", "Página siguiente");

        // Library
        t.insert("library", "Biblioteca");
        t.insert("all_books", "Todos los libros");
        t.insert("recent_books", "Libros recientes");
        t.insert("authors", "Autores");
        t.insert("series", "Series");
        t.insert("folders", "Carpetas");
        t.insert("no_books", "No se encontraron libros");
        t.insert("import", "Importar");
        t.insert("delete", "Eliminar");
        t.insert("rename", "Renombrar");

        // Dictionary
        t.insert("dictionary", "Diccionario");
        t.insert("enter_word", "Introduce una palabra");
        t.insert("not_found", "Palabra no encontrada");

        // Calculator
        t.insert("calculator", "Calculadora");

        // Sketch
        t.insert("sketch", "Dibujo");
        t.insert("clear", "Limpiar");
        t.insert("pen", "Bolígrafo");
        t.insert("eraser", "Borrador");

        // Messages
        t.insert("confirm_delete", "¿Confirmar eliminación?");
        t.insert("processing", "Procesando...");
        t.insert("error", "Error");

        t
    };
}
