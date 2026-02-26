# DUŹI Framework

DUŹI Framework to narzędzie CLI napisane w języku Rust, które pozwala na wygodne pisanie interfejsów użytkownika dla aplikacji **MAUI** oraz **Xamarin** przy użyciu czystego i ustrukturyzowanego formatu **JSON**. Program automatycznie parsuje plik JSON i generuje gotowy do użycia kod XAML.

## Funkcje
- 🚀 **Szybkość i prostota**: Pisz UI w czytelnym formacie JSON zamiast rozwlekłego XAML-a.
- 🔄 **Wsparcie dla MAUI i Xamarin**: Wybierz docelowy framework, a narzędzie automatycznie dobierze odpowiednie przestrzenie nazw (namespaces).
- 🌳 **Zagnieżdżanie elementów**: Łatwe budowanie drzewa widoków dzięki tablicy `children`.
- ⚙️ **Bindowanie i atrybuty**: Wszystkie niestandardowe pola w JSON są automatycznie mapowane na atrybuty XAML.

## Wymagania
- [Rust](https://www.rust-lang.org/) (cargo)
- Opcjonalnie: [Nix](https://nixos.org/) (projekt zawiera `flake.nix` ze skonfigurowanym środowiskiem)

## Budowanie i uruchamianie

Jeśli używasz Nix-a, wejdź do środowiska deweloperskiego:
```bash
nix develop
```

Zbuduj projekt za pomocą Cargo:
```bash
cargo build --release
```

## Użycie

```bash
cargo run -- --input <PLIK_WEJSCIOWY.json> --output <PLIK_WYNIKOWY.xaml> [--framework <maui|xamarin>]
```

### Parametry:
- `-i, --input <FILE>`: Ścieżka do wejściowego pliku JSON.
- `-o, --output <FILE>`: Ścieżka do wyjściowego pliku XAML.
- `-f, --framework <FRAMEWORK>`: Docelowy framework. Możliwe wartości to `maui` (domyślnie) lub `xamarin`.

## Struktura pliku JSON

Każdy element w pliku JSON to obiekt, który może zawierać następujące specjalne pola:
- `"type"` *(wymagane)*: Nazwa kontrolki/tagu XAML (np. `"ContentPage"`, `"StackLayout"`, `"Label"`).
- `"children"` *(opcjonalne)*: Tablica obiektów reprezentujących dzieci danego elementu.
- `"content"` *(opcjonalne)*: Tekst, który ma się znaleźć wewnątrz tagu (np. `<Label>Tekst</Label>`).

**Wszystkie pozostałe pola** zostaną potraktowane jako atrybuty XAML (np. `"Text"`, `"BackgroundColor"`, `"x:Class"`).

### Przykład

**Wejście (`widok.json`):**
```json
{
  "type": "ContentPage",
  "x:Class": "MyApp.MainPage",
  "Title": "Strona Główna",
  "BackgroundColor": "White",
  "children": [
    {
      "type": "StackLayout",
      "Padding": "20",
      "Spacing": 15,
      "children": [
        {
          "type": "Label",
          "Text": "Witaj w MAUI z JSONa!",
          "FontSize": 24,
          "HorizontalOptions": "Center"
        },
        {
          "type": "Button",
          "Text": "Kliknij mnie",
          "Command": "{Binding ClickCommand}",
          "BackgroundColor": "Blue",
          "TextColor": "White"
        }
      ]
    }
  ]
}
```

**Generowanie:**
```bash
cargo run -- -i widok.json -o widok.xaml -f maui
```

**Wyjście (`widok.xaml`):**
```xml
<?xml version="1.0" encoding="utf-8" ?>
<ContentPage BackgroundColor="White" Title="Strona Główna" x:Class="MyApp.MainPage" xmlns="http://schemas.microsoft.com/dotnet/2021/maui" xmlns:x="http://schemas.microsoft.com/winfx/2009/xaml">
    <StackLayout Padding="20" Spacing="15">
        <Label FontSize="24" HorizontalOptions="Center" Text="Witaj w MAUI z JSONa!" />
        <Button BackgroundColor="Blue" Command="{Binding ClickCommand}" Text="Kliknij mnie" TextColor="White" />
    </StackLayout>
</ContentPage>
```
