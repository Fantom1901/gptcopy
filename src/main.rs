use clap::{Command, CommandFactory, Parser, Subcommand, ValueHint};
use clap_complete::{ generate, Shell};
use std::{env, fs, io};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, Stdio};

const IGNORE_LIST: &[&str] = &[
  "node_modules", "venv", ".venv", ".git", ".idea", ".vscode", "__pycache__",
  "target", "dist", "build", ".next", ".astro", ".cache",
  "package-lock.json", "pnpm-lock.yaml", "yarn.lock",
];

// Ключевые слова для поиска случайно закоммиченных секретов
const SECRET_PATTERNS: &[&str] = &[
  "BEGIN PRIVATE KEY",
  "BEGIN RSA PRIVATE KEY",
  "api_key",
  "apikey",
  "secret_key",
  "aws_access_key_id",
  "ghp_",
];

#[derive(Parser)]
#[command(
  name = "gptcopy",
  version,
  about = "Подготовка контекста проекта для LLM",
  long_about = None
)]
struct Cli {
  #[command(subcommand)]
  command: Option<Commands>,

  #[arg(value_hint = ValueHint::AnyPath)]
  targets: Vec<PathBuf>,

  #[arg(short, long)]
  changed: bool,

  #[arg(short, long)]
  minify: bool,

  #[arg(short = 'x', long)]
  xml: bool,

  #[arg(short, long, value_delimiter = ',')]
  ext: Vec<String>,

}

#[derive(Subcommand)]
enum Commands {
  Completions {
    #[arg(value_enum)]
    shell: Shell,
  }
}

fn is_ignored(path: &Path, custom_ignores: &[String]) -> bool {
  if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
    if IGNORE_LIST.contains(&name) {
      return true;
    }
    if custom_ignores.iter().any(|ig| ig == name) {
      return true;
    }
  }
  false
}

fn load_gitignore(dir: &Path) -> Vec<String> {
  let gitignore_path = dir.join(".gitignore");
  let mut ignores = Vec::new();
  if let Ok(content) = fs::read_to_string(gitignore_path) {
    for line in content.lines() {
      let trimmed = line.trim();
      if !trimmed.is_empty() && !trimmed.starts_with('#') {
        // Убираем слеши в начале и конце для простого сопоставления по имени
        let clean = trimmed.trim_matches('/');
        ignores.push(clean.to_string());
      }
    }
  }
  ignores
}

fn is_text_file(path: &Path) -> bool {
  if let Ok(mut file) = fs::File::open(path) {
    let mut buffer = [0u8; 512];
    if let Ok(n) = file.read(&mut buffer) {
      if n == 0 {
        return true;
      }
      return !buffer[..n].contains(&0);
    }
  }
  false
}

fn matches_extension(path: &Path, extensions: &[String]) -> bool {
  if extensions.is_empty() {
    return true;
  }
  if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
    extensions.iter().any(|target_ext| target_ext.eq_ignore_ascii_case(ext))
  } else {
    false
  }
}

fn scan_for_secrets(path: &Path, content: &str) {
  let lower_content = content.to_lowercase();
  for pattern in SECRET_PATTERNS {
    if lower_content.contains(&pattern.to_lowercase()) {
      eprintln!(
        "\x1b[1;31m[ПРЕДУПРЕЖДЕНИЕ]\x1b[0m Файл \x1b[1m{}\x1b[0m может содержать секреты/ключи! (Найдено: '{}')",
        path.display(),
        pattern
      );
    }
  }
}

fn collect_files(
  dir: &Path,
  files: &mut Vec<PathBuf>,
  extensions: &[String],
  custom_ignores: &[String],
) {
  if let Ok(entries) = fs::read_dir(dir) {
    for entry in entries.flatten() {
      let path = entry.path();

      if is_ignored(&path, custom_ignores) {
        continue;
      }

      if path.is_dir() {
        collect_files(&path, files, extensions, custom_ignores);
      } else if path.is_file()
        && is_text_file(&path)
        && matches_extension(&path, extensions)
      {
        files.push(path);
      }
    }
  }
}

fn print_tree(dir: &Path, prefix: String, out: &mut String, custom_ignores: &[String]) {
  if is_ignored(dir, custom_ignores) {
    return;
  }

  if let Ok(entries) = fs::read_dir(dir) {
    let mut valid_entries: Vec<_> = entries
      .flatten()
      .map(|e| e.path())
      .filter(|p| !is_ignored(p, custom_ignores))
      .collect();

    valid_entries.sort();
    let total = valid_entries.len();

    for (i, path) in valid_entries.iter().enumerate() {
      let is_last = i == total - 1;
      let pointer = if is_last { "└── " } else { "├── " };
      let name = path.file_name().unwrap_or_default().to_string_lossy();

      out.push_str(&format!("{}{}{}\n", prefix, pointer, name));

      if path.is_dir() {
        let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
        print_tree(path, new_prefix, out, custom_ignores);
      }
    }
  }
}

fn get_clipboard_process() -> Option<ProcessCommand> {
  let term = env::var("TERM").unwrap_or_default();

  if term == "xterm-kitty" && ProcessCommand::new("kitten").arg("--version").output().is_ok() {
    let mut cmd = ProcessCommand::new("kitten");
    cmd.arg("clipboard");
    return Some(cmd);
  }

  if ProcessCommand::new("wl-copy").arg("--version").output().is_ok() {
    return Some(ProcessCommand::new("wl-copy"));
  }

  if ProcessCommand::new("xclip").arg("-version").output().is_ok() {
    let mut cmd = ProcessCommand::new("xclip");
    cmd.args(["-selection", "clipboard"]);
    return Some(cmd);
  }

  None
}

