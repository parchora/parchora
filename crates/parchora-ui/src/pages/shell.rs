//! PG-SHELL 面板外壳（UI-2 · TASK-500 PanelView 口径）。
//!
//! 结构（PG-SHELL.html 1:1 壳层口径）：bar（tabs 集锦/拾记/待办 + Pin/⚙）+ 中部视图内容
//! 区（fill）。右下 FAB（plus·54 圆）由页面视图自身承载——PG-A/PG-B/PG-C 每页 1:1 自带
//! 一枚；壳层不再叠加（DEC-70 去重）。图标（DEC-66）：Pin=pin（选中
//! 态＝背景主墨 t::PRIMARY＋纸光反白前景（MUTED 于主墨底 ~1.15:1 失读必须切
//! ON_PRIMARY）；未选中＝纸面底＋muted（DEC-124 背景轴））· ⚙=settings · FAB ＋=plus（页面层）。
//!
//! 演示宿主的交互总线（db/files/capture 为存根，示例数据取自 HTML 设计稿同源）：
//! - tabs 切换三视图（DEC-56：capture/collection/todo 三值）；
//! - Pin/⚙：Pin 为会话态翻转（翻转时上抛 PinChanged 同步宿主全局口径 · DEC-94；
//!   轮询收起/失活探测/点击外收起三路读 app 层单一事实源）；⚙ 经 app 层开设置窗
//!   （事件上抛，见 AppEvent）；鼠标移出面板根（非 Pin）上抛 MouseExitHide，
//!   宿主收面板（全隐）—— DEC-67 移出隐藏路径 REQ-PANEL-HIDE 的演示承载；
//! - 行单击 Peek：三子视图单击条目经 PreviewSink 统一映射为
//!   ShellEvent::PeekShow 上抛宿主（单上抛口 · peek.rs）；
//! - Esc：面板根 on_key_down 分派——次级浮层（队列层/确认 sheet/详情区）先关，
//!   级联末级上抛 ShellEvent::HidePanel 由宿主收面板（全隐 · §〇-4 · DEC-73）。
//!
//! 面板「停放屏外/不夺焦点/双材质」语义由 parchora-platform（DEC-59/60）承载。

use std::rc::Rc;

use gpui::{
    div, px, rgb, App, AppContext as _, ClickEvent, Context, Div, Entity, HoverListenerMode,
    InteractiveElement, ParentElement as _, Render, StatefulInteractiveElement as _, Styled,
    Window,
};

use crate::{
    components as co,
    pages::{collect, collection, peek, tasks},
    scale as s, tokens as t,
};

/// 视图枚举（TASK-500：current_view 三值）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    /// 拾记（PG-A）
    Capture,
    /// 集锦（PG-B）
    Collection,
    /// 待办（PG-C）
    Todo,
}

/// 面板 → 宿主（parchora-app）的上抛事件（演示宿主接线：⚙ 开设置窗）。
pub enum ShellEvent {
    /// 打开设置独立窗口（⌘, / ⚙）
    OpenSettings,
    /// 新建笔记（集锦层 FAB「新建笔记」上抛 → 宿主开/热换编辑区空稿；
    /// ⌘N 触发点留 TASK-502）
    QuickNote,
    /// 集锦首页笔记行「删除→回收站」（DEC-164 · 携题名词干）：宿主将 files 根面
    /// 该笔记 rename 移入同级 `.trash/` 并回灌列表（REQ-STORE-TRASH 文件侧切片；
    /// DB trash 记录/60 天清理随 trash crate 接线 TASK-50x）。
    TrashNote(String),
    /// 待办页 FAB「新建任务」上抛（DEC-165）：宿主开/激活「创建待办事项」独立
    /// 面板窗（单实例幂等 settings 先例；与 QuickNote 编辑窗通道同式；表单迁移
    /// 面板 · 提交经 TaskCreateEvent 回灌 BoardView.ingest_created）。
    QuickTask,
    /// 打开队列浮层演示
    OpenQueue,
    /// 拾记右键「转为笔记」上抛（DEC-194 · 携条目原文）：宿主开/热换「创建
    /// 集锦」编辑窗预填正文（题名留 Untitled 由用户定夺）；与 QuickNote 编辑
    /// 窗通道同式弱引用转发。
    ClipToNote(String),
    /// 拾记右键「转为任务」上抛（DEC-194 · 携条目原文）：宿主开/激活「新建
    /// 任务」面板并预填「任务」多行域（题名不动）。
    ClipToTask(String),
    /// 鼠标移出面板（非 Pin）→ 宿主收面板（全隐 · DEC-67 移出隐藏路径）
    MouseExitHide,
    /// Esc 级联末级（无次级浮层可关）→ 宿主收面板（全隐 · §〇-4 · DEC-73）
    HidePanel,
    /// 单击 Peek 显示条目（宿主开内容详情窗/原位更新 · 单实例）
    PeekShow(peek::PeekItem),
    /// Pin 翻转（宿主全局口径同步：轮询收起/失活探测/点击外/arm_peek_close 四
    /// 隐藏路径读 app 层单一事实源 · SCN-PANEL-PIN-BLOCKS-HIDE · DEC-94）
    PinChanged(bool),
}

