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

## 14.7. Обновление `docs/DEMO-NOTICE.md`

**Дата:** 2026-09-29
**Commit:** `f8b3adc`

### Зачем

`DEMO-NOTICE.md` был написан перед Stage 5 (2026-09-24) и устарел:

1. **§4.1 "Secrets in `localStorage`"** — **неверно.** Stage 13.4 подтвердил: notes не сохраняются в браузере.
2. **§5.1 "No CI/CD hardening — workflows will be added at Stage 10"** — Stage 12 добавил 4 workflow.
3. **§7 "planned, Stages 4.5/5–8/11"** — всё сделано.
4. **Отсутствовали ссылки на threat-model.** 12 сценариев не упомянуты.
5. **§3.1 (prover)** — говорилось "race condition", но мы уже добавили mutex.

### Что изменилось

**Добавлено сверху:**
```markdown
**See also:**
- [`threat-model.md`](threat-model.md) — 12 attack scenarios (A1–A12), 7 invariants.
- [`SECURITY.md`](../SECURITY.md) — how to report vulnerabilities.
- [`CHANGELOG.md`](../CHANGELOG.md) — release history.
```

**§2 (On-chain)** — добавлены A1, A9, A10 как ссылки на threat-model. Раньше были просто "known limitations".

**§3.1 (prover)** — теперь:
- "serialized by an async mutex" (вместо "race condition").
- "Verified in Stage 7.4."

**§3.2 (prover sees witness)** — добавлена ссылка A6. Усилено: "Client-side proving is the only architectural fix; a TEE is a mitigation, not a solution."

**§4.1** — **полностью переписан.**
- Было: "Secrets in `localStorage` in plaintext".
- Стало: "Notes are not stored in the browser. Positive: no XSS surface. Negative: closing browser without saving = loss of funds."

**§5.1 (CI/CD)** — теперь список 4 workflow с описанием. "What is missing: SHA pinning, OIDC, artifact signing".

**§7 (What IS in the project)** — переписан. Полный список по компонентам, ~140 тестов, ссылка на CHANGELOG.

**§8 (Bottom line)** — 8 шагов для production, со ссылками на threat-model ID.

### Ключевые изменения в философии

**Раньше:** "всё плохо, ничего нет".
**Теперь:** "вот что есть, вот чего нет, вот рекомендации".

**Почему это лучше:**

- **Читатель видит объём работы.** Не 30 строк "чего не хватает", а полный список компонентов с тестами.
- **Ссылки на threat-model.** Каждое ограничение = конкретный сценарий атаки.
- **Рекомендации для production** — actionable. Не "улучшить безопасность", а "increase ROOT_HISTORY_SIZE".

### Грабли

1. **`localStorage` — было неверно.** Писали "secrets in localStorage", но в коде их никогда не было. Stage 13.4 явно проверил. Документ надо периодически сверять с кодом.
2. **"Stage 10" в старом тексте.** Ссылался на будущее. Теперь — "Stage 12" в прошлом времени.
3. **Ссылки `../SECURITY.md`** — из `docs/` в корень. Работают.

### Уроки

1. **DEMO-NOTICE — живой документ.** Обновлять после каждого значимого этапа. Иначе становится вредным (вводит в заблуждение).
2. **Threat-model — естественное расширение DEMO-NOTICE.** DEMO-NOTICE — что не сделано простым языком. Threat-model — формализация.
3. **Ссылки на ID атак (A1, A6, A10) — сильный приём.** Читатель может пойти в threat-model и увидеть детали, если хочет.
4. **"Positive/Negative" для неоднозначных решений.** "Нет localStorage" — плюс для безопасности, минус для UX. Явно указать оба.

---

## 14.8. `cargo-deny` наконец проходит

**Дата:** 2026-09-29
**Commit:** `e09e8fd`

### Что было

На протяжении Stage 13.5 и 14.8 `cargo deny check` падал с `licenses FAILED`. Что только не пробовали:
- Перебирали SPDX-выражения в allow-списке (20+ штук).
- Добавляли `version = 2`, убирали, снова добавляли.
- Пытались `[licenses.private] ignore = true` и `private = { ignore = true }`.
- Копировали конфиг из v2 проекта.

