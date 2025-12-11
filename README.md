# micro:bit 貪食蛇遊戲

一個運行在 BBC micro:bit V2 開發板上的經典貪食蛇遊戲，使用 Rust 語言和嵌入式開發技術實現。

## 🎮 遊戲功能

### 核心玩法
- 🐍 **蛇的移動**：蛇會持續向前移動，每 3 秒前進一格
- 🎮 **方向控制**：按鈕 A 向左轉，按鈕 B 向右轉
- 🍎 **食物系統**：吃到食物後蛇身變長，分數增加
- 💀 **碰撞檢測**：撞到自己時遊戲結束
- 🔄 **邊界環繞**：蛇可以從螢幕一邊穿越到另一邊
- 🔁 **自動重啟**：遊戲結束後顯示動畫並自動開始新遊戲

### 視覺效果
- **蛇身閃爍**：亮 100ms，暗 10ms，週期性閃爍
- **食物閃爍**：與蛇同步閃爍，但會在偶數幀消失以示區別
- **遊戲結束動畫**：顯示 X 圖案並閃爍 2 次

### 遊戲參數
- **初始蛇長**：3 格
- **起始位置**：螢幕中央 (2, 2)
- **起始方向**：向右
- **移動速度**：3000ms（3秒）每格
- **閃爍頻率**：每 110ms 一個週期（亮 100ms + 暗 10ms）

## 🏗️ 程式架構

### 專案結構
```
microbit-snake/
├── src/
│   └── main.rs              # 主程式（包含所有遊戲邏輯）
├── .cargo/
│   └── config.toml          # Rust 目標配置
├── Cargo.toml               # 專案依賴和元數據
├── Cargo.lock               # 依賴版本鎖定
├── microbit-build.sh        # 建置和燒錄腳本
├── microbit-add-thumbv7em.sh # 環境設置腳本
└── README.md                # 本文件
```

### 依賴項
```toml
[dependencies]
microbit-v2 = "0.15.1"      # micro:bit V2 硬體抽象層
cortex-m-rt = "0.7.3"       # ARM Cortex-M 運行時
panic-halt = "0.2.0"        # Panic 處理器
heapless = "0.8"            # 無堆疊記憶體集合
embedded-hal = "1.0"        # 嵌入式硬體抽象層
```

## 📊 程式作業流程

### 1. 系統初始化流程
```
main() 啟動
    ↓
初始化硬體
  ├─ Timer（TIMER0）     → 時間控制
  ├─ Display             → LED 矩陣顯示
  ├─ RNG                 → 隨機數生成器
  └─ Buttons             → 按鈕輸入
    ↓
創建遊戲實例
  ├─ 初始化蛇（3格，中央位置）
  ├─ 生成隨機食物
  └─ 設定遊戲狀態為 Ongoing
    ↓
進入主遊戲循環
```

### 2. 主遊戲循環流程
```
┌─────────────────────────────────┐
│       主遊戲循環（無限）          │
└─────────────────────────────────┘
         ↓
    ┌────────────────┐
    │ 1. 讀取按鈕輸入 │
    │   - Button A    │
    │   - Button B    │
    └────────────────┘
         ↓
    ┌────────────────┐
    │ 2. 更新蛇方向   │
    │   - 左轉         │
    │   - 右轉         │
    │   - 不轉         │
    └────────────────┘
         ↓
    ┌────────────────┐
    │ 3. 移動蛇        │
    │   - 計算新位置   │
    │   - 檢測碰撞     │
    │   - 檢查食物     │
    │   - 更新蛇身     │
    └────────────────┘
         ↓
    ┌────────────────┐
    │ 4. 檢查遊戲狀態 │
    └────────────────┘
         ↓
      遊戲結束？
       /      \
     是         否
     ↓          ↓
  顯示動畫   閃爍顯示3秒
  重新開始    (27次閃爍)
     ↓          ↓
     └──────────┘
         ↓
    （回到循環開始）
```

### 3. 蛇移動邏輯流程
```
step(rng) 函數
    ↓
遊戲進行中？ ───否─→ 返回
    ↓ 是
增加 tick 計數器
    ↓
計算下一個位置
  = direction.step(head).wrap()
    ↓
檢測碰撞 ───是─→ 設定狀態為 GameOver
    ↓ 否          返回
檢查食物位置
    ↓
  下一位置 == 食物？
   /            \
  是             否
  ↓              ↓
蛇變長         正常移動
增加分數       (移除尾部)
生成新食物
  ↓              ↓
  └──────────────┘
         ↓
       返回
```

