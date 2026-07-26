**Каноническое Ядро L1.1**

Канонизируем не весь текущий Lay и не временные эксперименты. Канонизируем конкретный контракт:

> L1.1 - самостоятельное ядро восстановления повреждённого дискретного сигнала в устойчивый лексический центр либо честный `Tied/ABSTAIN`.

```text
повреждённая поверхность
-> типизированные n-граммные атомы
-> прямая волна кандидатов
-> обратная волна реконструкции
-> positive / anti / ambiguity interference
-> bounded lattice crystallization
-> Winner | Tied | ABSTAIN
```

**Числа Ядра**

```text
размер комплексного поля              128 re + 128 im ячеек
WordCenter                            строго 64 байта
AtomWaveCode                          строго 16 байт
компоненты WordCenter wave code       22
компоненты AtomWaveCode                4
основной phase frontier              128 центров
ambiguity reserve                     32 центра
settling iterations                    3
максимальная anchor sequence          32 символа
returned evidence lattice             32 кандидата
positive subcenters                    4 на центр
anti subcenters                        4 на центр
hard-negative subcenters               2 на центр
```

**Что Хранится**

```text
L1.1 PACKAGE
|
+-- SymbolTable / NGramGraph
|   symbol sequence -> плотный AtomId
|
+-- SharedWaveBasis
|   128 общих комплексных базисных волн
|
+-- AtomWaveCode[AtomId]
|   компактная комбинация общего базиса
|
+-- ForwardCouplings
|   AtomId -> WordCenterId
|
+-- ReverseCouplings
|   WordCenterId -> ожидаемые AtomId и позиции
|
+-- WordCenter64[]
|   устойчивые выученные лексические центры
|
+-- PositiveSubcenters
+-- AntiSubcenters
+-- HardNegativeSubcenters
+-- AmbiguitySubcenters
|
+-- PairwiseProfiles
|   направленные отношения между реальными кандидатами lattice
|
+-- DecoderGraph
    WordCenterId -> точная UTF-8 поверхность
```

Хеш не является словом, атомом или ответом. Он используется только для адресации и детерминированной проекции. Идентичность n-граммы принадлежит `NGramGraph`, идентичность слова - `WordCenterId`.

**Формула Прямой Волны**

Для атомов наблюдаемой поверхности:

```text
Forward(w) =
  sum over atoms i:
    coupling(i,w)
    * atom_weight(i)
    * position_coherence(i,w)
```

Одновременно строится комплексная поверхность:

```text
S = sum_i weight_i * expand(AtomWaveCode_i, SharedWaveBasis)
```

**Обратная Волна**

Для каждого родившегося центра:

```text
WordCenter(w)
-> ReverseCouplings(w)
-> ожидаемые атомы и позиции
-> ReconstructionWave R(w)
-> backward coherence(S, R(w))
```

То есть действительно встречаются две волны:

```text
SurfaceWave(input)
+
ReconstructionWave(candidate)
+
PositiveWave
-
AntiWave
-
HardNegativeWave
```

**Кристаллизация**

Начальная энергия:

```text
E0 = 3 * Forward
```

Три итерации:

```text
constructive =
    3 * Backward
  + 2 * max(Positive - 500, 0)

destructive =
    4 * Anti
  + 2 * max(500 - Positive, 0)
  + 2 * (1000 - LengthCoherence)

E(t+1) =
  (E(t) + 3*Forward + constructive + LengthCoherence - destructive) / 2
```

Затем действуют sequence, position, ambiguity и pairwise interference. Это не обычная сумма рейтингов: конкурент может разрушить authority другого центра.

**Рождение Кандидатов**

```text
все реальные forward postings наблюдаемых атомов
-> complete touched-center set
-> main mass frontier: максимум 128
-> geometry reserve: максимум 32
-> settling и интерференция
```

Для повреждённой поверхности reserve сохраняет:

```text
минимальный геометрический бассейн
+
соседнюю ambiguity shell: min_distance + 1
```

Эта оболочка не выбирает слово. Она не позволяет преждевременно уничтожить серьёзного конкурента.

**Закон Authority**

```text
один устойчивый центр
+ достаточная positive/backward coherence
+ нет сильной anti/hard-negative волны
+ нет unresolved ambiguity shell
+ нет pairwise conflict/cycle
= Winner
```