Результат всегда одинаковый: `305 errors` — каждая зависимость отвергалась, включая `MIT OR Apache-2.0`.

### Три реальные причины

**1. Корневой `deny.toml` не был смонтирован в контейнер.**

Volume'ы в `docker-compose.yml` — по подпапкам (`../onchain:/home/ubuntu/onchain`, и т.д.). Корень репозитория не смонтирован. Значит `/home/ubuntu/deny.toml` — не существует. При запуске `cargo deny --config /home/ubuntu/deny.toml`:

```
[WARN] config path '/home/ubuntu/deny.toml' doesn't exist, falling back to default config
```

Cargo-deny молча берёт **пустой дефолтный конфиг** (deny all). Отсюда 305 ошибок.

**Диагностика:** `head -3` вывода. До этого смотрели только `tail -5`, где warning не виден.

**Фикс:** добавили в `docker-compose.yml`:
```yaml
- ../deny.toml:/home/ubuntu/deny.toml:ro
```

**2. Порядок флагов `cargo deny --config` и подкоманды.**

Правильно:
```
cargo deny --config /home/ubuntu/deny.toml check licenses
```

Неправильно (то, что пробовали сначала):
```
cargo deny check licenses --config /home/ubuntu/deny.toml
```

**3. `deny.toml` от v2 проекта.** Схема `version = 2` в `[licenses]` и `[advisories]` устарела в 0.20.2. Но главное — **`allow-osi-fsf-free` больше нет**: каждый SPDX-идентификатор нужно перечислять явно.

**Однако** — если наш собственный crate исключён через `private = { ignore = true }`, а третьесторонние crates содержат только простые ID (`MIT`, `Apache-2.0`, `ISC`, `Unicode-3.0` и т.д.), то **сложных compound-выражений в allow-списке не нужно**. Простой список из 17 ID справляется.

### Итоговый `deny.toml`

```toml
[graph]
targets = [{ triple = "x86_64-unknown-linux-gnu" }]

[licenses]
confidence-threshold = 0.8
private = { ignore = true }
allow = ["0BSD", "Apache-2.0", "Apache-2.0 WITH LLVM-exception", "BSD-2-Clause",
         "BSD-3-Clause", "BSL-1.0", "CC0-1.0", "CDLA-Permissive-2.0", "ISC",
         "MIT", "MIT-0", "MPL-2.0", "OpenSSL", "Unicode-3.0", "Unicode-DFS-2016",
         "Unlicense", "Zlib"]

[advisories]
yanked = "warn"

[bans]
multiple-versions = "warn"
wildcards = "deny"

[sources]
unknown-registry = "warn"
unknown-git = "warn"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
allow-git = []
```

### Бонусный баг: `cargo-deny` пропадает при рекрейте контейнера

`cargo install cargo-deny` пишет в `/home/ubuntu/.cargo/bin/` — **внутри контейнера, не в volume**. При `docker compose up -d --force-recreate solana` — бинарь теряется. Приходится переустанавливать (`cargo install cargo-deny --locked --version 0.20.2` — 1m 41s).

**TODO:** добавить `cargo-deny` и `cargo-audit` в `infra/docker/Dockerfile.solana`. Тогда они сохранятся в образе.

### Итог

```
advisories ok, bans ok, licenses ok, sources ok
```

### Грабли

1. **`head` вместо `tail` для диагностики конфига.** `tail -5` показал только дерево зависимостей. Warning о ненайденном конфиге был в первых строках.
2. **Volume mount только для подпапок.** Корневые файлы (deny.toml, Makefile, README.md) не видны в контейнере.
3. **`private = { ignore = true }` — inline table, не секция `[licenses.private]`.** Разные синтаксисы, работающие по-разному.
4. **Порядок флагов в clap-приложениях: глобальные до подкоманды.**

### Уроки

