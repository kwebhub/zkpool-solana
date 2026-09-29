# 14. Finalization

> **Этап 14** проекта zkpool-solana.
> README, CHANGELOG, CONTRIBUTING, SECURITY, LICENSE, v0.1.0 release.

---

## TL;DR

Все функциональные этапы завершены. Stage 14 — упаковка проекта: документация для новых читателей, публичный релиз, финализация метаданных.

**Артефакты:**
- `.editorconfig` — единые настройки редактора.
- `LICENSE` (MIT).
- `README.md` — главная страница репозитория.
- `docs/ru/README.md` — русская версия.
- `CONTRIBUTING.md` — как контрибьютить.
- `SECURITY.md` — как сообщить об уязвимости.
- `CHANGELOG.md` — история версий.
- `docs/DEMO-NOTICE.md` — обновление.
- Git tag `v0.1.0` + GitHub Release.

---

## 14.1. `.editorconfig` + `LICENSE`

**Дата:** 2026-09-29
**Commit:** `843ac88`

### Зачем

**`.editorconfig`** — единые настройки для всех редакторов (VSCode, Vim, IntelliJ). Проект содержит Rust (4 пробела), TOML (2 пробела), Makefile (tab). Без `.editorconfig` каждый редактор решает по-своему.

**`LICENSE`** — MIT. Уже заявлен во всех `Cargo.toml` (`license = "MIT"`), `package.json`, но самого файла не было. GitHub показывает лицензию в about-секции только если файл есть.

### `.editorconfig`

```ini
root = true

[*]
charset = utf-8
end_of_line = lf
insert_final_newline = true
trim_trailing_whitespace = true
indent_style = space
indent_size = 2

[*.rs]
indent_size = 4

[*.md]
trim_trailing_whitespace = false

[Makefile]
indent_style = tab
indent_size = 4
```

**Почему Rust 4 пробела:** `rustfmt` по умолчанию использует 4.

**Почему Makefile tab:** обязательное требование `make`. Пробелы ломают правило (recipe lines должны начинаться с `\t`).

**Почему `*.md` без `trim_trailing_whitespace`:** trailing whitespace в Markdown — иногда значимый (два пробела в конце строки = `<br>`).

### `LICENSE`

MIT — стандартный текст. Copyright `kwebhub`, 2026.

**Почему MIT:**
- Уже заявлен в `Cargo.toml` / `package.json`.
- Соответствует origin (Solana Foundation Bootcamp).
- Максимально разрешительный — не блокирует никакое использование.

### Грабли

1. **`Makefile` использует tab, не пробелы.** VS Code / Vim могут автоматически конвертировать tab → spaces. `.editorconfig` фиксирует это.
2. **Trailing whitespace в `.md`.** Без исключения — `editorconfig`-aware редакторы удалят значимые пробелы (два в конце строки = `<br>` в Markdown).

### Уроки

1. **`.editorconfig` — необязательный, но дешёвый.** 30 строк, предотвращает случайные конфликты стиля.
2. **`LICENSE` без файла — не лицензия.** GitHub, автоматизированные тулы (SPDX, `cargo-deny`) — все ищут файл.
3. **MIT соответствует всем объявлениям.** Все `Cargo.toml` уже говорят MIT. Файл только делает это явным.

---

## 14.2. `README.md`

**Дата:** 2026-09-29
**Commit:** `5c60ca1`

### Зачем

GitHub показывает `README.md` на главной странице репозитория. Для портфолио это **первое, что видит читатель** — рекрутер, коллега, случайный посетитель.

Без README проект выглядит как заброшенный эксперимент. С README — как осмысленная работа.

### Структура

**9 секций:**

1. **Title + badges.** CI/security status, MIT.
2. **Demo warning.** ⚠️ Сразу говорит, что проект — не production.
3. **What it does.** 3 абзаца: что, как, зачем.
4. **Architecture.** ASCII-диаграмма + 5 принципов.
5. **Quick start.** `git clone && make up`.
6. **End-to-end demo.** Команды для deposit/withdraw.
7. **Repository layout.** Что где.
8. **Tests.** Таблица всех слоёв (139+ тестов).
9. **Security / Documentation / Origin / License.**

### Ключевые решения

**Badges вверху.** CI + security + license. Мгновенный статус.

**Demo warning отдельным блоком.** До того, как читатель углубится.

**ASCII-диаграмма.** Не картинка — текст. Копируется, рендерится в любом markdown, не гниёт.

**Makefile target'ы в quick start.** Единая точка входа. `make up` — и всё работает.

**End-to-end demo с реальными командами.** Не "как-нибудь запустите". `make exec-c CMD='cd /home/ubuntu/scripts/e2e-deposit && ./target/release/e2e-deposit 1_000_000'` — и вот результат.

