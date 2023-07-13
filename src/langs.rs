use std::{str, fs};

#[derive(Clone)]
pub struct Strings {
	lang: String,
	pub start_msg: String,
	pub song_not_found: String
}

impl Strings {
	pub fn new(lang: String, songs_path: String) -> Self {
		let start_msg: String;
		let song_not_found: String;
		let l: &str = lang.as_str();
		match l {
			"de" => {
				start_msg = String::from(format!(
					"Hallo. Dies ist ein digitales Liederbuch. :)\n\
					Befehle:\n\
					/list - Listet alle Lieder auf\n\
					{}\
					Ansonsten tippe einfach den Titel oder Teile des Titels \
					des Liedes ein und du bekommst dein Lied zugeschickt.",
					get_commands(songs_path).as_str()
				));
				song_not_found = String::from("Kein Lied mit diesem Titel gefunden.");
			},
			"md" => {
				start_msg = String::from(format!(
					"Salut! Această e o carte de cântari digitală. :)\n\
					Comenzi:\n\
					/list - Listează toate cântările\n\
					{}\
					Deasemenea puteți introduce titlul sau cuvinte din titlul \
					cântării iar bot-ul va găsi piesa corespondentă.",
					get_commands(songs_path).as_str()
				));
				song_not_found = String::from("Niciun cântec găsit cu acest nume");
			},
			_ => {
				start_msg = String::from(format!(
					"Hello. This is a digital song book. :)\n\
					Commands:\n\
					/list - Lists all songs\n\
					{}\
					Otherwise simply type the title or parts of the title \
					of the song and you will receive the song.",
					get_commands(songs_path).as_str()
				));
				song_not_found = String::from("Didn't find any song with this title.");
			}
		}
		Self {
			lang: lang,
			start_msg: start_msg,
			song_not_found: song_not_found
		}
	}
	pub fn format(&self, name: &String) -> String {
		let mut formatted_name = name.to_string();
		let l: &str = self.lang.as_str();
		match l {
			"de" => {
				formatted_name = formatted_name.replace("Ö", "Oe");
				formatted_name = formatted_name.replace("ö", "oe");
				formatted_name = formatted_name.replace("Ü", "Ue");
				formatted_name = formatted_name.replace("ü", "ue");
				formatted_name = formatted_name.replace("Ä", "Ae");
				formatted_name = formatted_name.replace("ä", "ae");
				formatted_name = formatted_name.replace("ß", "ss");
			},
			"md" => {
				formatted_name = formatted_name.replace("ă", "a");
				formatted_name = formatted_name.replace("â", "a");
				formatted_name = formatted_name.replace("î", "i");
				formatted_name = formatted_name.replace("ș", "s");
				formatted_name = formatted_name.replace("ț", "t");
			},
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