1. **Всегда смотреть полный вывод ошибки.** Warning в начале — критичен. `tail -5` систематически вводил в заблуждение.
2. **Проверять, что файл виден в контейнере.** `ls /home/ubuntu/deny.toml` перед запуском — мгновенная диагностика.
3. **v2-конфиг из прошлого проекта — не истина в последней инстанции.** Схема cargo-deny менялась между 0.18 → 0.19 → 0.20. `version = 2` жил в двух версиях, теперь устарел.
4. **`private = { ignore = true }` — ключ к простоте.** Без него пришлось бы перечислять 20+ compound-выражений. С ним — 17 простых ID.
5. **Не биться головой об конфиг бесконечно.** После 5–6 неудачных попыток — читать исходники инструмента (`src/licenses.rs`) и смотреть, как реально работает matching.

---

## 14.9. `cargo-deny` и `cargo-audit` в Dockerfile

**Дата:** 2026-09-29
**Commit:** `226401c`

### Зачем

После Stage 14.8 выяснилось: `cargo install cargo-deny` пишет бинарь в `~/.cargo/bin/` **внутри контейнера**, не в volume. При `docker compose up -d --force-recreate solana` — файловая система контейнера пересоздаётся из образа, все `cargo install` результаты теряются.

**Стоимость переустановки:** ~1m 40s на cargo-deny, ~2m на cargo-audit.

**Решение:** испечь в образ.

### Изменение в `Dockerfile.solana`

**Было:**
```dockerfile
  curl -fsSL https://get.pnpm.io/install.sh | env PNPM_VERSION=12.5.1 ... bash - && \
  echo 'export NVM_DIR="$HOME/.nvm"' >> $HOME/.bashrc && \
```

**Стало:**
```dockerfile
  curl -fsSL https://get.pnpm.io/install.sh | env PNPM_VERSION=12.5.1 ... bash - && \
  cargo install cargo-deny --locked --version 0.20.2 && \
  cargo install cargo-audit --locked && \
  echo 'export NVM_DIR="$HOME/.nvm"' >> $HOME/.bashrc && \
```

### Проверка

После rebuild + recreate:

```
$ which cargo-deny cargo-audit
/home/ubuntu/.cargo/bin/cargo-deny
/home/ubuntu/.cargo/bin/cargo-audit

$ cargo deny --config /home/ubuntu/deny.toml check
advisories ok, bans ok, licenses ok, sources ok
```

### Цена

**Build time:** 610 секунд (10 минут) — в основном компиляция cargo-deny из исходников.

**Размер образа:** ~50 MB дополнительно (cargo-deny + cargo-audit + их зависимости).

**Оправдано:** Dockerfile пересобирается редко. После пересборки экономия — 3–4 минуты на каждое пересоздание контейнера.

### Грабли

1. **`cargo install` в контейнере не персистентен.** Только volume'ы и образ сохраняют данные. `~/.cargo/` внутри контейнера — эфемерно.
2. **Первая сборка образа стала дольше.** 610s vs ~120s до этого. Кэш Docker-слоёв помогает при повторных сборках, но `cargo install` слои зависят от версии crates — обновление требует пересборки.

### Уроки

1. **Всё, что нужно для CI/разработки — в образе.** Не `cargo install` в runtime.
2. **`--locked --version X.Y.Z`** — фиксация версии cargo-deny важна (0.20.x vs 0.18.x — разные схемы конфига).
3. **Docker build — 10 минут, но это one-time.** Дальше `docker compose up` — секунды.
4. **Проверка после rebuild обязательна.** `which cargo-deny` + `cargo deny check` — двойная верификация, что всё на месте.

---

## 14.10. Release v0.1.0

**Дата:** 2026-09-29
**Commit:** `48302ed` (тег `v0.1.0`)

### Что произошло

Первый публичный релиз проекта. GitHub Actions workflow `release.yml` собрал и опубликовал артефакты.

**Release URL:** https://github.com/kwebhub/zkpool-solana/releases/tag/v0.1.0

### Артефакты

| Файл | Размер | Назначение |
|---|---|---|
| `zk_pool.so` | 205 KB | Anchor-программа (deployable) |
| `zk_pool.json` | 16.8 KB | IDL программы |
| `zkpool-backend` | 9.53 MB | Rust-бинарь backend'а |
| `zkpool-prover` | 3.46 MB | Rust-бинарь prover'а |
| `zkpool-web.tar.gz` | 1.16 MB | Vite-сборка фронтенда |
| `SHA256SUMS` | 401 B | Хэши всех артефактов |