**Tests таблица.** Показывает серьёзность проекта. 139+ тестов, разбивка по слоям.

**Threat model линк в Security-секции.** 12 атак, 5 protected / 3 partial / 4 not prevented — не скрываем ограничения.

**Origin секция.** Ссылка на bootcamp + v1/v2. Честная история.

### Грабли

1. **Badges требуют публичный репозиторий.** Для приватного — не рендерятся. Но CI уже будет.
2. **ASCII-диаграмма в 100+ строк.** Не всем нравится, но работает в любом markdown-редакторе без картинок.
3. **Числа тестов — могут устареть.** 139+ — округлённое. Точное число в `docs/PROJECT_CONTEXT.md`, ссылка в README ведёт туда.

### Уроки

1. **README — не список фич, а ответ на вопрос "зачем".** Первые 3 абзаца важнее всего остального.
2. **Demo warning до всего остального.** Иначе читатель начнёт с quick start и обидится, когда что-то не так.
3. **Reality в README соответствует `PROJECT_CONTEXT.md`.** Не выдумывать — цитировать реальные адреса, реальные команды.
4. **Без картинок.** ASCII-диаграмма вместо PNG. Не требует хостинга, рендерится везде.

### Связь с русским README (14.3)

Планируется `docs/ru/README.md` — перевод того же содержания. Основной README английский (стандарт GitHub), русский — для портфолио на русскоязычных платформах (Habr, Medium-ру, VC).

---

## 14.3. Русский README

**Дата:** 2026-09-29
**Commit:** `94d17b8`

### Зачем

Два разных назначения:
- **`README.md` (корень)** — английский. Стандарт GitHub, для международной аудитории.
- **`docs/ru/README.md`** — русский. Для портфолио на русскоязычных платформах: Habr, VC, Medium-ru, LinkedIn-ру.

**Ссылка сверху английского README** ведёт в русский, и наоборот.

### Что в русской версии

**Полный перевод** английского — 227 строк, те же 9 секций:
1. Title + badges + demo warning.
2. What it does.
3. Архитектура (ASCII-диаграмма).
4. Quick start.
5. E2E demo.
6. Repository layout.
7. Tests.
8. Security.
9. Origin.
10. Лицензия.

### Что НЕ переведено

- **Технические термины** — оставлены английскими: `commitment`, `nullifier`, `vault`, `PDA`, `Groth16`, `root`, `witness`, `proof`.
- **Пути к файлам** — в исходном виде: `docs/PROJECT_CONTEXT.md`, `services/backend/`.
- **Команды** — как есть: `make up`, `cargo test`, `pnpm`.
- **Названия сервисов** — `backend`, `prover`, `merkle`.

**Почему:** перевод технического термина (например, "обязательство" для `commitment`) ломает связь с кодом. Читатель должен видеть **то же слово**, что в исходниках.

### Форматирование

- **Ссылки относительные** — `../DEMO-NOTICE.md`, `../../LICENSE`. Работают и в GitHub UI, и в локальном редакторе.
- **Badges абсолютные** — URL на GitHub. Внутри `docs/ru/` бейджи рендерятся как обычно.
- **ASCII-диаграмма скопирована** — без перевода, потому что названия сервисов те же.

### Грабли

1. **Badges в подкаталоге.** Ссылки на workflow — абсолютные (`https://github.com/...`). Не ломаются.
2. **Относительные пути на два уровня вверх.** `../../LICENSE` — из `docs/ru/` в корень. Проверить при мерже.
3. **Русский длиннее английского в 1.3–1.5 раза.** Строка "Deposit into a shared vault" → "Депозит в общий vault" — примерно то же. Но абзацы разрастаются.

### Уроки

1. **Техническая терминология не переводится.** `Commitment` — это идентификатор, а не "обязательство". `Nullifier` — это идентификатор. Читатель кода узнаёт их мгновенно.
2. **Структура должна совпадать.** Читатель может переключаться между английской и русской версиями — порядок секций один.
3. **Демо-warning в обеих версиях.** Нельзя, чтобы русская версия звучала оптимистичнее английской.
4. **Ссылка из корневого README — обязательна.** Иначе русскую версию никто не найдёт.

---

## 14.4. `CONTRIBUTING.md`

**Дата:** 2026-09-29
**Commit:** `747a711`

### Зачем

Для одного разработчика `CONTRIBUTING.md` — формальность. Для публичного портфолио — сигнал: "проект структурирован, я знаю, что делаю".

GitHub автоматически показывает ссылку на этот файл при создании issue/PR.

### Структура

