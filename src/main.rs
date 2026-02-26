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

#[derive(Parser, Debug)]
#[command(name = "duzi-framework")]
#[command(about = "Xamarin (albo MAUI) w JSONie", long_about = None)]
struct Args {
    #[arg(short, long)]
    input: String,

    #[arg(short, long)]
    output: String,

    #[arg(short, long, value_enum, default_value_t = Framework::Maui)]
    framework: Framework,
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
                serde_json::Value::String("http://schemas.microsoft.com/winfx/2009/xaml".to_string()),
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("Wczytywanie pliku JSON: {}", args.input);
    let json_content = fs::read_to_string(&args.input)?;

    let root_element: Element = serde_json::from_str(&json_content)?;

    println!("Generowanie kodu XAML dla frameworka: {:?}", args.framework);

    let mut final_xaml = String::from("<?xml version=\"1.0\" encoding=\"utf-8\" ?>\n");
    final_xaml.push_str(&generate_xaml(&root_element, 0, true, args.framework));

    let mut output_file = fs::File::create(&args.output)?;
    output_file.write_all(final_xaml.as_bytes())?;

    println!("Sukces! Zapisano wygenerowany XAML do pliku: {}", args.output);

    Ok(())
}