Плюс автогенерируемые GitHub архивы `Source code (zip)` и `Source code (tar.gz)`.

### Пайплайн workflow

1. **Setup job** (1s) — Ubuntu latest, Node 24.
2. **checkout@v4** (1s) — клонирование репозитория.
3. **Install Rust (SBF)** (8s) — toolchain 1.89.0.
4. **Install Solana CLI** (11s) — через `release.anza.xyz`.
5. **Install Anchor** (2m 17s) — `avm install 1.1.2`.
6. **Build on-chain program** (3m 46s) — `anchor build --no-idl`.
7. **Build IDL** (0s) — `anchor idl build -o target/idl/zk_pool.json`. **Добавлено в этом этапе.**
8. **Collect program binary + IDL** (0s) — `cp` в `artifacts/`.
9. **Build backend** — `cargo build --release`.
10. **Collect backend binary** — `cp`.
11. **Build prover** — `cargo build --release`.
12. **Collect prover binary** — `cp`.
13. **Install Node.js** — setup-node@v4.
14. **Install pnpm** — 12.5.1.
15. **Install web deps** — `pnpm install --frozen-lockfile`.
16. **Build web** — `pnpm build`.
17. **Package web dist** — `tar czf`.
18. **Generate SHA-256 checksums** — `sha256sum * > SHA256SUMS`.
19. **Create Release** — `softprops/action-gh-release@v2`.

**Total duration:** 6m 37s.

### Два бага, которые пришлось починить

**1. GitHub Actions не запускались вообще.**

Симптом: workflow падали за 4 секунды, не начиная работу. Аннотация:

> The job was not started because recent account payments have failed or your spending limit needs to be increased.

Причина: приватный репозиторий с исчерпанным лимитом Actions-минут.

**Фикс:** сделать репозиторий **публичным**. Публичные репы получают неограниченные Actions-минуты на стандартных раннерах. Бонус: портфолио теперь видно всем.

**2. `anchor build --no-idl` не создаёт IDL.**

Симптом:

```
cp: cannot stat 'onchain/target/idl/zk_pool.json': No such file or directory
```

Причина: флаг `--no-idl` буквально означает «не генерировать IDL». Мы использовали его, чтобы `anchor build` не требовал keypair для деплоя.

**Фикс:** добавили отдельный шаг:

```yaml
- name: Build IDL
  working-directory: onchain
  run: anchor idl build -o target/idl/zk_pool.json
```

`anchor idl build` не требует keypair — генерирует IDL из скомпилированной программы.

### Пере-тегирование

Первый тег `v0.1.0` указывал на коммит `da93c91` (до фикса IDL). После фикса — удалили старый тег и создали новый на `48302ed`:

```bash
git tag -d v0.1.0
git push origin :v0.1.0
git tag -a v0.1.0 -m "..."
git push origin v0.1.0
```

**Допустимо,** потому что: тег был на приватном репо, релиз ещё не существовал, никто его не использовал. В production-сценарии — только через `workflow_dispatch` или UI-кнопку «Re-run».

### Грабли

1. **Failed payment → 4-секундные падения всех workflow.** Симптом легко спутать с YAML-ошибкой. Всегда смотреть аннотации в Summary.
2. **`--no-idl` — буквально.** Не «build без деплоя», а «build без IDL». IDL генерируется отдельной командой.
3. **`anchor idl build -o <path>`** — работает без keypair.
4. **Пере-тегирование — не всегда зло.** Пока релиз не опубликован и тег никто не скачал — можно.

### Уроки

1. **Довести CI до зелёного перед первым релизом.** Публикация релиза с падающим CI — плохой сигнал для портфолио.
2. **Смотреть на аннотации, не только на логи.** Billing-issue не появляется в выводе job'а — только в аннотациях наверху.
3. **Публичный репо — стандарт для портфолио.** Приватный — для коммерческой тайны, что не наш случай.
4. **`--no-idl` = «без IDL», не «без деплоя».** Внимательно читать флаги Anchor.
5. **Пере-тегирование допустимо до публикации артефактов.** После — только новый тег (`v0.1.1`).

