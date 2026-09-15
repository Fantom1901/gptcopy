use std::{env, fs, io};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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
  "ghp_", // GitHub Personal Access Token
];

struct Config {
  only_changed: bool,
  minify: bool,
  use_xml: bool,
  extensions: Vec<String>,
  targets: Vec<PathBuf>,
}

fn show_help() {
  println!("\x1b[1;34m🚀 gptcopy\x1b[0m — Подготовка контекста проекта для LLM (Rust edition)");
  println!();
  println!("\x1b[1;33mИСПОЛЬЗОВАНИЕ:\x1b[0m");
  println!("  gptcopy [пути] [флаги]");
  println!();
  println!("\x1b[1;33mФЛАГИ:\x1b[0m");
  println!("  \x1b[1;32m-c, --changed\x1b[0m       Копировать только изменённые файлы (нужен Git)");
  println!("  \x1b[1;32m-m, --minify\x1b[0m        Удалить пустые строки из кода (экономия токенов)");
  println!("  \x1b[1;32m-x, --xml\x1b[0m           Форматировать контекст в XML (отлично для Claude/DeepSeek)");
  println!("  \x1b[1;32m-e, --ext <exts>\x1b[0m    Фильтр по расширениям через запятую (напр. -e rs,toml)");
  println!("  \x1b[1;32m-h, --help\x1b[0m          Показать эту справку");
  println!();
  println!("\x1b[1;33mПРИМЕРЫ:\x1b[0m");
  println!("  gptcopy .                   # Весь текущий проект");
  println!("  gptcopy src/ -m -x          # Папка src, минификация, XML разметка");
  println!("  gptcopy . -e rs,toml        # Только файлы .rs и .toml");
  println!("  gptcopy -c                  # Только git diff");
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

// Простой чтец местного .gitignore
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

fn collect_files(dir: &Path, files: &mut Vec<PathBuf>, config: &Config, custom_ignores: &[String]) {
  if let Ok(entries) = fs::read_dir(dir) {
    for entry in entries.flatten() {
      let path = entry.path();

      if is_ignored(&path, custom_ignores) {
        continue;
      }

      if path.is_dir() {
        collect_files(&path, files, config, custom_ignores);
      } else if path.is_file()
        && is_text_file(&path)
        && matches_extension(&path, &config.extensions)
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

fn get_clipboard_process() -> Option<Command> {
  let term = env::var("TERM").unwrap_or_default();

  if term == "xterm-kitty" && Command::new("kitten").arg("--version").output().is_ok() {
    let mut cmd = Command::new("kitten");
    cmd.arg("clipboard");
    return Some(cmd);
  }

  if Command::new("wl-copy").arg("--version").output().is_ok() {
    return Some(Command::new("wl-copy"));
  }

  if Command::new("xclip").arg("-version").output().is_ok() {
    let mut cmd = Command::new("xclip");
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

fn main() {
  let args: Vec<String> = env::args().skip(1).collect();
  let mut config = Config {
    only_changed: false,
    minify: false,
    use_xml: false,
    extensions: Vec::new(),
    targets: Vec::new(),
  };

  let mut i = 0;
  while i < args.len() {
    match args[i].as_str() {
      "-c" | "--changed" => config.only_changed = true,
      "-m" | "--minify" => config.minify = true,
      "-x" | "--xml" => config.use_xml = true,
      "-e" | "--ext" => {
        if i + 1 < args.len() {
          i += 1;
          config.extensions = args[i]
            .split(',')
            .map(|s| s.trim().trim_start_matches('.').to_string())
            .collect();
        }
      }
      "-h" | "--help" => {
        show_help();
        return;
      }
      _ => {
        let raw_path = PathBuf::from(&args[i]);
        config.targets.push(expand_tilde(&raw_path));
      }
    }
    i += 1;
  }

  if config.targets.is_empty() {
    config.targets.push(PathBuf::from("."));
  }

  let root_target = &config.targets[0];
  let custom_ignores = load_gitignore(root_target);

  let mut output = String::new();

  // 1. СТРУКТУРА ПРОЕКТА
  if config.use_xml {
    output.push_str("<project_structure>\n");
  } else {
    output.push_str("=== PROJECT STRUCTURE ===\n");
  }

  if config.only_changed {
    let git_status = Command::new("git").args(["status", "-s"]).output();
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

  if config.use_xml {
    output.push_str("</project_structure>\n\n<source_code>\n");
  } else {
    output.push_str("\n=== SOURCE CODE ===\n\n");
  }

  // 2. СБОР ФАЙЛОВ
  let mut files_to_process: Vec<PathBuf> = Vec::new();

  if config.only_changed {
    let git_files = Command::new("git")
      .args(["ls-files", "-m", "-o", "--exclude-standard"])
      .output();

    if let Ok(out) = git_files {
      let stdout = String::from_utf8_lossy(&out.stdout);
      for line in stdout.lines() {
        let path = PathBuf::from(line);
        if !is_ignored(&path, &custom_ignores)
          && path.is_file()
          && matches_extension(&path, &config.extensions)
        {
          files_to_process.push(path);
        }
      }
    }
  } else {
    for target in &config.targets {
      if target.is_file() {
        if !is_ignored(target, &custom_ignores) && matches_extension(target, &config.extensions) {
          files_to_process.push(target.clone());
        }
      } else if target.is_dir() {
        collect_files(target, &mut files_to_process, &config, &custom_ignores);
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

      if config.use_xml {
        output.push_str(&format!("<file path=\"{}\">\n", file_path.display()));
      } else {
        output.push_str(&format!("FILE: {}\n```\n", file_path.display()));
      }

      if config.minify {
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

      if config.use_xml {
        output.push_str("</file>\n\n");
      } else {
        output.push_str("```\n\n");
      }
    }
  }

  if config.use_xml {
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