### 4. 碰撞檢測流程
```
檢測碰撞
    ↓
遍歷蛇身所有節點（跳過頭部）
    ↓
next_pos == 任一節點？
   /              \
  是               否
  ↓                ↓
返回 true      返回 false
(發生碰撞)     (無碰撞)
```

### 5. 食物生成流程
```
生成新食物
    ↓
生成隨機座標
  (row: 0-4, col: 0-4)
    ↓
檢查是否在蛇身上？
   /              \
  是               否
  ↓                ↓
重新生成       設定為食物位置
  ↓                ↓
  └────────────────┘
         ↓
       返回
```

### 6. 顯示渲染流程
```
render() 函數
    ↓
創建空白矩陣 [[0; 5]; 5]
    ↓
繪製蛇身
  ├─ 遍歷 body[1..]
  └─ 設定亮度 = 9
    ↓
繪製蛇頭
  └─ 設定 head 位置亮度 = 9
    ↓
繪製食物（條件式）
  ├─ 如果 tick % 2 == 0
  └─ 設定食物位置亮度 = 9
    ↓
返回矩陣
    ↓
閃爍顯示循環（27次）
  ├─ 顯示矩陣 100ms（亮）
  └─ 顯示空白 10ms（暗）
```

## 🔧 核心數據結構

### Coords（座標）
```rust
struct Coords {
    row: i8,    // 行座標（-∞ to +∞，會被 wrap 到 0-4）
    col: i8,    // 列座標（-∞ to +∞，會被 wrap 到 0-4）
}
```

**功能**：
- `new(row, col)` - 創建新座標
- `random(rng)` - 生成隨機座標（0-4 範圍）
- `wrap()` - 邊界環繞處理（將座標限制在 0-4 範圍）

**用途**：表示蛇的每個節點、食物位置

---

### Direction（方向）
```rust
enum Direction {
    Up,      // 向上（row - 1）
    Down,    // 向下（row + 1）
    Left,    // 向左（col - 1）
    Right,   // 向右（col + 1）
}
```

**功能**：
- `turn_left()` - 向左轉 90 度
- `turn_right()` - 向右轉 90 度
- `step(coords)` - 根據方向計算下一個座標

**轉向規則**：
```
        Up
         ↑
Left ←   ● → Right
         ↓
       Down
```

---

### Turn（轉向指令）
```rust
enum Turn {
    Left,    // 向左轉
    Right,   // 向右轉
    None,    // 不轉向
}
```

**用途**：表示玩家的按鈕輸入

---

### GameStatus（遊戲狀態）
```rust
enum GameStatus {
    Ongoing,   // 遊戲進行中
    GameOver,  // 遊戲結束
}
```

**用途**：控制遊戲循環和狀態機

---

### Snake（蛇）
```rust
struct Snake {
    head: Coords,                  // 蛇頭位置
    body: Vec<Coords, 25>,         // 蛇身所有節點（包含頭部）
    direction: Direction,          // 當前移動方向
}
```

**功能**：
- `new(start_pos, direction)` - 創建初始蛇（長度3）
- `collides_with_self(pos)` - 檢查位置是否與蛇身碰撞
- `move_forward()` - 正常移動（頭部前進，尾部縮短）
- `move_and_grow()` - 吃到食物後移動（頭部前進，保持尾部）

**body 陣列結構**：
```
body[0] = 蛇頭
body[1] = 頸部
body[2] = 身體
...
body[n-1] = 尾部
```

---

### Game（遊戲）
```rust
struct Game {
    snake: Snake,           // 蛇實例
    food: Coords,           // 食物位置
    status: GameStatus,     // 遊戲狀態
    score: u8,              // 當前分數
    tick: u32,              // 遊戲幀計數（用於閃爍效果）
}
```

**功能**：
- `new(rng)` - 創建新遊戲
- `turn(turn)` - 處理玩家轉向輸入
- `step(rng)` - 執行一個遊戲步驟（移動、碰撞檢測等）
- `render()` - 生成 LED 顯示矩陣

**遊戲狀態轉換**：
```
Ongoing ──(撞到自己)──→ GameOver
   ↑                        ↓
   └────(重新開始)──────────┘
```

## 📝 主要函數說明

### 初始化函數

#### `main() -> !`
**功能**：程式入口點，永不返回
**流程**：
1. 初始化硬體外設（Timer、Display、RNG、Buttons）
2. 創建遊戲實例
3. 進入無限主循環

---

