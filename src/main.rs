#![no_main]
#![no_std]

use cortex_m_rt::entry;
use embedded_hal::digital::InputPin;
use heapless::Vec;
use microbit::{
    board::Board,
    display::blocking::Display,
    hal::{timer::Instance, Rng, Timer},
};
use panic_halt as _;

// ===== 座標系統 =====

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Coords {
    row: i8,
    col: i8,
}

impl Coords {
    fn new(row: i8, col: i8) -> Self {
        Self { row, col }
    }

    fn random(rng: &mut Rng) -> Self {
        Self {
            row: (rng.random_u8() % 5) as i8,
            col: (rng.random_u8() % 5) as i8,
        }
    }

    fn wrap(&self) -> Self {
        Self {
            row: ((self.row % 5) + 5) % 5,
            col: ((self.col % 5) + 5) % 5,
        }
    }
}

// ===== 方向 =====

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn turn_left(&self) -> Self {
        match self {
            Direction::Up => Direction::Left,
            Direction::Left => Direction::Down,
            Direction::Down => Direction::Right,
            Direction::Right => Direction::Up,
        }
    }

    fn turn_right(&self) -> Self {
        match self {
            Direction::Up => Direction::Right,
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
        }
    }

    fn step(&self, coords: Coords) -> Coords {
        match self {
            Direction::Up => Coords::new(coords.row - 1, coords.col),
            Direction::Down => Coords::new(coords.row + 1, coords.col),
            Direction::Left => Coords::new(coords.row, coords.col - 1),
            Direction::Right => Coords::new(coords.row, coords.col + 1),
        }
    }

    fn opposite(&self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

// ===== 轉向指令 =====

#[derive(Copy, Clone)]
enum Turn {
    Left,
    Right,
    None,
}

// ===== 按鍵狀態追蹤 =====

struct ButtonState {
    a_was_pressed: bool, // 上一次 A 按鈕是否被按下
    b_was_pressed: bool, // 上一次 B 按鈕是否被按下
}

impl ButtonState {
    fn new() -> Self {
        Self {
            a_was_pressed: false,
            b_was_pressed: false,
        }
    }
}

// ===== 遊戲狀態 =====

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum GameStatus {
    Ongoing,
    GameOver,
}

// ===== 蛇 =====

struct Snake {
    head: Coords,
    body: Vec<Coords, 25>,
    direction: Direction,
}

impl Snake {
    fn new(start_pos: Coords, direction: Direction) -> Self {
        let mut body = Vec::new();

        // 初始蛇長為 3，尾部往相反方向延伸
        let _ = body.push(start_pos); // 頭部
        let opposite_dir = direction.opposite();
        let tail1 = opposite_dir.step(start_pos).wrap();
        //let tail2 = opposite_dir.step(tail1).wrap();
        let _ = body.push(tail1);
        //let _ = body.push(tail2);

        Self {
            head: start_pos,
            body,
            direction,
        }
    }

    fn collides_with_self(&self, pos: Coords) -> bool {
        self.body.iter().any(|&segment| segment == pos)
    }

    fn move_forward(&mut self) {
        let new_head = self.direction.step(self.head).wrap();
        self.head = new_head;
        self.body.insert(0, new_head).ok();
        let _ = self.body.pop();
    }

    fn move_and_grow(&mut self) {
        let new_head = self.direction.step(self.head).wrap();
        self.head = new_head;
        self.body.insert(0, new_head).ok();
    }
}

// ===== 遊戲 =====

struct Game {
    snake: Snake,
    food: Coords,
    status: GameStatus,
    score: u8,
    snake_blink_counter: u32, // 蛇閃爍計數器
    food_blink_counter: u32,  // 食物閃爍計數器
    pending_turn: Turn,       // 待處理的轉向指令
    difficulty: u8,           // 難度（吃幾個食物增長一個長度）
}

impl Game {
    fn new(rng: &mut Rng, difficulty: u8) -> Self {
        let snake = Snake::new(Coords::new(2, 2), Direction::Right);

        let mut food = Coords::random(rng);
        while snake.collides_with_self(food) {
            food = Coords::random(rng);
        }

        Self {
            snake,
            food,
            status: GameStatus::Ongoing,
            score: 0,
            snake_blink_counter: 0,
            food_blink_counter: 0,
            pending_turn: Turn::None,
            difficulty,
        }
    }

    fn queue_turn(&mut self, turn: Turn) {
        // 只有在沒有待處理的轉向時才接受新的轉向
        if matches!(self.pending_turn, Turn::None) {
            self.pending_turn = turn;
        }
    }

    fn apply_pending_turn(&mut self) {
        self.snake.direction = match self.pending_turn {
            Turn::Left => self.snake.direction.turn_left(),
            Turn::Right => self.snake.direction.turn_right(),
            Turn::None => self.snake.direction,
        };
        // 清除已處理的轉向
        self.pending_turn = Turn::None;
    }

    fn step(&mut self, rng: &mut Rng) {
        if self.status != GameStatus::Ongoing {
            return;
        }

        let next_pos = self.snake.direction.step(self.snake.head).wrap();

        // 碰撞檢測（跳過頭部，因為頭部總是在 body[0]）
        if self
            .snake
            .body
            .iter()
            .skip(1)
            .any(|&segment| segment == next_pos)
        {
            self.status = GameStatus::GameOver;
            return;
        }

        // 檢查是否吃到食物
        if next_pos == self.food {
            self.score += 1;

            // 根據難度決定是否增長
            if self.score % self.difficulty == 0 {
                self.snake.move_and_grow();
            } else {
                self.snake.move_forward();
            }

            // 生成新食物
            self.food = Coords::random(rng);
            while self.snake.collides_with_self(self.food) {
                self.food = Coords::random(rng);
            }
        } else {
            self.snake.move_forward();
        }
    }

    fn render(&self, show_snake: bool, show_food: bool) -> [[u8; 5]; 5] {
        let mut matrix = [[0u8; 5]; 5];

        // 根據閃爍狀態繪製蛇
        if show_snake {
            // 繪製蛇身（跳過頭部）
            for segment in self.snake.body.iter().skip(1) {
                matrix[segment.row as usize][segment.col as usize] = 9;
            }

            // 繪製蛇頭
            matrix[self.snake.head.row as usize][self.snake.head.col as usize] = 9;
        }

        // 根據閃爍狀態繪製食物
        if show_food {
            matrix[self.food.row as usize][self.food.col as usize] = 9;
        }

        matrix
    }

    fn update_blink_counters(&mut self) {
        self.snake_blink_counter += 1;
        self.food_blink_counter += 1;
    }
}

// ===== 輸入處理 =====

fn read_buttons(buttons: &mut microbit::board::Buttons, state: &mut ButtonState) -> Turn {
    let a_pressed = buttons.button_a.is_low().unwrap();
    let b_pressed = buttons.button_b.is_low().unwrap();

    // 檢測 A 按鈕的上升沿（從未按下到按下的瞬間）
    let a_just_pressed = a_pressed && !state.a_was_pressed;

    // 檢測 B 按鈕的上升沿（從未按下到按下的瞬間）
    let b_just_pressed = b_pressed && !state.b_was_pressed;

    // 更新狀態
    state.a_was_pressed = a_pressed;
    state.b_was_pressed = b_pressed;

    // 只有在剛按下的瞬間才返回轉向指令
    match (a_just_pressed, b_just_pressed) {
        (true, false) => Turn::Left,
        (false, true) => Turn::Right,
        _ => Turn::None,
    }
}

// ===== 開始畫面和難度選擇 =====

fn show_start_screen<T: Instance>(
    display: &mut Display,
    timer: &mut Timer<T>,
    buttons: &mut microbit::board::Buttons,
    button_state: &mut ButtonState,
) -> u8 {
    let mut difficulty = 5u8; // 預設難度：吃5個食物增長1個長度

    // 清空按鍵狀態，等待所有按鍵釋放
    loop {
        let a_pressed = buttons.button_a.is_low().unwrap();
        let b_pressed = buttons.button_b.is_low().unwrap();

        if !a_pressed && !b_pressed {
            // 兩個按鍵都釋放了，重置狀態
            button_state.a_was_pressed = false;
            button_state.b_was_pressed = false;
            break;
        }

        // 繼續顯示當前難度
        let mut matrix = [[0u8; 5]; 5];
        for i in 0..difficulty as usize {
            matrix[0][i] = 9;
        }
        display.show(timer, matrix, 50);
    }

    loop {
        // 顯示難度（第一排的 LED 數量）
        let mut matrix = [[0u8; 5]; 5];
        for i in 0..difficulty as usize {
            matrix[0][i] = 9; // 第一排亮起對應數量的 LED
        }

        display.show(timer, matrix, 100);

        // 檢查按鍵
        let turn = read_buttons(buttons, button_state);

        match turn {
            Turn::Right => {
                // 按 B：降低難度（輪迴遞減：5->4->3->2->1->5）
                difficulty = if difficulty > 1 { difficulty - 1 } else { 5 };
            }
            Turn::Left => {
                // 按 A：開始遊戲
                return difficulty;
            }
            Turn::None => {}
        }
    }
}

// ===== 遊戲結束畫面 =====

fn show_game_over<T: Instance>(display: &mut Display, timer: &mut Timer<T>) {
    let x_pattern = [
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 9, 0, 9, 0],
        [0, 0, 9, 0, 0],
        [0, 9, 0, 9, 0],
    ];

    // 閃爍 3 次
    for _ in 0..3 {
        display.show(timer, x_pattern, 480);
        display.show(timer, [[0; 5]; 5], 20);
    }
}

