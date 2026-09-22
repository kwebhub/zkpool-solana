# Этап 0. Методология чекпоинтов

## Зачем это

В прошлом проходе проекта баг с `withdraw` (`InvalidInstructionData`) не удалось найти за три дня, потому что **каждый этап проверялся отдельно**, а **стыки между этапами** — нет.

Здесь мы фиксируем артефакты **после каждого этапа** и **сверяем их побайтово** на следующем этапе. Это позволяет поймать расхождение **в момент его появления**, а не в конце.

## Правило

После каждого этапа, где генерируется **любой артефакт** (ACIR, witness, proof, public witness, instruction data, JSON-клиент), мы:

1. Копируем артефакт в `.checkpoints/NN-name/`.
2. Считаем `sha256sum`.
3. Записываем хеши в `.checkpoints/NN-name/manifest.txt`.
4. Фиксируем коммит в `.checkpoints/NN-name/commit.txt`.

На следующем этапе, где этот артефакт **потребляется**, сверяем хеш. Если не совпал — стоп, разбираемся.

## Структура

```
.checkpoints/
├── README.md
├── 00-init/
│   ├── commit.txt
│   └── manifest.txt
├── 01-setup/
│   ├── commit.txt
│   ├── versions.txt
│   └── sunspot-clone.txt
├── 02-circuits/
│   ├── commit.txt
│   ├── nargo-version.txt
│   ├── hash2.json.sha256
│   ├── hashes.json.sha256
│   ├── withdrawal.json.sha256
│   ├── Prover.toml
│   └── manifest.txt
├── 03-sunspot/
│   ├── commit.txt
│   ├── withdrawal.ccs.sha256
│   ├── withdrawal.pk.sha256
│   ├── withdrawal.vk.sha256
│   ├── withdrawal.so.sha256
│   ├── proof.sha256
│   ├── public_witness.sha256
│   └── manifest.txt
├── 04-anchor/
│   ├── commit.txt
│   ├── encode_public_inputs.hex
│   ├── idl.json.sha256
│   └── manifest.txt
├── 07-prover/
│   ├── commit.txt
│   ├── proof-from-prover.sha256
│   ├── public_witness-from-prover.sha256
│   └── manifest.txt
└── 08-frontend/
    ├── commit.txt
    ├── witness-from-frontend.sha256
    ├── instruction-data-from-frontend.hex
    └── manifest.txt
```

## Папка `.checkpoints/` — gitignored

Она **не коммитится**. Это локальные артефакты для сверки. Если нужно передать их кому-то — пакуются в архив вручную.

## Ключевые сверки

| Артефакт | Где создаётся | Где сверяется |
|---|---|---|
| `hash2.json`, `hashes.json`, `withdrawal.json` | Этап 2 | Этап 6 (merkle), Этап 8 (frontend) |
| `Prover.toml` (эталонный witness) | Этап 2 | Этап 3, 4, 7, 8 |
| `withdrawal.ccs`, `.pk`, `.vk`, `.so` | Этап 3 | Этап 4 (anchor), Этап 7 (prover) |
| `proof`, `public_witness` (локальные) | Этап 3 | Этап 7 (prover) |
| `encode_public_inputs` (172 байта) | Этап 4 | Этап 7, 8 |
| witness из `useWithdraw.ts` | Этап 8 | должен совпасть с `Prover.toml` |

## Формат `manifest.txt`

```
<sha256>  <relative-path-from-repo-root>
```

Пример:
```
a1b2c3...  circuits/withdrawal/target/withdrawal.json
d4e5f6...  circuits/withdrawal/Prover.toml
```

## Что делать, если хеши не совпали

1. **Стоп.** Не идём дальше.
2. Сравниваем байты (`cmp`, `diff <(xxd a) <(xxd b)`).
3. Ищем **источник** расхождения: версия nargo, версия noir_js, версия Sunspot, порядок полей, формат кодирования.
4. Фиксируем в `docs/notes/NN-name.md` раздел «Ошибки».
5. Только после устранения — идём дальше.

## Версии (зафиксированы в чекпоинте 1)

| Инструмент | Версия |
|---|---|
| rustc | 1.98.1 |
| cargo | 1.98.1 |
| solana-cli | 3.1.10 |
| anchor-cli | 1.1.2 |
| nargo | 1.0.0-rc.2 |
| sunspot | 1.0.0 |
| node | 24.21.0 |
| pnpm | 12.5.1 |

Любое расхождение с этими версиями — **повод остановиться** и разобраться.