---

## 14.11. Финальный чекпоинт Stage 14

**Дата:** 2026-09-29
**Commit:** `56eeaf1`

# ✅ Stage 14 завершён. Проект опубликован.

### Итог

| Под-этап | Тема | Commit |
|---|---|---|
| 14.1 | `.editorconfig` + LICENSE (MIT) | `843ac88` |
| 14.2 | `README.md` | `274bc4d` |
| 14.3 | `docs/ru/README.md` | `94d17b8` |
| 14.4 | `CONTRIBUTING.md` | `747a711` |
| 14.5 | `SECURITY.md` | `28553d4` |
| 14.6 | `CHANGELOG.md` (v0.1.0) | `537ea0d` |
| 14.7 | `docs/DEMO-NOTICE.md` update | `f8b3adc` |
| 14.8 | `cargo-deny` passes | `e09e8fd` |
| 14.9 | `cargo-deny` + `cargo-audit` в Dockerfile | `226401c` |
| 14.10 | Release `v0.1.0` | `48302ed` (tag) |
| 14.11 | Final checkpoint | `56eeaf1` |

### Публичный релиз

- **Tag:** `v0.1.0`
- **URL:** https://github.com/kwebhub/zkpool-solana/releases/tag/v0.1.0
- **Артефакты:** 6 файлов (программа, IDL, 2 бинаря, web-сборка, SHA256SUMS).

### Публичный репозиторий

- **URL:** https://github.com/kwebhub/zkpool-solana
- **Visibility:** Public
- **License:** MIT

### Что сделано в Stage 14

**Документация (11 файлов):**
- `README.md` (английский, основной).
- `docs/ru/README.md` (русский, портфолио-материал для русскоязычной аудитории).
- `CONTRIBUTING.md` — процесс для контрибьюторов.
- `SECURITY.md` — как сообщить об уязвимости.
- `CHANGELOG.md` — история релиза v0.1.0.
- `docs/DEMO-NOTICE.md` — обновлён под текущее состояние.
- `LICENSE`, `.editorconfig`.

**CI/CD:**
- Release workflow работает — успешно собран и опубликован v0.1.0.
- Docker-образ содержит `cargo-deny` + `cargo-audit`.
- `deny.toml` — все 4 проверки зелёные.

### Что осталось отложенным (Stage 15)

- **Split deposit** — разделение депозита на несколько commitment'ов.
- **Verify `new_root` on-chain** (A1 из threat model).
- **Client-side proving** (A6).
- **Increase `ROOT_HISTORY_SIZE`** (A10).
- **MPC trusted setup** для Groth16.
- **CSP для frontend** (production-only).
- **merkle CI job** — placeholder (нужен `nargo` в CI).

Все эти пункты задокументированы в `docs/threat-model.md` и `docs/DEMO-NOTICE.md`.

### Уроки Stage 14 (обобщение)

1. **`LICENSE` без файла — не лицензия.** GitHub, `cargo-deny`, SPDX — все ищут файл. `Cargo.toml` говорит MIT — файл `LICENSE` это подтверждает.
2. **Публичный репо — стандарт для портфолио.** Private блокирует GitHub Actions (billing) и скрывает работу от читателей.
3. **`--no-idl` = «без IDL».** Не путать с «без деплоя». IDL генерируется отдельной командой `anchor idl build`.
4. **`cargo-deny` 0.20.2 требует явных SPDX-выражений.** `allow-osi-fsf-free` удалён. `private = { ignore = true }` — ключ к простому списку.
5. **Docker build — 10 минут, но one-time.** `cargo install` внутри контейнера не персистентен. Всё нужное — в образ.
6. **Annotated tags с подробным сообщением.** Содержат описание релиза — полезно при просмотре на GitHub.
7. **Пере-тегирование до публикации — допустимо.** После публикации — только новый тег.

## Что дальше

- **Stage 15** — Split deposit (отложен, после v0.1.0).
- **После Stage 15** — статьи и туториалы на базе `docs/notes/` (Medium, Mirror.xyz, Habr, X, Telegram).
