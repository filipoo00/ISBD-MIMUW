# Projekt 2

## Wymagania

Rust 1.85+ (edition 2024)

## Uruchomienie

```
make            # build (cargo build --release)
make metrics    # metryki dla sample.bin
make generate   # wygenerowanie sample.bin od nowa
make test       # testy
```

Inny plik albo inna liczba wierszy:

```
make metrics SAMPLE=plik.bin
make generate SAMPLE=plik.bin ROWS=1000000
```

Bez `make`:

```
cargo run --release --bin metrics -- sample.bin
cargo run --release --bin generate -- sample.bin [number_of_rows]
```

## Przykładowy plik

`sample.bin`:  10 000 wierszy w 10 batchach (9 x 1024 + 784), wygenerowany przez `make generate`.

Kolumna | Typ | Zawartość

- 0 | INT64 | od -5000 do 4999
- 1 | BOOLEAN | `true` dla co trzeciego wiersza
- 2 | VARCHAR | `row_0`, `row_1`, ...
- 3 | VARCHAR | powtarzane: `a`, `ab`, `abc`, `ą`, `ąb`, `ąbc`


## Wynik
Wyniki na wygenerowanym `sample.bin`:
```
make metrics
```

```
Read 10000 rows in 10 batches.
Column 0 (INT64): mean = -0.5
Column 1 (BOOLEAN): entropy = 0.9183624862946866
Column 2 (VARCHAR): ascii_count = 78890
Column 3 (VARCHAR): ascii_count = 15000
```

## Format pliku

Wszystkie liczby zapisane są w little endian.

```
batch 0
    number_of_rows:       u16
    kolumna 0:
        payload_size      u64,
        payload
    kolumna 1:
        payload_size      u64,
        payload
    ...
batch 1
    ...
footer
    number_of_batches:    u32
    number_of_cols:       u32
    column_types:         u8 na kolumnę (typ)
footer_size:              u32 (footer size)
```

Footer jest na końcu pliku, więc writer zapisuje batche strumieniowo, bez znajomości ich liczby.

Reader czyta najpierw ostatnie 4 bajty (`footer_size`), potem footer (schemat i liczbę batchy), a następnie batche po kolei od początku pliku.

Batch ma co najwyżej `BATCH_SIZE` = 1024 wierszy.