/// 面板 → 宿主（parchora-app）的上抛事件。
pub type ShellSink = Box<dyn Fn(&ShellEvent, &mut App) + 'static>;

/// 面板外壳视图。
pub struct PanelView {
    focus: gpui::FocusHandle,
    view: View,
    pinned: bool,
    capture: Entity<collect::CaptureView>,
    collection: Entity<collection::CollectionView>,
    board: Entity<tasks::BoardView>,
    /// 面板 → 宿主事件
    on_shell_event: Option<ShellSink>,
}

impl PanelView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let capture = cx.new(collect::CaptureView::new);
        let collection = cx.new(collection::CollectionView::new);
        let board = cx.new(tasks::BoardView::new);
        // 单击 Peek 事件统一转发（PeekItem → ShellEvent::PeekShow；weak 上抛
        // 避免持有自环 strong Entity）。
        let forward = cx.weak_entity();
        let sink: peek::PreviewSink = Rc::new(move |item: &peek::PeekItem, cx: &mut App| {
            let _ = forward.update(cx, |panel, cx| {
                panel.emit(ShellEvent::PeekShow(item.clone()), cx)
            });
        });
        capture.update(cx, |v, _| v.set_peek_sink(sink.clone()));
        collection.update(cx, |v, _| v.set_peek_sink(sink.clone()));
        board.update(cx, |v, _| v.set_peek_sink(sink.clone()));
        // 集锦层 FAB「新建笔记」上抛通道（弱引用转发与 Peek 转发同模式防自环
        // 持有）：FAB 点击 → ShellEvent::QuickNote → 宿主开/热换编辑窗。
        let note_forward = cx.weak_entity();
        let note_sink: collection::NoteSink = Rc::new(move |cx: &mut App| {
            let _ = note_forward.update(cx, |panel, cx| panel.emit(ShellEvent::QuickNote, cx));
        });
        collection.update(cx, |v, _| v.set_note_sink(note_sink));
        // 集锦首页笔记行「删除→回收站」上抛通道（DEC-164 · 弱引用转发同式）：
        // 菜单「删除」→ ShellEvent::TrashNote(题名) → 宿主真移 .trash/ 并回灌列表。
        let trash_forward = cx.weak_entity();
        let trash_sink: collection::TrashSink = Rc::new(move |title: &str, cx: &mut App| {
            let _ = trash_forward.update(cx, |panel, cx| {
                panel.emit(ShellEvent::TrashNote(title.to_string()), cx)
            });
        });
        collection.update(cx, |v, _| v.set_trash_sink(trash_sink));
        // 待办页 FAB「新建任务」上抛通道（DEC-165 · 弱引用转发与 Note 同模式防
        // 自环持有）：FAB 点击 → ShellEvent::QuickTask → 宿主开/激活创建面板窗。
        let task_forward = cx.weak_entity();
        let task_sink: tasks::TaskSink = Rc::new(move |cx: &mut App| {
            let _ = task_forward.update(cx, |panel, cx| panel.emit(ShellEvent::QuickTask, cx));
        });
        board.update(cx, |v, _| v.set_task_sink(task_sink));
        // 拾记右键「转为笔记/转为任务」上抛通道（DEC-194 · 弱引用转发与 QuickTask
        // 同模式防自环持有）：条目原文 → ShellEvent::ClipToNote/ClipToTask →
        // 宿主开/热换编辑窗或开/激活创建面板预填。
        let conv_note_forward = cx.weak_entity();
        let conv_note_sink: collect::ClipConvertSink =
            Rc::new(move |content: &str, cx: &mut App| {
                let _ = conv_note_forward.update(cx, |panel, cx| {
                    panel.emit(ShellEvent::ClipToNote(content.to_string()), cx)
                });
            });
        capture.update(cx, |v, _| v.set_clip_to_note_sink(conv_note_sink));
        let conv_task_forward = cx.weak_entity();
        let conv_task_sink: collect::ClipConvertSink =
            Rc::new(move |content: &str, cx: &mut App| {
                let _ = conv_task_forward.update(cx, |panel, cx| {
                    panel.emit(ShellEvent::ClipToTask(content.to_string()), cx)
                });
            });
        capture.update(cx, |v, _| v.set_clip_to_task_sink(conv_task_sink));
        Self {
            focus: cx.focus_handle(),
            // 默认视图 = 集锦（聚合首页 · tab 序位首 · DEC-78）。
            view: View::Collection,
            pinned: false,
            capture,
            collection,
            board,
            on_shell_event: None,
        }
    }

    pub fn focus_handle(&self) -> gpui::FocusHandle {
        self.focus.clone()
    }

    pub fn set_event_sink(&mut self, sink: ShellSink) {
        self.on_shell_event = Some(sink);
    }

    /// 选项卡活值读点（DEC-181 显隐保持）：宿主 close_panel 整窗移除前记账用
    ///（TASK-500 current_view 三值口径命名同源）。
    pub fn current_view(&self) -> View {
        self.view
    }

    /// 选项卡恢复（DEC-181「显隐切换保持选项卡」宿主重开链落点）：
    /// 重建实体首帧前落值＋显式 notify（渲染同步契约）——tab 选中与激活态均
    /// 绑定 view 相等（co::tab 第二参），恢复即选中态随首帧同步。
    pub fn restore_view(&mut self, v: View, cx: &mut Context<Self>) {
        self.view = v;
        cx.notify();
    }

    /// Pin 复位（DEC-186 实体复用口径）：close_panel 提取实体跨窗存续——面板
    /// 侧 Pin 须与关闭后全局 pinned＝false 同源（DEC-94 单一事实源），否则
    /// 复用实体重开会带出陈旧 Pin（轮询收起域错误长开）；不上抛不改全局。
    pub fn reset_pin(&mut self, cx: &mut Context<Self>) {
        self.pinned = false;
        cx.notify();
    }

    /// 集锦页磁盘真实笔记注入转发（DEC-144）：宿主扫描 files 根 → 全量替换注入
    /// 集锦浏览列表（幂等刷新，唯一入口 CollectionView::set_disk_notes）。
    pub fn set_disk_notes(&mut self, notes: Vec<collection::DiskNote>, cx: &mut Context<Self>) {
        self.collection
            .update(cx, |v, cx| v.set_disk_notes(notes, cx));
    }

    /// 回收站真实条目注入转发（DEC-195 · 镜像 set_disk_notes）：宿主扫描 files
    /// 同级 `.trash/` → 全量替换注入回收站页列表（幂等刷新 · 唯一入口
    /// CollectionView::set_trash_notes）。
    pub fn set_trash_notes(&mut self, notes: Vec<collection::DiskNote>, cx: &mut Context<Self>) {
        self.collection
            .update(cx, |v, cx| v.set_trash_notes(notes, cx));
    }

    /// 回收站页删除动作通道注入转发（DEC-195 · 镜像 set_clip_delete_sink）：
    /// Purge/Empty 动作 → 宿主 defer 物理删除并回灌（复用实体重复注入幂等——
    /// sink 恒槽覆写）。
    pub fn set_trash_action_sink(
        &mut self,
        sink: collection::TrashActionSink,
        cx: &mut Context<Self>,
    ) {
        self.collection
            .update(cx, |v, _| v.set_trash_action_sink(sink));
    }

    /// 拾记捕获注入转发（DEC-163 · DEC-183 捕获点墙钟随注入）：app 层剪贴
    /// 板守护读得新文本 → 拾记列表顶插入（面板不在场时入 app 层 pending 缓冲）。
    pub fn ingest_clip(
        &mut self,
        text: String,
        at: std::time::Instant,
        wall: i64,
        cx: &mut Context<Self>,
    ) {
        self.capture
            .update(cx, |v, cx| v.ingest_clip(text, at, wall, cx));
    }

    /// 拾记行完整形态注入转发（TASK-300）：宿主组行 → 拾记列表顶插入（面板
    /// 不在场时行已落 app 层 pending 缓冲）。
    pub fn ingest_row(&mut self, row: collect::ClipRow, cx: &mut Context<Self>) {
        self.capture.update(cx, |v, cx| v.ingest_row(row, cx));
    }

    /// 队列镜像注入转发（DEC-207 · set_disk_notes 同构）：宿主 QueueHost
    /// 动作后/面板重开链全量替换队列表（消费序已排齐 · 幂等唯一入口）。
    pub fn set_queue_rows(&mut self, rows: Vec<collect::QueueRow>, cx: &mut Context<Self>) {
        self.capture.update(cx, |v, cx| v.set_queue_rows(rows, cx));
    }

    /// 队列按钮 bounce 注入转发（TASK-716 · §11.2 面板动效通道）：宿主提示
    /// 编排产出 PanelBounce 动作 → 拾记视图入口 chip 触发一轮脉冲
    ///（幂等自增序承载——同一拍多次注入只计一次序递增）。
    pub fn set_queue_bounce(&mut self, cx: &mut Context<Self>) {
        self.capture.update(cx, |v, cx| v.queue_chip_bounce(cx));
    }

    /// 队列动作通道注入转发（DEC-207 · set_trash_action_sink 同链接线）：
    /// 入队/切换/停止/清空上拋宿主 mutator（重复注入幂等——恒槽覆写）。
    pub fn set_queue_action_sink(
        &mut self,
        sink: collect::QueueActionSink,
        cx: &mut Context<Self>,
    ) {
        self.capture
            .update(cx, |v, _| v.set_queue_action_sink(sink));
    }

    /// 拾记行题名披露（DEC-186 · app 层跨关板存续验证读点）：仅读不写。
    pub fn clip_titles(&self, cx: &App) -> Vec<String> {
        self.capture.read(cx).clip_titles()
    }

    /// 队列镜像披露（DEC-207 · clip_titles 同式仅读不写）：app 层 QueueHost
    /// 回推验证读点，列序即镜像呈现序（首项＝placed）。
    pub fn queue_texts(&self, cx: &App) -> Vec<String> {
        self.capture.read(cx).queue_texts()
    }

    /// 拾记条目存活清理转发（DEC-163 · DEC-188 增计数上限档）：两阈值（分钟）
    /// 与非 Pin 计数上限（0=无限）自 app 层配置档全局读出。
    pub fn sweep_clip(
        &mut self,
        survive_min: u64,
        valid_min: u64,
        limit: u64,
        cx: &mut Context<Self>,
    ) {
        self.capture
            .update(cx, |v, cx| v.sweep_clip(survive_min, valid_min, limit, cx));
    }

    /// 拾记行 DB 身份披露（DEC-188 · D-06 回填断言读点）：仅读不写。
    pub fn clip_db_ids(&self, cx: &App) -> Vec<Option<String>> {
        self.capture.read(cx).clip_db_ids()
    }

    /// 重试补写成功 DB 身份回填转发（DEC-188 · D-06）：按加入时刻对位命中行。
    pub fn update_clip_db_ids(
        &mut self,
        hits: &[(std::time::Instant, String)],
        cx: &mut Context<Self>,
    ) -> usize {
        self.capture.update(cx, |v, cx| v.update_db_ids(hits, cx))
    }

    /// 待办创建面板提交回灌转发（DEC-165）：宿主提交漏斗 → BoardView
    /// ::ingest_created（入行/序号/成功 Toast 权威在 BoardView；面板亡时宿主
    /// stderr 披露不入行）。
    pub fn ingest_created_task(
        &mut self,
        title: String,
        bucket: usize,
        priority: usize,
        mark: usize,
        cx: &mut Context<Self>,
    ) {
        self.board.update(cx, |v, cx| {
            v.ingest_created(title, bucket, priority, mark, cx)
        });
    }

    // ── TASK-300 持久化链转发（宿主装配位，弱引用/复用实体同式幂等）──

    /// 拾记行删除通道注入转发：右键「删除」→ CaptureView 上抛 db_id → 宿主
    /// 落 DB 删除（复用实体重复注入幂等——sink 恒槽覆写）。
    pub fn set_clip_delete_sink(&mut self, sink: collect::ClipDeleteSink, cx: &mut Context<Self>) {
        self.capture.update(cx, |v, _| v.set_delete_sink(sink));
    }

    /// 待办 DB 事件通道注入转发：Created/Status/Bucket 事件 → 宿主事务写。
    pub fn set_task_db_sink(&mut self, sink: tasks::TaskDbSink, cx: &mut Context<Self>) {
        self.board.update(cx, |v, _| v.set_db_sink(sink));
    }

    /// 回填种子转发（TASK-300 启动恢复 · 仅全新实体首次装配调用）：拾记行
    /// 全集 + 待办行集与序号续接，一次落板。
    pub fn seed_data(
        &mut self,
        clip_rows: Vec<collect::ClipRow>,
        board_rows: Vec<(String, String, usize, usize, usize, usize)>,
        next_seq: usize,
        cx: &mut Context<Self>,
    ) {
        self.capture.update(cx, |v, cx| v.seed_rows(clip_rows, cx));
        self.board
            .update(cx, |v, cx| v.seed_created(board_rows, next_seq, cx));
    }

    /// 待办行集序号读点（TASK-300 诊断/测试面）：行题名 + next_seq 当前值。
    pub fn board_rows_snapshot(&self, cx: &App) -> (Vec<String>, usize) {
        let view = self.board.read(cx);
        (view.board_titles(), view.next_seq_snapshot())
    }

    /// 待办子视图借出（TASK-300 测试/诊断面：右键事件落库链的宿主侧访问点；
    /// 生产交互链不直接越层级操作子视图）。仅借出实体句柄，不破封装租借域。
    pub fn board(&self) -> gpui::Entity<tasks::BoardView> {
        self.board.clone()
    }

    /// 拾记子视图借出（与 board() 同口径）。
    pub fn capture(&self) -> gpui::Entity<collect::CaptureView> {
        self.capture.clone()
    }

    /// 待办行题名披露（DEC-186 · 同上读点）：仅读不写。
    pub fn board_titles(&self, cx: &App) -> Vec<String> {
        self.board.read(cx).board_titles()
    }

    /// 待办创建面板空稿收口披露转发（DEC-165 · BackEmpty 漏斗落点 → 列表 Toast）。
    pub fn task_create_closed_empty(&mut self, cx: &mut Context<Self>) {
        self.board
            .update(cx, |v, cx| v.task_create_closed_empty(cx));
    }

    /// 面板 → 宿主上抛（生产链：子视图转发闭包于 PanelView 租借态内调用 →
    /// app sink 必须 defer 出借域；app 层回归测试以同构方式在租借内直达本口）。
    pub fn emit(&self, ev: ShellEvent, cx: &mut App) {
        if let Some(f) = &self.on_shell_event {
            f(&ev, cx);
        }
    }

    fn on_capture(&mut self, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>) {
        self.on_tab(View::Capture, cx);
    }

    fn on_collection(&mut self, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>) {
        self.on_tab(View::Collection, cx);
    }

    fn on_todo(&mut self, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>) {
        self.on_tab(View::Todo, cx);
    }

    fn on_tab(&mut self, v: View, cx: &mut Context<Self>) {
        self.view = v;
        cx.notify();
    }

    fn toggle_pin(&mut self, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>) {
        self.pinned = !self.pinned;
        cx.notify();
        // Pin 全局口径同步（DEC-94）：上抛仅触全局（app 层置 DockHandles.pinned），
        // 租借铁律豁免同步直呼（DEC-77：仅触全局/其它实体的臂可同步）。
        self.emit(ShellEvent::PinChanged(self.pinned), cx);
    }

    /// Esc 级联（§〇-4：次级浮层先关，级联末级才关面板 · DEC-73）。
    /// 返回是否消化了本键（有浮层被关 = 消化）。
    fn dismiss_cascade(&mut self, cx: &mut Context<Self>) -> bool {
        let mut handled = false;
        match self.view {
            View::Capture => self
                .capture
                .update(cx, |v, cx| handled = v.dismiss_overlay(cx)),
            View::Collection => self
                .collection
                .update(cx, |v, cx| handled = v.dismiss_overlay(cx)),
            View::Todo => self
                .board
                .update(cx, |v, cx| handled = v.dismiss_overlay(cx)),
        }
        handled
    }

    fn on_key_down(&mut self, ev: &gpui::KeyDownEvent, _w: &mut Window, cx: &mut Context<Self>) {
        if ev.keystroke.key != "escape" {
            return;
        }
        if self.dismiss_cascade(cx) {
            cx.stop_propagation();
            return;
        }
        // 末级：无次级浮层可关 → 宿主收面板全隐（级联末级才关面板）。
        self.emit(ShellEvent::HidePanel, cx);
    }
}