```text
несколько правдоподобных центров
+ нет полного pairwise certificate
= Tied
```

```text
слабое, противоречивое или непокрытое поле
= ABSTAIN
```

Только полный crystallization certificate может превратить неоднозначный бассейн в singleton. Ни индекс, ни позиция кандидата, ни правильный ответ proof не дают authority.

**Что Не Хранится**

```text
сырой корпус
таблица "ошибка -> правильное слово"
повреждённые учебные строки
HashMap<String, Word>
target heldout-примера
полный 128-ячеечный вектор для каждого слова
```

**Текущий Канонический Артефакт**

```text
словарь                              10,000 центров
training surfaces                   789,936
heldout surfaces                    107,544
NGram atoms                         113,864
forward couplings                 5,304,083
reverse couplings                 1,053,773
package size                        49,663,268 bytes
WordCenter bank                     640,000 bytes
startup / full one-shot CLI         0.25-0.60 s
peak process RSS                    124,072-124,196 KiB
hot p50 / p99                       3.782 / 4.754 ms
```

```text
clean preservation                  100.000%
top-1                               94.521%
top-64                              99.943%
evidence retained                   99.752%
authority target winner              65.699%
ambiguity safety                    2783/2783 = 100%
false authority                     0
false singleton                     0
reconstruction recovered/lost       217/0
```

Вердикты различаются:

```text
L1.1 restoration safety             PASS_shadow
общий target-label ranking          WATCH_shadow
```

То есть восстановительный контракт уже доказан, но модель ещё не достигла требуемого качества выбора по всем классам.

**Фактическая Память, Диск И CPU**

Замер сделан 23 июля 2026 года на локальном T480:

```text
CPU                                 Intel Core i7-8650U
physical cores / logical CPUs       4 / 8
CPU L1d / L2 / L3                   128 KiB / 1 MiB / 8 MiB
RAM                                 32,176,259,072 bytes = 29.97 GiB
OS                                  Linux x86_64
```

Канонический package:

```text
/home/ubu/projects/lay/data/lexical_grokking/l1_l11_crystallization_10k.bin
```

Его точная дисковая раскладка:

```text
header                                      192 bytes
NGram nodes: 125,535 x 12             1,506,420 bytes
NGram arcs: 125,534 x 8               1,004,272 bytes
shared basis: 128 x 256                  32,768 bytes
atoms: 113,864 x 24                   2,732,736 bytes
compressed forward postings          18,680,980 bytes
reverse couplings: 1,053,773 x 8      8,430,184 bytes
anti centers: 25,103 x 64             1,606,592 bytes
pair profiles: 74,469 x 24            1,787,256 bytes
pair centers: 75,038 x 64             4,802,432 bytes
primary WordCenter64: 10,000 x 64       640,000 bytes
decoder nodes: 38,677 x 8               309,416 bytes
base package subtotal                41,533,248 bytes

L1.1 extension header                       40 bytes
center phase profiles: 10,000 x 24       240,000 bytes
positive subcenters: 36,646 x 64        2,345,344 bytes
anti subcenters: 25,103 x 64            1,606,592 bytes
hard-negative subcenters: 16,798 x 64   1,075,072 bytes
keyboard geometry units: 99,407 x 4       397,628 bytes
ambiguity subcenters: 38,521 x 64       2,465,344 bytes
L1.1 extension subtotal                 8,130,020 bytes

TOTAL                                  49,663,268 bytes
logical file size                          47.36 MiB
allocated filesystem blocks            49,668,096 bytes
```

На диске `forward postings` сжаты до `18,680,980` байт. Текущий decoder
разворачивает их в `5,304,083 x 8 = 42,432,664` байта RAM. Поэтому размер
файла нельзя считать размером готовой структуры в heap.

Расчёт полезной нагрузки decoded package в RAM:

```text
NGram nodes                            1,506,420 bytes
NGram arcs                             1,004,272 bytes
shared complex basis                      32,768 bytes
AtomRecord bank                        2,732,736 bytes
expanded forward couplings            42,432,664 bytes
reverse couplings                      8,430,184 bytes
anti centers                           1,606,592 bytes
pair profiles in Rust                  1,489,380 bytes
pair centers                           4,802,432 bytes
center phase profiles in Rust            280,000 bytes
positive subcenters                    2,345,344 bytes
anti subcenters                        1,606,592 bytes
hard-negative subcenters               1,075,072 bytes
ambiguity subcenters                   2,465,344 bytes
keyboard geometry units                  397,628 bytes
primary WordCenter64 bank                640,000 bytes
decoder nodes                            309,416 bytes

decoded Vec payload subtotal          73,156,844 bytes = 69.77 MiB
```