// ===== 主程式 =====

#[entry]
fn main() -> ! {
    // 初始化硬體
    let board = Board::take().unwrap();
    let mut timer = Timer::new(board.TIMER0);
    let mut display = Display::new(board.display_pins);
    let mut rng = Rng::new(board.RNG);
    let mut buttons = board.buttons;

    let mut button_state = ButtonState::new();

    // 遊戲參數
    let move_delay_ms = 900u32; // 移動週期（1秒）
    let base_on_ms = 24u32; // 基礎亮時間（99毫秒）
    let base_off_ms = 1u32; // 基礎暗時間（1毫秒）
    let base_cycle_ms = base_on_ms + base_off_ms; // 基礎週期（100毫秒）

    let snake_blink_multiplier = 1u32; // 蛇的閃爍倍率
    let food_blink_multiplier = 12u32; // 食物的閃爍倍率

    // 顯示開始畫面並選擇難度
    let difficulty = show_start_screen(&mut display, &mut timer, &mut buttons, &mut button_state);

    // 創建遊戲（開始畫面函數已經重置了按鍵狀態）
    let mut game = Game::new(&mut rng, difficulty);

    // 主遊戲循環
    loop {
        // 1. 應用待處理的轉向並移動蛇
        game.apply_pending_turn();
        game.step(&mut rng);

        // 2. 檢查遊戲結束
        if game.status == GameStatus::GameOver {
            show_game_over(&mut display, &mut timer);

            // 重新顯示開始畫面選擇難度（函數內會重置按鍵狀態）
            let difficulty =
                show_start_screen(&mut display, &mut timer, &mut buttons, &mut button_state);
            game = Game::new(&mut rng, difficulty);
            continue;
        }

        // 3. 在移動週期內持續顯示和閃爍，並持續檢測按鍵
        let num_base_cycles = move_delay_ms / base_cycle_ms; // 每個移動週期有多少個基礎週期

        for _ in 0..num_base_cycles {
            // 在每個基礎週期都檢測按鍵
            let turn = read_buttons(&mut buttons, &mut button_state);
            if !matches!(turn, Turn::None) {
                game.queue_turn(turn);
            }

            // 更新閃爍計數器
            game.update_blink_counters();

            // 計算蛇是否顯示（亮 snake_multiplier 個週期，暗 snake_multiplier 個週期）
            let snake_cycle_pos = game.snake_blink_counter % (snake_blink_multiplier * 2);
            let show_snake = snake_cycle_pos < snake_blink_multiplier;

            // 計算食物是否顯示（亮 food_multiplier 個週期，暗 food_multiplier 個週期）
            let food_cycle_pos = game.food_blink_counter % (food_blink_multiplier * 2);
            let show_food = food_cycle_pos < food_blink_multiplier;

            // 渲染畫面
            let matrix = game.render(show_snake, show_food);

            // 顯示基礎亮時間
            display.show(&mut timer, matrix, base_on_ms);

            // 顯示基礎暗時間（全暗）
            display.show(&mut timer, [[0; 5]; 5], base_off_ms);
        }
    }
}
