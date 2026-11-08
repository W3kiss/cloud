# Setup Hub

Локальная мастерская для быстрой настройки Windows после переустановки:
каталог приложений, установка через winget и прямые ссылки, твики системы и
восстановление профилей AppData.

Стек: **Tauri 2 + Rust + SvelteKit 2 (Svelte 5 runes) + TypeScript**.

---

## Требования

- Windows 10/11 (x64);
- Node.js 20+ и npm;
- Rust stable (MSVC toolchain) + Visual Studio Build Tools с «Desktop development with C++»;
- WebView2 Runtime (в Windows 11 уже установлен).

## Запуск в разработке

```powershell
npm install
npm run tauri dev
```

Проверка типов и предупреждений Svelte:

```powershell
npm run check
```

Проверка Rust:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

Тесты `catalog.rs` дополнительно проверяют, что замаскированный каталог
корректно расшифровывается и что все URL используют HTTPS.

## Сборка релиза

```powershell
npm run tauri build
```

Готовые установщики появятся в `src-tauri/target/release/bundle/`:
NSIS (`nsis/*.exe`) и MSI (`msi/*.msi`). Только исполняемый файл без
установщика:

```powershell
npm run tauri build -- --no-bundle
# результат: src-tauri/target/release/setup-hub.exe
```

### Профиль релиза

В `src-tauri/Cargo.toml`:

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
incremental = false
strip = "symbols"
```

`strip = "symbols"` — нативная замена ручному `strip --strip-all`: Cargo
убирает таблицу символов ещё при компоновке, поэтому установщик уже содержит
очищенный бинарник.

### Ручная очистка символов

Если нужно гарантированно обработать готовый `.exe` (например, профиль
временно меняли), используйте один из вариантов.

Скрипт (сохраняет копию `setup-hub.exe.unstripped`):

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\strip-release.ps1
```

Вручную через LLVM (рекомендуется):

```powershell
rustup component add llvm-tools-preview
$strip = "$env:USERPROFILE\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib\rustlib\x86_64-pc-windows-msvc\bin\llvm-strip.exe"
& $strip --strip-all "src-tauri\target\release\setup-hub.exe"
```

Альтернатива без `llvm-tools-preview` — `rust-objcopy` из стандартного
тулчейна:

```powershell
$objcopy = "$env:USERPROFILE\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib\rustlib\x86_64-pc-windows-msvc\bin\rust-objcopy.exe"
& $objcopy --strip-all "src-tauri\target\release\setup-hub.exe"
```

> После ручной очистки подписывать/упаковывать бинарник нужно заново:
> `npm run tauri build` пересоберёт его и снова применит `strip = "symbols"`.

### Проверка маскировки каталога

После релизной сборки строки каталога не должны встречаться в бинарнике:

```powershell
node -e "const fs=require('fs');const b=fs.readFileSync('src-tauri/target/release/setup-hub.exe');for(const n of ['RARLab.WinRAR','EpicGamesLauncher','downloads.example.com','gog-statics']){const i=b.indexOf(Buffer.from(n));console.log(n, i===-1?'MASKED':'FOUND at '+i)}"
```

---

## Экран первого запуска

При первом старте показывается обязательный дисклеймер с размытием фона
(`backdrop-filter`). Кнопки:

- **Выйти** — вызывает Rust-команду `exit_app` (`app.exit(0)`) и завершает процесс;
- **Я согласен** — сохраняет флаг в `localStorage` под ключом
  `setup-hub.disclaimer.accepted.v1` и больше не показывает экран.

Сбросить согласие для проверки:

```js
// DevTools (F12) в окне приложения
localStorage.removeItem("setup-hub.disclaimer.accepted.v1");
location.reload();
```

## Каталог приложений

Определения лежат в `src-tauri/catalog.json`. Категории:

| Категория    | Содержимое                                                        |
| ------------ | ----------------------------------------------------------------- |
| `system`     | WinRAR, Cursor, 7-Zip, VS Code, Git, Happ                       |
| `browsers`   | Chrome, Discord, Telegram, Firefox                                |
| `gaming`     | Steam, Prism Launcher + шаблоны Epic Games, GOG GALAXY, EA app     |
| `production` | VLC + шаблоны DaVinci Resolve, OBS Studio, Adobe Creative Cloud    |

Поля записи:

```json
{
  "id": "epic-games",
  "name": "Epic Games Launcher",
  "summary": "Шаблон лаунчера с прямой ссылкой",
  "category": "gaming",
  "kind": "download",
  "downloadUrl": "https://example.com/installer.msi",
  "installerArgs": [],
  "customConfig": { "installDir": "C:\\Program Files\\Epic Games" },
  "placeholder": true,
  "sha256": null
}
```