Это subtotal самих массивов. Сверх него процесс держит `Vec` metadata,
allocator capacity, `character_anchors`, exact-surface index и краткоживущие
контейнеры readout.

Текущий standalone runtime пока не является `mmap`-runtime:

```text
std::fs::read(package)
-> raw input buffer                    49,663,268 bytes = 47.36 MiB
-> decode
-> expanded package Vec payload        73,156,844 bytes = 69.77 MiB
-> runtime indexes and allocator overhead
-> readout
```

Во время decode сырой пакет и уже развёрнутые массивы существуют одновременно.
Пять успешных запусков текущего release-бинарника дали:

```text
peak RSS                               124,088-124,196 KiB
                                       121.18-121.29 MiB
observed minimum including дождь       124,072 KiB = 121.16 MiB
wall time                              0.25-0.60 s
user + system CPU time                 0.21-0.29 s
swap                                   0
```

CPU scratch на один runtime thread:

```text
ForwardActivation[10,000]              160,000 bytes
touched WordCenterId[<=10,000]         <=40,000 bytes
one complex surface re/im pair           1,024 bytes
main phase frontier                    <=128 candidates
geometry reserve                       <=32 candidates
anchor sequence                        <=32 u32 atoms
```

То есть постоянный dense scratch сейчас около `200 KiB` на thread плюс
bounded candidate/sequence containers. Полная модель не помещается в CPU cache:
она живёт в RAM, а L1/L2 cache получают только текущие graph nodes, postings,
frontier и фазовые аккумуляторы. На этом T480 общий L3 равен `8 MiB`.

Текущий standalone release-бинарник:

```text
/home/ubu/projects/lay/target/release/lay-l11-restore
logical size                              890,408 bytes = 0.849 MiB
allocated filesystem blocks               892,928 bytes
binary + one canonical package         50,553,676 bytes = 48.21 MiB
```

`target/` не является Lay, моделью или runtime dependency. Это удаляемый Cargo
build-cache:

```text
/home/ubu/projects/lay/target
allocated size                       7,827,763,200 bytes = 7.29 GiB
configured budget                   12,884,901,888 bytes = 12.00 GiB
```

Каталог `data/lexical_grokking/` сейчас занимает `155,934,720` байт
выделенных блоков, потому что в нём лежат несколько экспериментальных packages.
Для одного запуска L1.1 нужен только один выбранный package. Сырой учебный
корпус в runtime package не хранится: `0 bytes`.

**Что Отвергнуто**

```text
global frontier 256
-> квадратичный рост pairwise
-> proof больше 12 минут
-> не канонизируем

precomputed nearest-neighbor map
-> startup около 22 секунд
-> RSS около 500 MiB
-> не канонизируем

словарные правила
-> не канонизируем

candidate reranker под названием L2
-> не является L2
-> остаётся внутри L1 либо удаляется

подключение L1.1 к IME/daemon
-> пока запрещено
```

**Что Именно Фиксируем**

```text
L1.1-CANONICAL-SHADOW-10K
|
+-- typed reversible atoms
+-- dense AtomId
+-- WordCenter64
+-- 128-cell shared complex basis
+-- bidirectional couplings
+-- positive/anti/hard-negative subcenters
+-- pairwise crystallization
+-- frontier 128
+-- ambiguity reserve 32
+-- three settling iterations
+-- Winner/Tied/ABSTAIN
+-- zero false authority gate
```

Package: [l1_l11_crystallization_10k.bin](/home/ubu/projects/lay/data/lexical_grokking/l1_l11_crystallization_10k.bin)

Архитектура: [l1-crystal-kernel-memory-layout.md](/home/ubu/projects/lay/docs/l1-crystal-kernel-memory-layout.md:603)

Коммит: `b934922 Complete L1.1 ambiguity-safe crystallization`.

Для `100k/600k` канонизируется смысл ядра и его authority-контракт. Текущие способы обхода touched-centers и хранения package не канонизируются: перед масштабированием нужны compressed complete postings, sound Block-Max pruning и mmap-runtime.
