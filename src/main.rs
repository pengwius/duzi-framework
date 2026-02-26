use clap::{Parser, ValueEnum};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum Framework {
    Maui,
    Xamarin,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum Mode {
    Xaml,
    Csharp,
}

#[derive(Parser, Debug)]
#[command(name = "duzi-framework")]
#[command(about = "Xamarin w JSONie z elementami pajtona", long_about = None)]
struct Args {
    #[arg(short, long)]
    input: String,

    #[arg(short, long)]
    output: String,

    #[arg(short, long, value_enum, default_value_t = Framework::Maui)]
    framework: Framework,

    #[arg(short, long, value_enum, default_value_t = Mode::Xaml)]
    mode: Mode,
}

#[derive(Deserialize, Debug)]
struct Element {
    #[serde(rename = "type")]
    tag: String,

    #[serde(default)]
    children: Vec<Element>,

    #[serde(default)]
    content: Option<String>,

    #[serde(flatten)]
    properties: BTreeMap<String, serde_json::Value>,
}

fn escape_xml(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn generate_xaml(element: &Element, indent: usize, is_root: bool, framework: Framework) -> String {
    let indent_str = "    ".repeat(indent);
    let mut xaml = format!("{}<{}", indent_str, element.tag);

    let mut props = element.properties.clone();

    if is_root {
        if !props.contains_key("xmlns") {
            let xmlns = match framework {
                Framework::Maui => "http://schemas.microsoft.com/dotnet/2021/maui",
                Framework::Xamarin => "http://xamarin.com/schemas/2014/forms",
            };
            props.insert(
                "xmlns".to_string(),
                serde_json::Value::String(xmlns.to_string()),
            );
        }
        if !props.contains_key("xmlns:x") {
            props.insert(
                "xmlns:x".to_string(),
                serde_json::Value::String(
                    "http://schemas.microsoft.com/winfx/2009/xaml".to_string(),
                ),
            );
        }
    }

    for (key, value) in &props {
        let val_str = match value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            _ => continue,
        };
        xaml.push_str(&format!(" {}=\"{}\"", key, escape_xml(&val_str)));
    }

    if element.children.is_empty() && element.content.is_none() {
        xaml.push_str(" />\n");
    } else {
        xaml.push_str(">\n");

        if let Some(content) = &element.content {
            xaml.push_str(&format!("{}    {}\n", indent_str, escape_xml(content)));
        }

        for child in &element.children {
            xaml.push_str(&generate_xaml(child, indent + 1, false, framework));
        }

        xaml.push_str(&format!("{}</{}>\n", indent_str, element.tag));
    }

    xaml
}

fn clean_args(args: &str) -> String {
    if args == "()" || args.trim().is_empty() {
        return "()".to_string();
    }
    let inner = args.trim_start_matches('(').trim_end_matches(')');
    if inner.trim().is_empty() {
        return "()".to_string();
    }

    let mut csharp_args = Vec::new();
    for arg in inner.split(',') {
        let arg = arg.trim();
        if arg.is_empty() {
            continue;
        }
        if arg.contains(':') {
            let parts: Vec<&str> = arg.split(':').collect();
            let name = parts[0].trim();
            let py_type = parts[1].trim();
            let cs_type = match py_type {
                "str" => "string",
                "int" => "int",
                "bool" => "bool",
                "float" => "double",
                _ => "object",
            };
            csharp_args.push(format!("{} {}", cs_type, name));
        } else {
            csharp_args.push(format!("object {}", arg));
        }
    }
    format!("({})", csharp_args.join(", "))
}

fn transpile_python_to_csharp(python_code: &str) -> String {
    let mut csharp_code = String::new();
    csharp_code
        .push_str("using System;\nusing System.Collections.Generic;\nusing System.Linq;\n\n");

    let mut indent_stack: Vec<usize> = vec![0];
    let mut current_class = String::new();

    for line in python_code.lines() {
        if line.trim().is_empty() {
            csharp_code.push('\n');
            continue;
        }

        let indent = line.chars().take_while(|c| *c == ' ' || *c == '\t').count();
        let trimmed = line.trim();

        while indent_stack.len() > 1 && indent < *indent_stack.last().unwrap() {
            indent_stack.pop();
            let spaces = " ".repeat(*indent_stack.last().unwrap());
            csharp_code.push_str(&format!("{}}}\n", spaces));
        }

        if indent > *indent_stack.last().unwrap() {
            indent_stack.push(indent);
        }

        let spaces = " ".repeat(indent);
        let out_line;

        if trimmed.starts_with("class ") {
            let class_name = trimmed[6..].trim_end_matches(':').trim();
            current_class = class_name.to_string();
            out_line = format!("public partial class {} {{", class_name);
        } else if trimmed.starts_with("def ") {
            let sig = trimmed[4..].trim_end_matches(':').trim();
            if sig.starts_with("__init__") {
                let args = sig[8..]
                    .replace("self,", "")
                    .replace("self", "")
                    .trim()
                    .to_string();
                let args = clean_args(&args);
                out_line = format!("public {}{} {{", current_class, args);
            } else {
                let parts: Vec<&str> = sig.splitn(2, '(').collect();
                let name = parts[0];
                let args = if parts.len() > 1 {
                    format!("({}", parts[1])
                } else {
                    "()".to_string()
                };
                let args = args.replace("self,", "").replace("self", "");
                let args = clean_args(&args);
                out_line = format!("public void {}{} {{", name, args);
            }
        } else if trimmed.starts_with("import ") || trimmed.starts_with("from ") {
            out_line = format!("// {}", trimmed);
        } else if trimmed == "pass" {
            out_line = "// pass".to_string();
        } else {
            let mut stmt = trimmed.to_string();

            stmt = stmt.replace("self.", "this.");
            stmt = stmt.replace("print(", "Console.WriteLine(");
            stmt = stmt
                .replace("True", "true")
                .replace("False", "false")
                .replace("None", "null");
            stmt = stmt
                .replace(" and ", " && ")
                .replace(" or ", " || ")
                .replace(" not ", " !");

            if stmt.contains("f\"") {
                stmt = stmt.replace("f\"", "$\"");
            }

            if stmt.ends_with(':') {
                stmt = stmt.trim_end_matches(':').trim().to_string();
                stmt = stmt
                    .replace("elif ", "else if (")
                    .replace("if ", "if (")
                    .replace("while ", "while (")
                    .replace("for ", "foreach (var ");

                if stmt.starts_with("if (")
                    || stmt.starts_with("else if (")
                    || stmt.starts_with("while (")
                {
                    stmt.push_str(") {");
                } else if stmt.starts_with("foreach (") {
                    stmt = stmt.replace(" in ", " in ");
                    stmt.push_str(") {");
                } else if stmt == "else" {
                    stmt = "else {".to_string();
                } else {
                    stmt.push_str(" {");
                }
            } else {
                if !stmt.ends_with(';') && !stmt.ends_with('{') && !stmt.ends_with('}') {
                    stmt.push(';');
                }
            }
            out_line = stmt;
        }

        csharp_code.push_str(&format!("{}{}\n", spaces, out_line));
    }

    while indent_stack.len() > 1 {
        indent_stack.pop();
        let spaces = " ".repeat(*indent_stack.last().unwrap());
        csharp_code.push_str(&format!("{}}}\n", spaces));
    }

    csharp_code
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.mode {
        Mode::Xaml => {
            println!("Tryb XAML: Wczytywanie pliku JSON: {}", args.input);
            let json_content = fs::read_to_string(&args.input)?;
            let root_element: Element = serde_json::from_str(&json_content)?;

            println!("Generowanie kodu XAML dla frameworka: {:?}", args.framework);
            let mut final_xaml = String::from("<?xml version=\"1.0\" encoding=\"utf-8\" ?>\n");
            final_xaml.push_str(&generate_xaml(&root_element, 0, true, args.framework));

            let mut output_file = fs::File::create(&args.output)?;
            output_file.write_all(final_xaml.as_bytes())?;
            println!(
                "Sukces! Zapisano wygenerowany XAML do pliku: {}",
                args.output
            );
        }
        Mode::Csharp => {
            println!("Tryb C#: Wczytywanie pliku Python: {}", args.input);
            let python_content = fs::read_to_string(&args.input)?;

            println!("Transpilowanie logiki Python do C#...");
            let csharp_code = transpile_python_to_csharp(&python_content);

            let mut output_file = fs::File::create(&args.output)?;
            output_file.write_all(csharp_code.as_bytes())?;
            println!(
                "Sukces! Zapisano wygenerowany kod C# do pliku: {}",
                args.output
            );
        }
    }

    Ok(())
}