**9 секций:**

1. **Welcome + demo warning.** Сразу предупреждаем о статусе.
2. **Before you start.** Читать `PROJECT_CONTEXT.md` и `docs/notes/` перед кодом.
3. **Setup.** `git clone && make up`. Проверка через `make status`.
4. **Workflow.** Ветки, малые изменения, коммиты.
5. **Code style.** Rust, TypeScript/Vue, shell.
6. **Что мержится.** Три категории: welcome / discuss first / not accepted.
7. **Pitfalls.** Ссылка на §8 PROJECT_CONTEXT + топ-7 грабель.
8. **Security reporting.** Через `SECURITY.md`, не публичный issue.
9. **Getting help.** Куда идти с вопросом.
10. **License.** "By contributing you agree to MIT".

### Ключевые решения

**Conventional Commits с примерами.** Список типов (feat/fix/docs/...), список scopes (onchain/backend/prover/merkle/web/...), 4 примера.

**Три категории PR:**
- **Welcome:** bug fixes (с тестом), tests, docs, small features, CI.
- **Discuss first:** новые сервисы, изменения схем, изменение layout'а публичных входов, новые зависимости, большие рефакторинги.
- **Not accepted:** сломанный демо-flow, удаление тестов, удаление информации из docs, смена лицензии.

**Pitfalls — топ-7 из §8.** Компактный список для быстрого ознакомления. Полный — по ссылке.

### Грабли

1. **"Not accepted" секция может отпугнуть.** Но она защищает от типичных ошибок: "упрощу, удалив раздел". Явно перечислено, что не принимается — и почему.
2. **Ссылки на `PROJECT_CONTEXT.md`** — 3 раза. Это самая важная точка входа.
3. **Code style без жёстких правил.** "Follow existing patterns" — не формально, но практично.

### Уроки

1. **Contributing-гайд — не бюрократия.** Это способ масштабировать правила проекта без личного участия.
2. **"Discuss first"** важнее "not accepted". Позволяет не отвергать людей, а направлять.
3. **Conventional Commits — стандарт, не открытие.** Но без примеров многие не знают, какой scope использовать. Примеры снижают порог.
4. **Ссылка на threat-model в секции Security reporting.** Не "открой issue", а "прочти threat-model сначала" — многие вопросы уже отвечены.

---

## 14.5. `SECURITY.md`

**Дата:** 2026-09-29
**Commit:** `28553d4`

### Зачем

GitHub требует `SECURITY.md` для показа ссылки "Report a vulnerability" в Security-табе репозитория. Без него GitHub использует свою форму — не настраиваемую.

Для портфолио: наличие `SECURITY.md` = "автор задумывался о безопасности".

### Структура

**8 секций:**

1. **Supported versions.** Только `main`. Всё остальное — нет.
2. **⚠️ Important context.** Ссылки на `DEMO-NOTICE.md` и `threat-model.md`. **Ключевая секция.**
3. **How to report.** Через GitHub Security Advisories (private).
4. **Response timeline.** 3 / 7 / 14 / 90 дней. Best-effort.
5. **Scope.** In scope / out of scope / minimum bar.
6. **What we do.** Автоматизация + ручные меры.
7. **Disclosure policy.** Что мы будем / не будем делать.
8. **Bug bounty.** Нет. Только credit.

### Ключевые решения

**Список known limitations в топе файла.**

```
- A1 — new_root не проверяется
- A5 — phishing frontend
- A6 — prover видит witness
- A9 — commitment forgery
- A10 — ROOT_HISTORY_SIZE = 10
- Trusted setup без MPC
- Single-keypair upgrade authority
- Backend видит commitments + nullifiers
```

**Смысл:** исследователь сначала видит, что мы уже знаем. Если его находка **не** в списке — welcome. Если в списке — это discussion, не security report.

Это снимает нагрузку с мейнтейнера и одновременно даёт понять "мы не игнорируем проблемы, мы их документируем".

**Response timeline — best-effort.**

```
Acknowledgment:      3 business days
Initial assessment:  7 business days
Fix or mitigation:   14 business days
Public disclosure:   after fix, or 90 days
```

**Best-effort only, because solo-maintained.**

**Scope — три уровня.**

- **In scope:** наши компоненты (on-chain program, backend, prover, merkle, frontend, circuits, scripts).
- **Out of scope:** Solana L1, Anchor, Noir, Sunspot, третьи библиотеки — докладывать upstream.
- **Minimum bar:** не принимаем теоретические атаки без демонстрации, "best practice" замечания, DoS при обычной нагрузке, typos.

**Disclosure policy — "we will not / we will".**