impl gpui::Focusable for PanelView {
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.focus.clone()
    }
}

/// Pin 两态配色映射（DEC-124 · **背景轴**，t::PRIMARY）：
/// 返回（前景 fg，背景 bg，悬停底 hover_bg）。选中＝主墨底＋纸光反白——前景必须切
/// ON_PRIMARY（MUTED 于主墨底对比仅 ~1.15:1 完全失读；ON_PRIMARY↔PRIMARY 7.3:1
/// 有 tokens.rs 测试背书）；悬停维持主墨底防反白字打回浅底失读。未选中＝ibtn 基线
/// 三元组（纸面底＋muted＋warm 悬停）。纯函数供调用点与测试共用（防轴回归）。
fn pin_palette(pinned: bool) -> (u32, u32, u32) {
    if pinned {
        (t::ON_PRIMARY, t::PRIMARY, t::PRIMARY)
    } else {
        (t::MUTED, t::SURFACE, t::WARM)
    }
}

impl Render for PanelView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let _s = s::enter(window, cx);
        let view = self.view;
        let pinned = self.pinned;
        // 选项卡次序 = 集锦/拾记/待办（DEC-78）。
        let tabs = co::tabs(14.0)
            .child(
                co::tab("集锦", view == View::Collection)
                    .id("tab-collection")
                    .on_click(cx.listener(Self::on_collection))
                    .active(co::act_border),
            )
            .child(
                co::tab("拾记", view == View::Capture)
                    .id("tab-capture")
                    .on_click(cx.listener(Self::on_capture))
                    .active(co::act_border),
            )
            .child(
                co::tab("待办", view == View::Todo)
                    .id("tab-todo")
                    .on_click(cx.listener(Self::on_todo))
                    .active(co::act_border),
            );
        let right = div()
            .flex()
            .gap(px(6.0))
            .child({
                // Pin 选中态＝背景主墨＋纸光反白（DEC-124 背景轴）；active 随态条件：
                // 选中期按下维持主墨底（act_border 米底＋反白字闪断不可读）。hover 已在
                // 同底组件内单次注册（选中期悬停同主墨 · 未选中期 warm·ibtn 基线），
                // 此处绝不再挂 .hover()（vendor hover_style 单槽 debug_assert）。
                let pin = pin_palette(pinned);
                let mut pin_btn = co::ibtn_icon_text_bg(co::Icon::Pin, "Pin", pin.0, pin.1, pin.2)
                    .id("btn-pin")
                    .on_click(cx.listener(Self::toggle_pin));
                pin_btn = if pinned {
                    pin_btn.active(|s| s.bg(rgb(t::PRIMARY)))
                } else {
                    pin_btn.active(co::act_border)
                };
                pin_btn
            })
            .child(
                co::icon_btn(co::Icon::Settings)
                    .id("btn-set")
                    .on_click(cx.listener(
                        |this, _e: &ClickEvent, _w: &mut Window, cx: &mut Context<Self>| {
                            this.emit(ShellEvent::OpenSettings, cx)
                        },
                    ))
                    .active(co::act_border),
            );
        let bar = co::bar().child(tabs).child(right);

        // 中部内容区（fill）：三视图按当前视图切换。壳层不再叠加全局 FAB——
        // FAB 由各页面视图自带（每页 1 枚，DEC-70 去重）。
        // 列表滚动视口根基（REQ-PANEL-LIST-SCROLL · DEC-106）：content 必须为
        // flex_col——gpui div 默认 Display::Block，Block 子项 auto 高=内容高，
        // 页面根的 .flex_1()（集锦五态/待办）在 Block 父下失效致高度无界，
        // overflow_y_scroll 视口永不约束（列表不滚、溢出被壳层裁切）。
        let content: Div = match view {
            View::Capture => div()
                .size_full()
                .flex()
                .flex_col()
                .child(self.capture.clone()),
            View::Collection => div()
                .size_full()
                .flex()
                .flex_col()
                .child(self.collection.clone()),
            View::Todo => div()
                .size_full()
                .flex()
                .flex_col()
                .child(self.board.clone()),
        };

        div()
            .id("panel-root")
            .key_context("Panel")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(Self::on_key_down))
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(t::SURFACE))
            .overflow_hidden()
            // 键盘模态独立（REQ-PANEL-SCROLL-STABILITY · DEC-107）：gpui 默认键盘
            // 输入抑制 hover（vendor HitboxId::is_hovered 于 last_input_was_keyboard
            // 恒 false）——面板内任意键入会伪翻转 on_hover(false) 上抛 MouseExitHide。
            // 显隐判定只认「鼠标真离开」：模态过滤关闭，键盘滚动/键入不触显隐。
            .hover_listener_mode(HoverListenerMode::InputModalityIndependent)
            .on_hover(cx.listener(
                |this, hovered: &bool, _window: &mut Window, cx: &mut Context<Self>| {
                    // 移出面板（非 Pin）→ 上抛宿主收面板全隐（REQ-PANEL-HIDE；
                    // Pin 屏蔽移出隐藏 · SCN-PANEL-PIN-BLOCKS-HIDE）。
                    if !*hovered && !this.pinned {
                        this.emit(ShellEvent::MouseExitHide, cx);
                    }
                },
            ))
            .child(bar)
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .bg(rgb(t::NEUTRAL))
                    .relative()
                    .child(content),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[gpui::test]
    async fn keyboard_modality_does_not_emit_mouse_exit_hide(cx: &mut gpui::TestAppContext) {
        // REQ-PANEL-SCROLL-STABILITY（DEC-107）：面板内键入/方向键滚动不得被 gpui
        // 键盘模态 hover 抑制语义伪翻转成「移出」→ MouseExitHide 收面板。根
        // hover_listener_mode=InputModalityIndependent：hover 只认真移出与几何变化；
        // 键盘事件后悬停保持不上抛；真移出（MouseExited）照常上抛（控制组防过修）。
        use gpui::{InputEvent as _, VisualContext as _};
        let events: std::rc::Rc<std::cell::RefCell<Vec<&'static str>>> = Default::default();
        let (view, visual_cx) = cx.add_window_view(|_window, cx| {
            let mut panel = PanelView::new(cx);
            let events = events.clone();
            panel.set_event_sink(Box::new(move |ev: &ShellEvent, _cx: &mut App| {
                if let ShellEvent::MouseExitHide = ev {
                    events.borrow_mut().push("exit-hide");
                }
            }));
            panel
        });
        // 悬停会话建立：面板根仅 false 分支上抛 → 进入不发事件。
        visual_cx.simulate_mouse_move(
            gpui::point(gpui::px(300.0), gpui::px(300.0)),
            None,
            Default::default(),
        );
        visual_cx.run_until_parked();
        assert!(events.borrow().is_empty());
        // 键盘滚动语义键入：模态翻 Keyboard + refresh 重绘——MouseExitHide 不得伪发。
        visual_cx.simulate_keystrokes("down");
        visual_cx.run_until_parked();
        visual_cx.simulate_keystrokes("a");
        visual_cx.run_until_parked();
        assert!(events.borrow().is_empty());
        // 控制组：真移出（MouseExited 平台事件）仍上抛 MouseExitHide。
        let handle = visual_cx.window_handle();
        handle
            .update(&mut visual_cx.cx, |_, window, cx| {
                window.dispatch_event(
                    gpui::MouseExitEvent {
                        position: gpui::point(gpui::px(0.0), gpui::px(0.0)),
                        ..Default::default()
                    }
                    .to_platform_input(),
                    cx,
                );
            })
            .unwrap();
        visual_cx.run_until_parked();
        assert_eq!(*events.borrow(), vec!["exit-hide"]);
        assert!(!visual_cx.read(|cx| view.read(cx).pinned));
    }

    #[test]
    fn pin_palette_maps_selected_to_ink_background() {
        // DEC-124：Pin 选中态配色落在**背景轴**——
        // t::PRIMARY 背景底；前景必须切 ON_PRIMARY（MUTED 于主墨底
        // ~1.15:1 失读）；悬停/按下维持主墨底防反白字打回浅底。未选中＝ibtn 基线
        // 三元组（纸面底＋muted＋warm）。防错轴回归（DEC-117/118②）。
        assert_eq!(pin_palette(true), (t::ON_PRIMARY, t::PRIMARY, t::PRIMARY));
        assert_eq!(pin_palette(false), (t::MUTED, t::SURFACE, t::WARM));
    }
}