- `kind: "winget"` — установка по `wingetId` в скрытом режиме;
- `kind: "download" | "custom"` — загрузка по HTTPS с прогрессом, проверка
  SHA-256 (если задан), запуск `.exe/.msi/.msix/.appx`;
- `placeholder: true` — шаблон: backend отказывает в установке, UI показывает
  окно с параметрами;
- `customConfig` передаётся установщику в env-переменной
  `SETUP_HUB_APP_CONFIG` как JSON.

Перед установкой шаблона укажите реальный `downloadUrl`, при необходимости
`installerArgs`/`sha256` и установите `"placeholder": false`.

### Выбор и очередь установки

Каталог работает в два шага:

1. кнопка **«Выбрать»** на карточке добавляет приложение в локальный список;
2. кнопка **«Скачать выбранное (N)»** в нижней панели запускает
   последовательную установку очереди.

Во время очереди карточки блокируются, для текущего приложения показываются
фаза и прогресс, а общая полоса в доке отражает долю завершённых шагов.
Шаблоны (`placeholder: true`) не участвуют в выборе — они по-прежнему
открывают окно с параметрами. Уведомления автоматически поднимаются над
доком, чтобы не перекрывать прогресс.

Логотипы приложений лежат в `static/logos` и сопоставляются с ID в
`src/lib/logos.ts`; для записей без логотипа карточка показывает инициалы.

## Windows Tweaks

Отдельная вкладка боковой панели, четыре модуля:

1. **Конфиденциальность системы** — политика `AllowTelemetry = 0`, службы
   `DiagTrack` и `dmwappushservice` останавливаются (нужен UAC).
2. **Интерфейс проводника** — классическое контекстное меню Windows 11 через
   CLSID-override в HKCU; после включения доступна кнопка перезапуска
   Проводника.
3. **Оптимизация автозапуска** — удаляет записи OneDrive, Teams, Edge
   autolaunch, Spotify из `Run` и ярлыки из Startup. Действие необратимо.
4. **Дополнительные утилиты** — безопасный хук: запускает
   `%APPDATA%\SetupHub\scripts\custom.ps1`, только если файл начинается с
   маркера `# SETUP-HUB-CUSTOM-HOOK` и не превышает 256 КиБ.

Все команды выполняются асинхронно (`spawn_blocking`) через PowerShell в
скрытом окне (`CREATE_NO_WINDOW`, `-WindowStyle Hidden`). Ошибки доступа
преобразуются в `admin_required`, UI предлагает перезапуск с UAC.

## Маскировка каталога

`src-tauri/build.rs` при компиляции:

1. читает `catalog.json`;
2. сжимает его gzip (`flate2`, максимальный уровень);
3. накладывает позиционный XOR + rotation маску;
4. записывает `catalog.masked` в `OUT_DIR`.

В бинарник попадает только `include_bytes!(OUT_DIR/catalog.masked)`; исходный
JSON и открытые URL не линкуются. `src-tauri/src/protected.rs` собирает ключ из
четырёх фрагментов и распаковывает каталог в памяти при вызове
`bundled_catalog`.

Важно: это **обфускация**, а не криптографическая защита. Она усложняет
извлечение строк обычными сканерами, но не делает reverse engineering
невозможным. Реальные секреты (токены, приватные ключи) в каталог класть
нельзя.

## Структура проекта

```
src-tauri/
  build.rs              # gzip + маскировка каталога на этапе сборки
  catalog.json          # 19 записей, 4 категории, шаблоны
  src/
    lib.rs              # регистрация Tauri-команд
    catalog.rs          # загрузка, валидация, поиск
    protected.rs        # распаковка замаскированного каталога
    installer.rs        # winget / download / custom, прогресс, SHA-256
    tweaks.rs           # четыре твика, скрытый PowerShell, UAC
    backup.rs           # восстановление папок AppData
src/
  routes/+page.svelte   # каркас, вкладки, выбор, очередь, тосты
  lib/api.ts            # типы и обёртки invoke
  lib/logos.ts          # сопоставление ID приложений и локальных SVG
  lib/components/       # Sidebar, SoftwareHub, AppCard, TweaksPanel...
static/
  logos/                # локальные монохромные логотипы приложений
scripts/
  strip-release.ps1     # финальная очистка символов
```

## Безопасность

- все загрузки только по HTTPS, редиректы на HTTP отклоняются;
- лимит загрузки — 3 ГиБ, допускаются только `.exe/.msi/.msix/.appx`;
- ID winget и URL валидируются на backend;
- восстановление AppData блокирует системные папки и пересекающиеся пути;
- приложение не отправляет телеметрию; сеть используется только для
  выбранных пользователем установщиков.