### 遊戲邏輯函數

#### `Game::new(rng: &mut Rng) -> Self`
**功能**：創建新遊戲
**參數**：
- `rng` - 隨機數生成器引用
**返回**：初始化的 Game 實例
**內部操作**：
1. 創建初始蛇（位置 (2,2)，方向向右，長度 3）
2. 生成隨機食物（確保不在蛇身上）
3. 初始化遊戲狀態和分數

---

#### `Game::turn(&mut self, turn: Turn)`
**功能**：根據玩家輸入更新蛇的方向
**參數**：
- `turn` - 轉向指令（Left/Right/None）
**操作**：
- `Turn::Left` → 調用 `direction.turn_left()`
- `Turn::Right` → 調用 `direction.turn_right()`
- `Turn::None` → 保持當前方向

---

#### `Game::step(&mut self, rng: &mut Rng)`
**功能**：執行一個遊戲步驟（核心遊戲邏輯）
**參數**：
- `rng` - 隨機數生成器引用
**流程**：
1. 檢查遊戲是否進行中，否則直接返回
2. 增加 tick 計數器
3. 計算蛇的下一個位置（考慮邊界環繞）
4. 碰撞檢測（檢查是否撞到自己）
5. 如果碰撞，設定狀態為 GameOver 並返回
6. 檢查是否吃到食物：
   - 是：蛇變長，分數+1，生成新食物
   - 否：蛇正常移動

---

#### `Game::render(&self) -> [[u8; 5]; 5]`
**功能**：生成 LED 顯示矩陣
**返回**：5×5 亮度矩陣（0=熄滅，9=最亮）
**渲染規則**：
- 蛇身（body[1..]）：亮度 9
- 蛇頭（body[0]）：亮度 9
- 食物：亮度 9（僅在 tick 為偶數時顯示）

---

### 蛇操作函數

#### `Snake::new(start_pos: Coords, direction: Direction) -> Self`
**功能**：創建初始蛇
**參數**：
- `start_pos` - 起始位置（頭部）
- `direction` - 初始方向
**返回**：長度為 3 的蛇
**實現**：
1. 頭部在 `start_pos`
2. 第二節在頭部反方向一格
3. 第三節（尾部）在第二節反方向一格

---

#### `Snake::collides_with_self(&self, pos: Coords) -> bool`
**功能**：檢查位置是否與蛇身碰撞
**參數**：
- `pos` - 要檢查的位置
**返回**：true=碰撞，false=無碰撞
**實現**：遍歷 body 陣列，檢查是否有任何節點位於 pos

---

#### `Snake::move_forward(&mut self)`
**功能**：正常移動蛇（不吃食物）
**操作**：
1. 計算新頭部位置 = `direction.step(head).wrap()`
2. 將新頭部插入 body[0]
3. 移除 body 最後一個元素（尾部）
4. 更新 head 欄位

---

#### `Snake::move_and_grow(&mut self)`
**功能**：移動並增長蛇（吃到食物）
**操作**：
1. 計算新頭部位置 = `direction.step(head).wrap()`
2. 將新頭部插入 body[0]
3. **不移除**尾部（蛇變長）
4. 更新 head 欄位

---

### 方向操作函數

#### `Direction::turn_left(&self) -> Self`
**功能**：向左轉 90 度
**映射**：
- Up → Left
- Left → Down
- Down → Right
- Right → Up

---

#### `Direction::turn_right(&self) -> Self`
**功能**：向右轉 90 度
**映射**：
- Up → Right
- Right → Down
- Down → Left
- Left → Up

---

#### `Direction::step(&self, coords: Coords) -> Coords`
**功能**：根據方向計算下一個座標
**參數**：
- `coords` - 當前座標
**返回**：新座標（可能超出邊界，需要後續 wrap）
**計算**：
- Up: row - 1
- Down: row + 1
- Left: col - 1
- Right: col + 1

---

### 座標操作函數

#### `Coords::new(row: i8, col: i8) -> Self`
**功能**：創建座標
**參數**：
- `row` - 行（可為負數或超過 4）
- `col` - 列（可為負數或超過 4）

---

#### `Coords::random(rng: &mut Rng) -> Self`
**功能**：生成隨機座標
**參數**：
- `rng` - 隨機數生成器
**返回**：行和列都在 0-4 範圍內的座標
**實現**：`(rng.random_u8() % 5) as i8`

---