- **Не будем:** судиться, требовать молчания, игнорировать.
- **Будем:** подтверждать, честно говорить "не можем пофиксить", давать credit.

### Грабли

1. **GitHub Security Advisories — единственный приватный канал.** Публичный issue для уязвимости — плохо. `config.yml` в `ISSUE_TEMPLATE/` уже ссылается на этот URL.
2. **Threat model уже описывает 4 "not prevented" атаки.** Их перечисление в SECURITY.md — не "список багов", а "список известных ограничений". Явно это указано.
3. **`v0.1.0` ещё нет.** Таблица "Supported versions" помечает `main` как supported, `< 0.1.0` — нет. После тега — обновить.

### Уроки

1. **`SECURITY.md` — про доверие.** Чем яснее политика, тем серьёзнее выглядит проект. Даже если это демо.
2. **Known limitations в топе — защита от noise.** Исследователь не потратит 3 дня на анализ A1, если сразу видит "известно, документировано".
3. **"We will not sue you" — не пустая фраза.** В некоторых юрисдикциях такое явное заявление снимает юридические риски с исследователя.
4. **Bug bounty — не обязанность.** Отсутствие bounty явно обозначено. Исследователи выбирают проект по интересу, не по деньгам.
5. **Response timeline — best-effort.** Обещать 3 дня в соло-проекте — обман. Честное "best-effort" лучше.

---

## 14.6. `CHANGELOG.md`

**Дата:** 2026-09-29
**Commit:** `537ea0d`

### Зачем

История изменений для внешних читателей. GitHub показывает `CHANGELOG.md` в разделе Releases, автоматически парсит для release notes.

### Формат

**Keep a Changelog 1.1.0** — стандарт. Секции:
- `Added` — новые фичи.
- `Changed` — изменения в существующем.
- `Deprecated` — что будет удалено.
- `Removed` — что удалено.
- `Fixed` — багфиксы.
- `Security` — уязвимости (RUSTSEC, CVE).

**Semantic Versioning** — `[0.1.0]`.

### Структура релиза 0.1.0

**Added** — по этапам:
- Core protocol (Stages 0–4): circuits, verifier, Anchor program, scripts.
- Services (Stages 5–7): backend, merkle, prover.
- On-chain pool deployment (Stage 8).
- Frontend (Stage 9).
- E2E demo (Stage 10).
- Infrastructure (Stage 11): Makefile, Prometheus, Grafana.
- Engineering (Stage 12): CI/CD, templates.
- Security (Stage 13): threat model, adversarial tests, hardening.

**Security:**
- `RUSTSEC-2024-0363` — sqlx upgrade.
- BN254 mask fix.

**Fixed** — 8 строк:
- Borsh length prefix.
- `NULLIFIER_RECORD_SEED`.
- Compute budget.
- Codama signer.
- `--experimental-strip-types`.
- pnpm allowBuilds.
- Stale ACIRs.

**Known limitations** — 7 строк, ссылка на threat model.

**Statistics:**
- ~140 тестов.
- 4 workflow.
- 5 контейнеров.
- 4 deployed programs/PDAs.
- 2 E2E цикла.

### How this release was built

Раздел про stages 0–14 + описание системы чекпоинтов.

### Грабли

1. **Не даты, а релизы.** `[Unreleased]` — что в main, но не в релизе. `[0.1.0] — 2026-09-29` — тег.
2. **Ссылки на сравнения внизу:**
   ```markdown
   [Unreleased]: https://github.com/.../compare/v0.1.0...HEAD
   [0.1.0]: https://github.com/.../releases/tag/v0.1.0
   ```
   Без них `[Unreleased]` и `[0.1.0]` не станут ссылками.
3. **Порядок секций фиксирован.** Keep a Changelog явно требует: Added / Changed / Deprecated / Removed / Fixed / Security.

### Уроки

1. **CHANGELOG ≠ git log.** Git log — хронология разработки (включая "docs: record"). CHANGELOG — что изменилось для пользователя.
2. **Пропускать "docs: record" коммиты.** Они не в changelog. Включать только `feat`, `fix`, `chore` (значимые), `security`.
3. **Один большой раздел Added для v0.1.0.** Разбивка по этапам — даёт читателю понять scope работы.
4. **Known limitations в CHANGELOG — необычно, но правильно.** Для демо-проекта прозрачность важнее формата.
5. **Statistics внизу.** Быстрые цифры для читателя, который не хочет читать всё.

---

## Что дальше

- **14.7** — обновление `docs/DEMO-NOTICE.md` (устарел с Stage 5).
- **14.8** — root `deny.toml` + strict security workflow.
- **14.9** — тег `v0.1.0` + release.
- **14.10** — финальный чекпоинт.
