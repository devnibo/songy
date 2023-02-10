use clap::Parser;
use frankenstein::Api;
use frankenstein::TelegramApi;
use frankenstein::SendMessageParams;
use frankenstein::GetUpdatesParams;
use frankenstein::ChatId;
use frankenstein::Message;
use frankenstein::api_params::File;
use frankenstein::api_params::InputFile;
use frankenstein::api_params::SendDocumentParams;
use frankenstein::objects::UpdateContent;
use frankenstein::objects::AllowedUpdate;
use std::{fs, ffi, thread, time};
use std::path::PathBuf;

mod langs;

struct ErrNotFound {
	message: String
}

#[derive(Parser, Debug)]
struct Args {
	#[arg(short, long, help = "Telegram Bot API Token")]
	token: String,
	#[arg(short, long, help = "Absolute Path To Folder With PDF Files")]
	songs_path: String,
	#[arg(short, long, default_value = "en", help = "Language that the bot speaks: 'en', 'de' or 'md'")]
	lang: String
}

fn main() {
	let args = Args::parse();
	let strings = langs::Strings::new(args.lang);
	let songs_path: String = add_ending_slash(args.songs_path);
	let api = Api::new(&args.token.as_str());
	let mut updates_params = GetUpdatesParams::builder()
		.allowed_updates(vec![AllowedUpdate::Message])
		.build();
	loop {
		let dur = time::Duration::new(1, 0);
		thread::sleep(dur);
		let result = TelegramApi::get_updates(&api, &updates_params);
		match result {
			Ok(val) => {
				for update in &val.result {
					updates_params.offset = Some(i64::from(update.update_id) + 1);
					match &update.content {
						UpdateContent::Message(msg) => {
							if msg.text.is_some() {
								handle_text_message(&api, msg, &strings, &songs_path);
							}
						},
						_ => {}
					}
				}
			},
			Err(_err) => {
				println!("Error receiving updates from Telegram Bot API.");
			}
		}
	}
}

fn add_ending_slash(path: String) -> String {
	if !path.ends_with("/") {
		let mut new_path = path.to_owned();
		new_path.push_str("/");
		return new_path;
	}
	else {
		return path;
	}
}

fn handle_text_message(api: &Api, msg: &Message, strings: &langs::Strings, songs_path: &String) {
	let text: &String = msg.text
		.as_ref()
		.expect("Error while extracting text message.");
	let chat_id: u64 = msg.from
		.as_ref()
		.expect("Error while extracting user.").id;
	let mut params = SendMessageParams::builder()
		.chat_id(ChatId::Integer(chat_id.try_into().unwrap()))
		.text("")
		.build();
	if text == "/start" {
		params.text = (*strings.start_msg).to_string();
		send_message(api, &params);
	}
	else if text == "/list" {
		let songs = get_all_songs(songs_path);
		for song in &songs {
			let s: Vec<&str> = song.split(".").collect();
			let name = s.get(0).expect("Error: get(0)");
			let mut command: String = "/".to_string();
			command.push_str(name);
			command.push_str("\n");
			params.text.push_str(command.as_str());
		}
		send_message(api, &params);
	}
	else {
		if text.starts_with("/") {
			let len = text.as_bytes().len();
			let maybe_file = get_song(&text[1..len], songs_path);
			match maybe_file {
				Ok(filepath) => {
					let mut path = PathBuf::new();
					path.push(filepath);
					let input_file = InputFile { path };
					let send_document_params = SendDocumentParams::builder()
						.chat_id(ChatId::Integer(chat_id.try_into().unwrap()))
						.document(File::InputFile(input_file))
						.build();
					send_document(api, &send_document_params);
				},
				Err(err) => {
					println!("{}", err.message);
					params.text = (*strings.song_not_found).to_string();
					send_message(api, &params);
				}
			}
		}
		else {
			match find_songs(&text, songs_path, strings) {
				Ok(files) => {
					let mut send_document_params = SendDocumentParams::builder()
						.chat_id(ChatId::Integer(chat_id.try_into().unwrap()))
						.document(File::String(String::new())) // Just to have a val
						.build();
					if files.len() == 1 {
						let file = files.get(0).expect("Err.");
						let mut filepath: String = songs_path.to_owned();
						filepath.push_str(file.as_str());
						let mut path = PathBuf::new();
						path.push(filepath);
						let input_file = InputFile { path };
						send_document_params.document = File::InputFile(input_file);
						send_document(api, &send_document_params);
					}
					else {
						for file in files {
							let f: Vec<&str> = file.split(".").collect();
							let name = f.get(0).expect("Error: get(0)");
							let mut command: String = "/".to_string();
							command.push_str(name);
							command.push_str("\n");
							params.text.push_str(command.as_str());
						}
						send_message(api, &params);
					}
				},
				Err(err) => {
					params.text = (*strings.song_not_found).to_string();
					send_message(api, &params);
					println!("{}", err.message);
				}
			}
		}
	}
}

fn send_document(api: &Api, params: &SendDocumentParams) {
	let result = api.send_document(params);
	match result {
		Err(err) => {
			println!("send_document failed.");
			dbg!(err);
		},
		Ok(_res) => {}
	}
}

fn send_message(api: &Api, params: &SendMessageParams) {
	let result = api.send_message(params);
	match result {
		Err(err) => {
			println!("send_message failed.");
			dbg!(err);
		},
		Ok(_res) => {}
	}
}

fn find_songs(search_string: &String, songs_path: &String, strings: &langs::Strings) -> Result<Vec<String>, ErrNotFound> {
	let mut songs: Vec<String> = vec![];
	for file in get_all_songs(songs_path) {
		let f: Vec<&str> = file.split(".").collect();
		let mut name: String = f.get(0).expect("Error: get(0)").to_string();
		name = name.to_lowercase();
		let ss = strings.format(search_string).to_lowercase();
		if name.contains(&ss) {
			songs.push(file);
		}
	}
	if songs.len() > 0 {
		return Ok(songs);
	}
	else {
		return Err(ErrNotFound { message: "Didn't find any song.".to_string() });
	}
}

fn get_song(song_name: &str, songs_path: &String) -> Result<String, ErrNotFound> {
	let mut filepath = String::new();
	for file in get_all_songs(songs_path) {
		let f: Vec<&str> = file.split(".").collect();
		let name: &str = f.get(0).expect("Error: get(0)");
		if name == song_name {
			filepath = songs_path.to_owned();
			filepath.push_str(&file);
			return Ok(filepath);
		}
	}
	return Err(ErrNotFound { message: "Didn't find song.".to_string() });
}

fn get_all_songs(songs_path: &String) -> Vec<String> {
	let paths = fs::read_dir(songs_path);
	let mut songs: Vec<String> = vec![];
	match paths {
		Ok(read_dir) => {
			for r in read_dir {
				match r {
					Ok(dir_entry) => {
						let filename: ffi::OsString = dir_entry.file_name();
						let name: String = filename.to_str().expect("Error: filename").to_string();
						songs.push(name);
					},
					Err(err) => {
						println!("Cannot access filepath.");
						dbg!(err);
					}
				}
			}
		},
		Err(_) => {
			println!("Cannot open/read or what ever the --songs-path.");
		}
	}
	return songs;
}