#### `Coords::wrap(&self) -> Self`
**功能**：邊界環繞處理
**返回**：限制在 0-4 範圍內的座標
**算法**：`((value % 5) + 5) % 5`
**範例**：
- -1 → 4（從左邊出去出現在右邊）
- 5 → 0（從右邊出去出現在左邊）
- 2 → 2（正常範圍內不變）

---

### 輸入處理函數

#### `read_buttons(buttons: &mut Buttons) -> Turn`
**功能**：讀取按鈕狀態並轉換為轉向指令
**參數**：
- `buttons` - micro:bit 按鈕引用
**返回**：轉向指令
**邏輯**：
```
Button A 按下 且 Button B 未按 → Turn::Left
Button B 按下 且 Button A 未按 → Turn::Right
其他情況（都按或都不按）   → Turn::None
```

---

### 顯示函數

#### `show_game_over<T: Instance>(display: &mut Display, timer: &mut Timer<T>)`
**功能**：顯示遊戲結束動畫
**參數**：
- `display` - LED 顯示器引用
- `timer` - 計時器引用
**動畫**：
1. 顯示 X 圖案 500ms
2. 全暗 500ms
3. 重複 2 次（共閃爍 2 次）

**X 圖案**：
```
[ ·  ·  ·  ●  · ]
[ ·  ●  ·  ●  · ]
[ ·  ·  ●  ·  · ]
[ ·  ●  ·  ●  · ]
[ ·  ●  ·  ·  · ]
```

---

## 🔢 重要變數說明

### 全域常量（在 main 函數中定義）

#### `move_delay_ms: u32 = 3000`
**用途**：蛇移動週期（毫秒）
**值**：3000（3秒）
**影響**：控制遊戲速度，越小遊戲越快

---

#### `blink_on_ms: u32 = 100`
**用途**：LED 亮的持續時間（毫秒）
**值**：100
**影響**：閃爍時亮的時間，越長越容易看清

---

#### `blink_off_ms: u32 = 10`
**用途**：LED 暗的持續時間（毫秒）
**值**：10
**影響**：閃爍時暗的時間，越長閃爍越明顯

---

#### `blink_cycle_ms: u32 = 110`
**用途**：一個完整閃爍週期（毫秒）
**計算**：`blink_on_ms + blink_off_ms`
**值**：110

---

#### `num_blinks: u32`
**用途**：每個移動週期內的閃爍次數
**計算**：`move_delay_ms / blink_cycle_ms`
**值**：27（3000 / 110）

---

### 結構體成員變數

#### `Game.snake: Snake`
**類型**：Snake 結構體
**用途**：存儲蛇的完整狀態

---

#### `Game.food: Coords`
**類型**：Coords 座標
**用途**：當前食物的位置
**範圍**：(0-4, 0-4)

---

#### `Game.status: GameStatus`
**類型**：GameStatus 枚舉
**用途**：當前遊戲狀態
**可能值**：
- `Ongoing` - 遊戲進行中
- `GameOver` - 遊戲結束

---

#### `Game.score: u8`
**類型**：無符號 8 位整數
**用途**：玩家當前分數
**範圍**：0-255
**增長**：每吃到一個食物 +1

---

#### `Game.tick: u32`
**類型**：無符號 32 位整數
**用途**：遊戲幀計數器
**用途**：
- 追蹤遊戲進行了多少步
- 用於食物閃爍效果（tick % 2）

---

#### `Snake.head: Coords`
**類型**：Coords 座標
**用途**：蛇頭當前位置
**範圍**：(0-4, 0-4)
**特性**：始終等於 `body[0]`

---

#### `Snake.body: Vec<Coords, 25>`
**類型**：heapless::Vec（固定容量向量）
**容量**：最多 25 個元素（5×5 螢幕）
**用途**：存儲蛇的所有節點
**結構**：
- `body[0]` - 頭部
- `body[1..n-1]` - 身體
- `body[n-1]` - 尾部

---

#### `Snake.direction: Direction`
**類型**：Direction 枚舉
**用途**：蛇當前移動方向
**可能值**：Up, Down, Left, Right

---

#### `Coords.row: i8`
**類型**：有符號 8 位整數
**用途**：座標的行（Y 軸）
**範圍**：理論上 -128 到 127，但會被 wrap 到 0-4

---

#### `Coords.col: i8`
**類型**：有符號 8 位整數
**用途**：座標的列（X 軸）
**範圍**：理論上 -128 到 127，但會被 wrap 到 0-4

---

## 🚀 建置和執行

