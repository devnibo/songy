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
use std::{fs, thread, time};

mod langs;

/*
 * 4096 is the max character length
 * of the text parameter in the sendMessage
 * method of the telegram bot api
 * (char>=1byte )
 * see: https://core.telegram.org/bots/api#sendmessage
*/
const MAX_TEXT_LEN: usize = 4096;

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
	let songs_path: String = add_ending_slash(args.songs_path);
	let strings = langs::Strings::new(args.lang, songs_path.clone());
	let api = Api::new(&args.token.as_str());
	let mut updates_params = GetUpdatesParams::builder()
		.allowed_updates(vec![AllowedUpdate::Message])
		.build();
	loop {
		let dur = time::Duration::from_millis(500);
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
	let text: &str = msg.text
		.as_ref()
		.expect("Error while extracting text message.");
	let chat_id: u64 = msg.from
		.as_ref()
		.expect("Error while extracting user.").id;
	let mut params = SendMessageParams::builder()
		.chat_id(ChatId::Integer(chat_id.try_into().unwrap()))
		.text("")
		.build();
	match text {
		"/start" => {
			params.text = (*strings.start_msg).to_string();
			send_message(api, &mut params);
		},
		"/list" => {
			let songs = get_songs(songs_path, None);
			params.text = form_message(&songs);
			send_message(api, &mut params);
		},
		_ => {
			if text.starts_with("/") {
				for name in langs::get_folder_names(songs_path) {
					if text == "/".to_owned() + name.as_str() {
						let songs = get_songs(songs_path, Some(&name));
						params.text = form_message(&songs);
						send_message(api, &mut params);
						return;
					}
				}
				let len = text.as_bytes().len();
				let maybe_files = find_songs(&text[1..len].to_string(), songs_path, strings);
				match maybe_files {
					Ok(files) => {
						let file = files.get(0);
						let input_file = InputFile::builder()
							.path(file.expect("Whatever").path())
							.build();
						let send_document_params = SendDocumentParams::builder()
							.chat_id(ChatId::Integer(chat_id.try_into().unwrap()))
							.document(File::InputFile(input_file))
							.build();
						send_document(api, &send_document_params);
					},
					Err(err) => {
						println!("{}", err.message);
						params.text = (*strings.song_not_found).to_string();
						send_message(api, &mut params);
					}
				}
			}
			else {
				match find_songs(&text.to_string(), songs_path, strings) {
					Ok(files) => {
						let mut send_document_params = SendDocumentParams::builder()
							.chat_id(ChatId::Integer(chat_id.try_into().unwrap()))
							.document(File::String(String::new())) // Just to have a val
							.build();
						if files.len() == 1 {
							let file = files.get(0).expect("Err.");
							let input_file = InputFile::builder()
								.path(file.path())
								.build();
							send_document_params.document = File::InputFile(input_file);
							send_document(api, &send_document_params);
						}
						else {
							params.text = form_message(&files);
							send_message(api, &mut params);
						}
					},
					Err(err) => {
						params.text = (*strings.song_not_found).to_string();
						send_message(api, &mut params);
						println!("{}", err.message);
					}
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

fn send_message(api: &Api, params: &mut SendMessageParams) {
	let text_len = params.text.chars().count();
	let msg_count = text_len as f64 / MAX_TEXT_LEN as f64;
	if msg_count <= 1.0 {
		let result = api.send_message(params);
		match result {
			Err(err) => {
				println!("send_message failed.");
				dbg!(err);
			},
			Ok(_res) => {}
		}
	} else {
		let mut text: String = params.text.clone();
		let mut part: &str;
		loop {
			if text.chars().count() > MAX_TEXT_LEN {
				match find_last_line_break(text.clone()) {
					Ok(index) => {
						part = &text[..index];
						params.text = part.to_string();
						let result = api.send_message(params);
						match result {
							Err(err) => {
								println!("send_message failed.");
								dbg!(err);
							},
							Ok(_res) => {}
						}
						text = text[index+1..].to_string();
					},
					Err(_) => {
						println!("Dude, there's no line break. Deal with it.");
					}
				}
			} else {
				params.text = text;
				let result = api.send_message(params);
				match result {
					Err(err) => {
						println!("send_message failed.");
						dbg!(err);
					},
					Ok(_res) => {}
				}
				break;
			}
		}
	}
}

fn find_last_line_break(text: String) -> Result<usize, usize> {
	let mut i: usize = MAX_TEXT_LEN as usize;
	loop {
		if i == 0 {
			return Err(i);
		}
		match text.chars().nth(i) {
			Some(c) => {
				if c == '\n' {
					return Ok(i);
				}
			},
			None => {
				println!("nth error");
			}
		}
		i -= 1;
	}
}

fn form_message(songs: &Vec<fs::DirEntry>) -> String {
	let mut message = String::new();
	for song in songs {
		let file_name = song.file_name();
		let filename = file_name.to_str().expect("Error: ");
		let s: Vec<&str> = filename.split(".").collect();
		let name = s.get(0).expect("Error: get(0)");
		let mut command: String = "/".to_string();
		command.push_str(name);
		command.push_str("\n");
		message.push_str(command.as_str());
	}
	return message;
}

fn find_songs(search_string: &String, songs_path: &String, strings: &langs::Strings) -> Result<Vec<fs::DirEntry>, ErrNotFound> {
	let mut result: Vec<fs::DirEntry> = vec![];
	let mut filename: String;
	let ss = strings.format(search_string).to_lowercase();
	for file in get_songs(songs_path, None) {
		filename = file.file_name().to_str().expect("Error: name").to_string();
		let f: Vec<&str> = filename.split(".").collect();
		let mut name: String = f.get(0).expect("Error: get(0)").to_string();
		name = name.to_lowercase();
		if name == ss {
			let mut one = vec![file];
			one.append(&mut result);
			result = one;
		} else if name.contains(&ss) {
			result.push(file);
		}
	}
	if result.len() > 0 {
		return Ok(result);
	}
	else {
		return Err(ErrNotFound { message: "Didn't find any song.".to_string() });
	}
}

fn get_songs(songs_path: &String, folder_name: Option<&String>) -> Vec<fs::DirEntry> {
	match folder_name {
		Some(name) => {
			return get_files_recursive(&(songs_path.to_owned() + name));
		},
		None => {
			return get_files_recursive(songs_path);
		}
	}
}

fn get_files_recursive(folder_path: &String) -> Vec<fs::DirEntry> {
	let path = fs::read_dir(folder_path);
	let mut is_dir: bool;
	let mut songs: Vec<fs::DirEntry> = vec![];
	match path {
		Ok(read_dir) => {
			for r in read_dir {
				match r {
					Ok(dir_entry) => {
						is_dir = dir_entry.file_type().expect("Error: is_dir").is_dir();
						if is_dir {
							let path = dir_entry.path().to_str().expect("Error: name").to_string();
							for song in get_files_recursive(&path) {
								songs.push(song);
							}
						} else {
							songs.push(dir_entry);
						}
					},
					Err(err) => {
						println!("Cannot access filepath.");
						dbg!(err);
					}
				}
			}
		},
		Err(_) => {
			println!("Cannot open/read or what ever the path {}.", folder_path);
		}
	}
	songs.sort_by_key(|name| name.file_name().into_string().expect("Error").to_lowercase());
	return songs;
}
