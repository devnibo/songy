use std::str;

pub struct Strings {
	lang: String,
	pub start_msg: String,
	pub song_not_found: String
}

impl Strings {
	pub fn new(lang: String) -> Self {
		let start_msg: String;
		let song_not_found: String;
		let l: &str = lang.as_str();
		match l {
			"de" => {
				start_msg = concat!(
					"Hallo. Dies ist ein digitales Liederbuch. :)\n",
					"/list - Listet alle Lieder auf.\n",
					"Ansonsten tippe einfach den Titel oder Teile des Titels",
					" des Liedes ein und du bekommst dein Lied zugeschickt."
				).to_string();
				song_not_found = "Kein Lied mit diesem Titel gefunden.".to_string();
			},
			"md" => {
				start_msg = concat!(
					"Salut! Această e o carte de cântari digitală. :)\n",
					"/list - Listează toate cântările.\n",
					"Deasemenea puteți introduce titlul sau cuvinte din titlul",
					" cântării iar bot-ul va găsi piesa corespondentă."
				).to_string();
				song_not_found = "Niciun cântec găsit cu acest nume".to_string();
			},
			_ => {
				start_msg = concat!(
					"Hello. This is a digital song book. :)\n",
					"/list - Lists all songs.\n",
					"Otherwise simply type the title or parts of the title",
					" of the song and you will receive the song."
				).to_string();
				song_not_found = "Didn't find any song with this title.".to_string();
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