### 環境設置
```bash
# 1. 安裝 Rust 目標和工具
bash microbit-add-thumbv7em.sh

# 會安裝：
# - thumbv7em-none-eabihf 目標（ARM Cortex-M4F）
# - llvm-tools-preview（用於生成 hex 檔案）
# - probe-rs-tools、flip-link（燒錄工具）
```

### 建置專案
```bash
# 只建置，不燒錄
bash microbit-build.sh --no-flash

# 建置並自動燒錄到 micro:bit
bash microbit-build.sh
```

### 輸出檔案
- **ELF 檔案**：`target/thumbv7em-none-eabihf/release/microbit-snake`（約 136KB）
- **HEX 檔案**：`target/thumbv7em-none-eabihf/release/microbit-snake.hex`（約 25KB）

### 手動燒錄
如果自動燒錄失敗，可以手動複製：
1. 連接 micro:bit 到電腦（會顯示為 USB 磁碟機）
2. 複製 hex 檔案到 micro:bit 磁碟機
3. 等待 LED 閃爍完成

---

## 🎯 遊戲操作

### 控制方式
| 操作 | 按鈕 | 效果 |
|------|------|------|
| 向左轉 | Button A | 方向逆時針旋轉 90° |
| 向右轉 | Button B | 方向順時針旋轉 90° |
| 重置遊戲 | Reset 按鈕 | 重新啟動 micro:bit |

### 遊戲畫面
```
蛇頭/蛇身：● ● ● （持續閃爍）
食  物  ：◐ （閃爍，偶數幀消失）
空  白  ：·
```

### 遊戲技巧
1. **提前轉向**：由於移動速度較慢（3秒），可以提前按按鈕
2. **利用環繞**：蛇可以從邊界穿越，利用這點躲避自己
3. **規劃路線**：在蛇變長後，需要提前規劃避免撞到自己

---

## 🔍 技術特點

### 嵌入式特性
- **無標準庫**：`#![no_std]` - 不依賴作業系統
- **無動態分配**：使用 `heapless::Vec` 避免堆疊分配
- **裸機運行**：直接在硬體上運行，無 OS 開銷
- **低功耗**：ARM Cortex-M4F 處理器

### 硬體抽象
- **Timer**：精確時間控制（微秒級）
- **Display**：5×5 LED 矩陣時分復用
- **RNG**：硬體隨機數生成器
- **GPIO**：按鈕輸入檢測

### 記憶體使用
- **Stack**：約 1KB（主要是遊戲狀態）
- **Heap**：0（完全不使用堆疊）
- **Flash**：約 25KB（程式碼）
- **RAM**：約 2KB（靜態變數和棧）

---

## 🐛 已知限制

1. **最大蛇長**：25 格（整個螢幕）
2. **最大分數**：255（u8 限制）
3. **按鈕回應**：每 3 秒讀取一次（移動週期限制）
4. **閃爍頻率**：固定 110ms 週期，無法動態調整

---

## 🔧 自定義參數

如需修改遊戲參數，編輯 `src/main.rs` 的 `main()` 函數：

```rust
// 遊戲速度（毫秒）
let move_delay_ms = 3000u32;  // 改小→更快，改大→更慢

// 閃爍時間（毫秒）
let blink_on_ms = 100u32;     // 亮的時間
let blink_off_ms = 10u32;     // 暗的時間
```

修改初始設置，編輯 `Game::new()` 函數：

```rust
// 初始位置和方向
let snake = Snake::new(
    Coords::new(2, 2),    // 起始位置 (row, col)
    Direction::Right      // 起始方向
);
```

---

## 📚 延伸學習

### 相關文件
- [micro:bit V2 硬體規格](https://tech.microbit.org/hardware/)
- [Rust Embedded Book](https://docs.rust-embedded.org/book/)
- [microbit-v2 Crate 文件](https://docs.rs/microbit-v2/)

### 改進建議
1. 添加難度選擇（開機時按按鈕選擇）
2. 實現分數顯示（使用 LED 滾動數字）
3. 添加音效（使用蜂鳴器）
4. 記錄最高分（使用 Flash 存儲）
5. 多人模式（兩條蛇）

---

## 📄 授權

本專案使用 Rust 和開源社群的各種 crate。請參閱各 crate 的授權條款。

---

## 🙏 致謝

感謝以下專案和社群：
- [micro:bit Educational Foundation](https://microbit.org/)
- [Rust Embedded Working Group](https://github.com/rust-embedded)
- [nRF HAL](https://github.com/nrf-rs/nrf-hal)

---

**享受遊戲！** 🐍🎮
