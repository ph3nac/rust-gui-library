use vizia::prelude::*;

// viziaでの状態はモデルに保存される
// モデルにはModel traitを実装する任意のデータを保存することができる
// Lens deriveマクロを使用することでモデルのフィールドにアクセスするためのメソッドを自動生成することができる
// レンズオブジェクトはビューにモデルをバインドするために使用され，モデルの特定の値が変更されたときにビューを更新する
#[derive(Lens)]
pub struct AppData {
    pub count: i32,
}

impl Model for AppData {
    // イベントはViewまたはModelのeventメソッドを使用して処理する
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        // イベントで map() を呼び出すと、イベントメッセージを指定されたタイプにキャストしようとし、成功した場合は提供されたクロージャを呼び出す．
        // クロージャーはメッセージタイプとメタデータを提供し，イベントの発生元やターゲットを特定したり，イベントメッセージを消費することでイベントの伝搬を防止することができる
        event.map(|app_event, meta| match app_event {
            AppEvent::Decrement => {
                self.count -= 1;
            }
            AppEvent::Increment => {
                self.count += 1;
            }
        })
    }
}

// Viziaはイベントを使用してモデルを更新したり，ビューを表示するアクションを伝える
// イベントは，イベントを放出するビューからツリーをたどりメインインウィンドウまで伝搬する
// イベントには任意の方にできるメッセージが含まれており，通常列挙型を使用する
pub enum AppEvent {
    Increment,
    Decrement,
}

// ------------------------------------------
// countをView内に保持することもできるが，今回はステートレスなViewとする
// 代わりにモデルにバインドするLensとボタンのイベントを処理するためのコールバックを使用する
pub struct Counter {
    // AppEventへの依存を取り除くためにコールバックを追加する
    on_increment: Option<Box<dyn Fn(&mut EventContext)>>,
    on_decrement: Option<Box<dyn Fn(&mut EventContext)>>,
}

// ユーザーがコールバックを追加できるようにするためにCounterにCounterModifiersトレイトを定義する
pub trait CounterModifiers {
    // 'staticライフタイムはコールバックがプログラム全体で有効であることを示す
    // callbackはEventContextを受け取り，何らかのアクションを実行する
    fn on_increment<F: Fn(&mut EventContext) + 'static>(self, callback: F) -> Self;
    fn on_decrement<F: Fn(&mut EventContext) + 'static>(self, callback: F) -> Self;
}

// CounterModifiersトレイトをHandle<'a, Counter>に実装する
// 'aはHandleのライフタイムパラメータで，Counterのライフタイムを指定する．
// HandleはViewを操作するためのハンドルで，Viewの状態を変更するためのメソッドを提供する
impl<'a> CounterModifiers for Handle<'a, Counter> {
    // Handleのmodifyメソッドを使用することで直接callbackを追加することができる
    fn on_decrement<F: Fn(&mut EventContext) + 'static>(self, callback: F) -> Self {
        self.modify(|counter| counter.on_decrement = Some(Box::new(callback)))
    }
    fn on_increment<F: Fn(&mut EventContext) + 'static>(self, callback: F) -> Self {
        self.modify(|counter| counter.on_increment = Some(Box::new(callback)))
    }
}

// ボタンから発行するイベントを作成する
pub enum CounterEvent {
    Increment,
    Decrement,
}

// View traitを実装することでビューを定義する
impl View for Counter {
    // 動的に追加されるコールバックをイベントによって呼び出す
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|e, _meta| match e {
            CounterEvent::Increment => {
                if let Some(callback) = &self.on_increment {
                    (callback)(cx);
                }
            }
            CounterEvent::Decrement => {
                if let Some(callback) = &self.on_decrement {
                    (callback)(cx);
                }
            }
        });
    }
}

impl Counter {
    // Viewを使用するにはコンストラクタでViewをContextに追加する必要がある
    // データバインディングを追加するにはコンストラクタでLensを引数に渡す必要がある．またLensはジェネリックを使用してLens traitを実装する任意の型を受け取っている
    pub fn new<L>(cx: &mut Context, lens: L) -> Handle<Self>
    where
        L: Lens<Target = i32>,
    {
        // Viewトレイトによって提供される build()関数は、カスタムViewのコンテンツを構築するために使用できるクロージャを引数に取る。
        Self {
            // エラーになるため,初期化時にコールバックをNoneに設定する
            on_increment: None,
            on_decrement: None,
        }
        .build(cx, |cx| {
            // アプリケーションにビューを追加する
            // ビューの構成はHStackのようなコンテナビューを使って行う
            // HStackは水平方向にビューを並べる
            // デフォルトではHStackは親ビュー(window)を埋めるように拡張される
            // レイアウトシステムについてはmorphormのドキュメントを参照
            HStack::new(cx, |cx| {
                // ボタンを追加する
                Button::new(cx, |cx| Label::new(cx, "Decrement"))
                    // EventContextを使用してイベントを発行する
                    // ツリーを辿ってAppDataモデルに伝搬される
                    .on_press(|ex| ex.emit(CounterEvent::Decrement))
                    .class("dec");
                Button::new(cx, |cx| Label::new(cx, "Increment"))
                    .on_press(|ex| ex.emit(CounterEvent::Increment))
                    .class("inc");
                // countが更新されるたび，ビューを更新するバインディングが設定される
                Label::new(cx, lens).class("count");
            })
            .class("row");
        })
    }
}
// ------------------------------------------

fn main() {
    // アプリケーションを初期化する
    // クロージャ内でContextを受け取り，ビューを追加していく
    Application::new(|cx| {
        // buildメソッドを使用することでアプリケーションに状態を追加する
        // これによりモデルデータがツリーに組み込まれる．今回の場合root windowに関連付けられる
        AppData { count: 0 }.build(cx);

        Counter::new(cx, AppData::count)
            .on_increment(|cx| cx.emit(AppEvent::Increment))
            .on_decrement(|cx| cx.emit(AppEvent::Decrement));

        // アプリケーションにスタイルを適用する
        cx.add_stylesheet(include_style!("src/style.css"))
            .expect("Failed to load stylesheet");
    })
    .title("Counter")
    .inner_size((800, 300))
    .run()
    .unwrap();
}