fn expand_tilde(path: &Path) -> PathBuf {
  if let Some(path_str) = path.to_str() {
    if path_str.starts_with("~/") || path_str == "~" {
      if let Ok(home) = env::var("HOME") {
        return PathBuf::from(path_str.replacen('~', &home, 1));
      }
    }
  }
  path.to_owned()
}

fn print_completions<G: clap_complete::Generator>(generator: G, cmd: &mut Command) {
  generate(generator, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

fn main() {
  let cli = Cli::parse();

  if let Some(Commands::Completions { shell }) = cli.command {
    let mut cmd = Cli::command();
    print_completions(shell, &mut cmd);
    return;
  }

  let targets = if cli.targets.is_empty() {
    vec![PathBuf::from(".")]
  } else {
    cli.targets.into_iter().map(|p| expand_tilde(&p)).collect()
  };

  let root_target = &targets[0];
  let custom_ignores = load_gitignore(root_target);

  let mut output = String::new();

  // 1. СТРУКТУРА ПРОЕКТА
  if cli.xml {
    output.push_str("<project_structure>\n");
  } else {
    output.push_str("=== PROJECT STRUCTURE ===\n");
  }

  if cli.changed {
    let git_status = ProcessCommand::new("git").args(["status", "-s"]).output();
    match git_status {
      Ok(out) if out.status.success() => {
        output.push_str(&String::from_utf8_lossy(&out.stdout));
      }
      _ => output.push_str("Git не инициализирован\n"),
    }
  } else {
    output.push_str(&format!("{}\n", root_target.display()));
    print_tree(root_target, "".to_string(), &mut output, &custom_ignores);
  }

  if cli.xml {
    output.push_str("</project_structure>\n\n<source_code>\n");
  } else {
    output.push_str("\n=== SOURCE CODE ===\n\n");
  }

  // 2. СБОР ФАЙЛОВ
  let mut files_to_process: Vec<PathBuf> = Vec::new();

  if cli.changed {
    let git_files = ProcessCommand::new("git")
      .args(["ls-files", "-m", "-o", "--exclude-standard"])
      .output();

    if let Ok(out) = git_files {
      let stdout = String::from_utf8_lossy(&out.stdout);
      for line in stdout.lines() {
        let path = PathBuf::from(line);
        if !is_ignored(&path, &custom_ignores)
          && path.is_file()
          && matches_extension(&path, &cli.ext)
        {
          files_to_process.push(path);
        }
      }
    }
  } else {
    for target in &targets {
      if target.is_file() {
        if !is_ignored(target, &custom_ignores) && matches_extension(target, &cli.ext) {
          files_to_process.push(target.clone());
        }
      } else if target.is_dir() {
        collect_files(target, &mut files_to_process, &cli.ext, &custom_ignores);
      }
    }
  }

  // 3. ОБРАБОТКА И ФОРМАТИРОВАНИЕ
  for file_path in &files_to_process {
    eprint!("\x1b[32m  ->\x1b[0m Обработка: {}\n", file_path.display());
    let _ = io::stderr().flush();

    if let Ok(content) = fs::read_to_string(file_path) {
      // Проверка на секреты
      scan_for_secrets(file_path, &content);

      if cli.xml {
        output.push_str(&format!("<file path=\"{}\">\n", file_path.display()));
      } else {
        output.push_str(&format!("FILE: {}\n```\n", file_path.display()));
      }

      if cli.minify {
        for line in content.lines() {
          if !line.trim().is_empty() {
            output.push_str(line);
            output.push('\n');
          }
        }
      } else {
        output.push_str(&content);
        if !content.ends_with('\n') {
          output.push('\n');
        }
      }

      if cli.xml {
        output.push_str("</file>\n\n");
      } else {
        output.push_str("```\n\n");
      }
    }
  }

  if cli.xml {
    output.push_str("</source_code>\n");
  }

  // 4. КОПИРОВАНИЕ В БУФЕР ОБМЕНА
  if let Some(mut child_cmd) = get_clipboard_process() {
    let mut child = child_cmd
      .stdin(Stdio::piped())
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .spawn()
      .expect("Не удалось запустить утилиту копирования");

    if let Some(mut stdin) = child.stdin.take() {
      let _ = stdin.write_all(output.as_bytes());
      let _ = stdin.flush();
    }

    let _ = child.wait();
  } else {
    eprintln!("\x1b[1;33m[!]\x1b[0m Утилита для копирования не найдена. Вывод в консоль:\n");
    println!("{}", output);
  }

  // 5. ИТОГОВАЯ СТАТИСТИКА
  let total_bytes = output.len();
  let total_kb = total_bytes as f64 / 1024.0;
  let estimated_tokens = total_bytes / 4;

  eprintln!(
    "\x1b[1;34m::\x1b[0m \x1b[32mГотово!\x1b[0m Обработано файлов: \x1b[1m{}\x1b[0m | Размер: \x1b[1m{:.1} KB\x1b[0m | ~Tokens: \x1b[1;33m{}\x1b[0m",
    files_to_process.len(),
    total_kb,
    estimated_tokens
  );
}