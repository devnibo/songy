use std::fs;

#[derive(Clone)]
pub struct ReportMsgs {
    pub msg: String,
    pub error_msg: String,
    pub success_msg: String,
    pub cancel_msg: String,
}

#[derive(Clone)]
pub struct I18n {
    lang: String,
    pub start_msg: String,
    pub song_not_found: String,
    pub report: ReportMsgs,
}

impl I18n {
    pub fn new(lang: String, songs_path: String) -> Self {
        match lang.as_str() {
            "de" => Self {
                lang,
                start_msg: String::from(format!(
                    "Hallo. Dies ist ein digitales Liederbuch. :)\n\
						Befehle:\n\
						/list - Listet alle Lieder auf\n\
						{}\
						Ansonsten tippe einfach den Titel oder Teile des Titels \
						des Liedes ein und du bekommst dein Lied zugeschickt.",
                    get_commands(songs_path).as_str()
                )),
                song_not_found: String::from("Kein Lied mit diesem Titel gefunden."),
                report: ReportMsgs {
                    msg: String::from(
                        "Bitte sende einen gefundenen Fehler \
							entweder als Text oder Sprachnachricht.",
                    ),
                    error_msg: String::from(
                        "Das hat nicht funktioniert. Versuche es nochmal oder /cancel.",
                    ),
                    success_msg: String::from("Deine Korrektur wurde erfolgreich gemeldet."),
                    cancel_msg: String::from("Das Fehlermelden wurde abgebrochen."),
                },
            },
            "md" => Self {
                lang,
                start_msg: String::from(format!(
                    "Salut! Această e o carte de cântari digitală. :)\n\
						Comenzi:\n\
						/list - Listează toate cântările\n\
						{}\
						Deasemenea puteți introduce titlul sau cuvinte din titlul \
						cântării iar bot-ul va găsi piesa corespondentă.",
                    get_commands(songs_path).as_str()
                )),
                song_not_found: String::from("Niciun cântec găsit cu acest nume"),
                report: ReportMsgs {
                    msg: String::from(
                        "Vă rugăm să trimiteți eroare pe care \
							ați găsit-o fie ca mesaj text sau vocal.",
                    ),
                    error_msg: String::from("Asta nu a mers. Încercați din nou sau /cancel"),
                    success_msg: String::from("A raportat corect corectia."),
                    cancel_msg: String::from("Raportarea a fost anulată."),
                },
            },
            _ => Self {
                lang,
                start_msg: String::from(format!(
                    "Hello. This is a digital song book. :)\n\
						Commands:\n\
						/list - Lists all songs\n\
						{}\
						Otherwise simply type the title or parts of the title \
						of the song and you will receive the song.",
                    get_commands(songs_path).as_str()
                )),
                song_not_found: String::from("Didn't find any song with this title."),
                report: ReportMsgs {
                    msg: String::from(
                        "Please send an error you found \
							either as text or voice message.",
                    ),
                    error_msg: String::from("That didn't work. Try again or /cancel."),
                    success_msg: String::from("Successfully reported your correction."),
                    cancel_msg: String::from("Reporting canceled."),
                },
            },
        }
    }
    pub fn format(&self, name: &String) -> String {
        let mut formatted_name = name.to_string();
        match self.lang.as_str() {
            "de" => {
                formatted_name = formatted_name.replace("Ö", "Oe");
                formatted_name = formatted_name.replace("ö", "oe");
                formatted_name = formatted_name.replace("Ü", "Ue");
                formatted_name = formatted_name.replace("ü", "ue");
                formatted_name = formatted_name.replace("Ä", "Ae");
                formatted_name = formatted_name.replace("ä", "ae");
                formatted_name = formatted_name.replace("ß", "ss");
            }
            "md" => {
                formatted_name = formatted_name.replace("ă", "a");
                formatted_name = formatted_name.replace("â", "a");
                formatted_name = formatted_name.replace("î", "i");
                formatted_name = formatted_name.replace("ș", "s");
                formatted_name = formatted_name.replace("ț", "t");
            }
            _ => {}
        }
        formatted_name = formatted_name.replace("-", "_");
        formatted_name = formatted_name.replace(" ", "_");
        return formatted_name;
    }
}

fn get_commands(songs_path: String) -> String {
    let mut commands: String = String::new();
    for name in get_folder_names(&songs_path) {
        commands.push_str(&("/".to_owned() + name.as_str() + "\n"));
    }
    return commands;
}

pub fn get_folder_names(songs_path: &String) -> Vec<String> {
    let songs_dir = fs::read_dir(songs_path).unwrap();
    let mut folder_names: Vec<String> = vec![];
    let mut is_dir: bool;
    let mut dir_entry;
    for f in songs_dir {
        dir_entry = f.expect("Error: f");
        is_dir = dir_entry.file_type().unwrap().is_dir();
        if is_dir {
            let name: String = dir_entry.file_name().to_str().unwrap().to_string();
            folder_names.push(name)
        }
    }
    return folder_names;
}
