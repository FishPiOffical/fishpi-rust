use rustyline::{Editor, Config, CompletionType};
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::FileHistory;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};
use colored::*;
use std::io;
use fishpi_rust::ChatRoomUser;


pub struct CommandItem {
    pub name: &'static str,
    pub desc: &'static str,
}

/// 命令补全器
pub struct CommandCompleter {
    commands: Vec<CommandItem>,
    users: Vec<ChatRoomUser>,
}

impl CommandCompleter {
    fn new() -> Self {
        Self {
            commands: vec![],
            users: vec![],
        }
    }

    fn set_commands(&mut self, commands: Vec<CommandItem>) {
        self.commands = commands;
    }
    
    pub fn set_users(&mut self, users: Vec<ChatRoomUser>) {
        self.users = users;
    }
}

impl Helper for CommandCompleter {}
impl Highlighter for CommandCompleter {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> std::borrow::Cow<'l, str> {
        if line.starts_with(':') {
            line.green().to_string().into()
        } else {
            line.into()
        }
    }
}
impl Hinter for CommandCompleter {
    type Hint = String;
}
impl Validator for CommandCompleter {}

impl Completer for CommandCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        // 命令补全（以:开头）
        if line.starts_with(':') {
            let candidates: Vec<Pair> = self
                .commands
                .iter()
                .filter(|cmd| cmd.name.starts_with(line))
                .map(|cmd| Pair {
                    display: format!("{} - {}", cmd.name.green(), cmd.desc.cyan()),
                    replacement: cmd.name.to_string(),
                })
                .collect();

            return Ok((0, candidates));
        }

        // @用户名补全
        if let Some(at_pos) = line.rfind('@') {
            if at_pos < pos {
                let prefix = &line[at_pos + 1..pos];
                let candidates: Vec<Pair> = self
                    .users
                    .iter()
                    .filter(|user| user.user_name.to_lowercase().starts_with(&prefix.to_lowercase()))
                    .map(|user| Pair {
                        display: user.user_name.cyan().to_string(),
                        replacement: user.user_name.clone(),
                    })
                    .collect();

                return Ok((at_pos + 1, candidates));
            }
        }

        Ok((0, vec![]))
    }
}

pub struct CrosstermInputHandler {
    editor: Editor<CommandCompleter, FileHistory>,
}

impl CrosstermInputHandler {
    pub fn new() -> Self {
        let config = Config::builder()
            .completion_show_all_if_ambiguous(true)
            .completion_type(CompletionType::List)
            .build();

        let mut editor = Editor::with_config(config).unwrap_or_else(|e| {
            eprintln!("警告: 初始化输入编辑器失败: {}", e);
            Editor::with_config(config).expect("无法创建备用编辑器")
        });

        // 设置补全器
        editor.set_helper(Some(CommandCompleter::new()));

        Self { editor }
    }

    pub fn set_commands(&mut self, commands: Vec<CommandItem>) {
        if let Some(helper) = self.editor.helper_mut() {
            helper.set_commands(commands);
        }
    }
    
    pub fn set_users(&mut self, users: Vec<ChatRoomUser>) {
        if let Some(helper) = self.editor.helper_mut() {
            helper.set_users(users);
        }
    }

    pub async fn start_input_loop(&mut self, prompt: &str) -> io::Result<Option<String>> {
        match self.editor.readline(prompt) {
            Ok(line) => {
                if !line.trim().is_empty() {
                    let _ = self.editor.add_history_entry(&line);
                }
                Ok(Some(line))
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl+C
                Ok(None)
            }
            Err(ReadlineError::Eof) => {
                // Ctrl+D
                Ok(None)
            }
            Err(err) => {
                eprintln!("读取输入错误: {}", err);
                Ok(None)
            }
        }
    }

    pub async fn read_password(&mut self, prompt: &str) -> io::Result<Option<String>> {
        use std::io::Write;
        print!("{}", prompt);
        io::stdout().flush()?;
        
        match rpassword::read_password() {
            Ok(password) => Ok(Some(password)),
            Err(_) => Ok(None),
        }
    }
}
