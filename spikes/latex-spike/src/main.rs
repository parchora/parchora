// TASK-004 latex-pipeline spike（design §11.1 / DEC-42）
// 目标：①单条编译延迟分布（设计目标：首帧 ≤16ms、缓存命中 ≤1ms——目标值待入口确认）
//      ②release 二进制体积增量（内嵌全量字体档；字体子集化属发布期工作）
//      ③World 内存文件系统原型（无磁盘访问、无 package 下载）
//      ④字体许可义务清单（typst-assets NOTICE，见 spike 报告）
use std::path::PathBuf;
use std::time::Instant;

use ecow::eco_format;
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime, Str};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};
use typst_assets::fonts as embedded_fonts;
use typst_layout::PagedDocument;

/// typst::World 的内存实现：主源码驻内存，字体来自 typst-assets 内嵌（无磁盘、无 package）。
struct MemWorld {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
    main: Source,
}

impl MemWorld {
    fn new(main: Source, fonts: Vec<Font>, book: FontBook, library: Library) -> Self {
        Self {
            library: LazyHash::new(library),
            book: LazyHash::new(book),
            fonts,
            main,
        }
    }
}

impl World for MemWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }
    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }
    fn main(&self) -> FileId {
        self.main.id()
    }
    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main.id() {
            Ok(self.main.clone())
        } else {
            Err(FileError::NotFound(PathBuf::from(format!("{id:?}"))))
        }
    }
    fn file(&self, id: FileId) -> FileResult<Bytes> {
        if id == self.main.id() {
            let text: Str = self.main.text().into();
            Ok(Bytes::from_string(text))
        } else {
            Err(FileError::NotFound(PathBuf::from(format!("{id:?}"))))
        }
    }
    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }
    fn today(&self, _offset: Option<typst::foundations::Duration>) -> Option<Datetime> {
        Datetime::from_ymd_hms(2026, 10, 17, 0, 0, 0)
    }
}

/// 公式语料：分数/根号/矩阵/积分/大算符 各 20 条（TASK-004 ①）。
fn corpus() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        (
            "fraction",
            vec![
                r"\frac{a}{b}",
                r"\frac{1}{2}",
                r"\frac{x+y}{x-y}",
                r"\frac{a+b}{c+d}",
                r"\frac{1}{x^2+y^2}",
                r"\frac{\alpha}{\beta}",
                r"\frac{d}{dx}",
                r"\frac{dy}{dx}",
                r"\frac{n!}{k!(n-k)!}",
                r"\frac{\pi}{4}",
                r"\frac{1}{4}",
                r"\frac{3}{7}",
                r"\frac{a_1}{b_1}",
                r"\frac{\partial f}{\partial x}",
                r"\frac{\sqrt{2}}{2}",
                r"\frac{5}{8}",
                r"\frac{p}{q}",
                r"\frac{m+n}{m-n}",
                r"\frac{-b \pm \sqrt{b^2-4ac}}{2a}",
                r"\frac{\sin\theta}{\cos\theta}",
            ],
        ),
        (
            "sqrt",
            vec![
                r"\sqrt{2}",
                r"\sqrt{x}",
                r"\sqrt{a^2+b^2}",
                r"\sqrt[3]{8}",
                r"\sqrt[3]{x+y}",
                r"\sqrt{x^2+1}",
                r"\sqrt{\frac{1}{2}}",
                r"\sqrt{2\pi}",
                r"\sqrt{n}",
                r"\sqrt[4]{16}",
                r"\sqrt{b^2-4ac}",
                r"\sqrt{x_1+x_2}",
                r"\sqrt{\alpha+\beta}",
                r"\sqrt[3]{a^3}",
                r"\sqrt{e^x}",
                r"\sqrt{\sin^2\theta+\cos^2\theta}",
                r"\sqrt{2}x",
                r"\sqrt[5]{32}",
                r"\sqrt{|x|}",
                r"\sqrt{1-v^2/c^2}",
            ],
        ),
        (
            "matrix",
            vec![
                r"\begin{pmatrix}1&2\\3&4\end{pmatrix}",
                r"\begin{pmatrix}a&b\\c&d\end{pmatrix}",
                r"\begin{bmatrix}1&0\\0&1\end{bmatrix}",
                r"\begin{bmatrix}x&y\end{bmatrix}",
                r"\begin{pmatrix}1\\2\\3\end{pmatrix}",
                r"\begin{bmatrix}1&2&3\\4&5&6\end{bmatrix}",
                r"\begin{pmatrix}0&1\\1&0\end{pmatrix}",
                r"\begin{bmatrix}a_{11}&a_{12}\\a_{21}&a_{22}\end{bmatrix}",
                r"\begin{vmatrix}a&b\\c&d\end{vmatrix}",
                r"\begin{matrix}1&0\\0&1\end{matrix}",
                r"\begin{pmatrix}\lambda_1&0\\0&\lambda_2\end{pmatrix}",
                r"\begin{bmatrix}1&1&1\\0&1&1\\0&0&1\end{bmatrix}",
                r"\begin{pmatrix}x\\y\\z\end{pmatrix}",
                r"\begin{bmatrix}\cos\theta&-\sin\theta\\ \sin\theta&\cos\theta\end{bmatrix}",
                r"\begin{pmatrix}1&2&3\end{pmatrix}",
                r"\begin{bmatrix}0&-i\\ i&0\end{bmatrix}",
                r"\begin{pmatrix}p&q\\r&s\end{pmatrix}",
                r"\begin{bmatrix}1&0&0\\0&1&0\\0&0&1\end{bmatrix}",
                r"\begin{vmatrix}1&2\\3&4\end{vmatrix}",
                r"\begin{matrix}a&b\\c&d\end{matrix}",
            ],
        ),
        (
            "integral",
            vec![
                r"\int_0^1 x\,dx",
                r"\int_a^b f(x)\,dx",
                r"\int_{-\infty}^{\infty} e^{-x^2}\,dx",
                r"\int x^2\,dx",
                r"\int_1^n \frac{1}{x}\,dx",
                r"\int_0^{\pi}\sin\theta\,d\theta",
                r"\iint_D f(x,y)\,dA",
                r"\iiint_V \rho\,dV",
                r"\oint_{\partial S} F\cdot dr",
                r"\int\!\!\int f\,dx\,dy",
                r"\int_0^{2\pi} d\theta",
                r"\int_{\Omega} u\,dv",
                r"\int_0^t v(\tau)\,d\tau",
                r"\int_{S^1}\omega",
                r"\int_0^1 \frac{dx}{1+x^2}",
                r"\int_1^e \ln x\,dx",
                r"\int\cos x\,dx",
                r"\int_0^\infty \frac{e^{-x}}{x}\,dx",
                r"\int_C \nabla f\cdot d\vec r",
                r"\int_0^1 \sqrt{x}\,dx",
            ],
        ),
        (
            "big-operator",
            vec![
                r"\sum_{i=1}^n i",
                r"\sum_{i=0}^{n} a_i",
                r"\sum_{k=1}^\infty \frac{1}{k^2}",
                r"\prod_{i=1}^n i",
                r"\prod_{k=1}^\infty \frac{4}{\pi^2}",
                r"\bigcup_{i=1}^n A_i",
                r"\bigcap_{i=1}^n A_i",
                r"\lim_{n\to\infty} a_n",
                r"\lim_{x\to 0} \frac{\sin x}{x}",
                r"\max_{x\in S} f(x)",
                r"\min_{x\in S} f(x)",
                r"\sup_{x>0} f(x)",
                r"\inf_{x>0} f(x)",
                r"\sum_{j=1}^{m}\sum_{k=1}^{n} a_{jk}",
                r"\sum_{\sigma\in S_n} \operatorname{sgn}(\sigma)",
                r"\bigoplus_{i=1}^n V_i",
                r"\bigotimes_{i=1}^n V_i",
                r"\coprod_{i=1}^n X_i",
                r"\sum_{\lambda} p(\lambda)",
                r"\lim_{t\to\infty} x(t)",
            ],
        ),
    ]
}

fn fmt_ms(ns: u128) -> String {
    format!("{:.2}ms", ns as f64 / 1_000_000.0)
}

fn stats(mut latencies: Vec<u128>) -> (u128, u128, u128) {
    latencies.sort_unstable();
    let n = latencies.len();
    let median = latencies.get(n / 2).copied().unwrap_or(0);
    let p95 = latencies
        .get((n as f64 * 0.95) as usize)
        .copied()
        .unwrap_or(0);
    let max = *latencies.last().unwrap_or(&0);
    (median, p95, max)
}

/// 一次性加载内嵌字体（全量档——字体子集化属发布期工作，见报告）。
fn load_fonts() -> (Vec<Font>, FontBook) {
    let mut fonts = Vec::new();
    let mut book = FontBook::new();
    for data in embedded_fonts() {
        for font in Font::iter(Bytes::new(data)) {
            book.push(font.info().clone());
            fonts.push(font);
        }
    }
    (fonts, book)
}

/// mitex 0.2.4 转换输出的运行时前置（生产实现 TASK-109 应内嵌 mitex .typ 包源码替代此最小集）。
/// 依 probe 证据闭合：mitexsqrt（root 参数序 = index→radicand，root.rs L26-33）、
/// zws / negthinspace（typst 0.15 math scope 未定义，全 src/ grep 无命中）、
/// operatorname、矩阵别名（mat 只以 "mat" 注册，无 pmatrix 等，mitex 环境名直出）。
const PRELUDE: &str = concat!(
    "#let mitexsqrt(..a) = {\n",
    "  let p = a.pos()\n",
    "  if p.len() == 2 { math.root(p.first(), p.last()) } else { math.sqrt(p.first()) }\n",
    "}\n",
    "#let diff = $dif$\n",
    "#let mitexdisplay(a) = a\n",
    "#let operatorname(a) = math.upright(a)\n",
    "#let zws = h(0pt)\n",
    "#let negthinspace = h(-0.1667em)\n",
    "#let pmatrix(..a) = math.mat(..a, delim: \"(\")\n",
    "#let bmatrix(..a) = math.mat(..a, delim: \"[\")\n",
    "#let vmatrix(..a) = math.mat(..a, delim: \"|\")\n",
    "#let matrix(..a) = math.mat(..a)\n",
);

fn wrap_doc(converted: &str, with_page: bool) -> String {
    if with_page {
        eco_format!(
            "#set page(width: 9cm, height: auto, margin: 4pt)\n#set text(11pt, lang: \"en\")\n{}$ {} $\n",
            PRELUDE,
            converted
        )
        .to_string()
    } else {
        eco_format!(
            "#set page(width: 9cm, height: auto, margin: 4pt)\n{}$ {} $\n",
            PRELUDE,
            converted
        )
        .to_string()
    }
}

fn main_id() -> FileId {
    let vpath = VirtualPath::new("main.typ").expect("static virtual path");
    FileId::new(RootedPath::new(VirtualRoot::Project, vpath))
}

/// mitex 0.2.4 spec 面向旧版 Codex 符号集（typst ≤0.12），typst 0.15（codex 0.3.0）已改名：
/// sect→inter · plus.circle→plus.o · times.circle→times.o（sym.txt L247/L275/L503-506）。
/// 带点符号名在 math 解析层直接查符号表，不经用户作用域，不可用 #let 别名——
/// 故带名缺口只能在上层做字符串桥；生产 TASK-109 应以自定义 spec 映射替代本桥（见报告）。
fn compat_shim(converted: &str) -> String {
    converted
        .replace("sect.big", "inter.big")
        .replace("plus.circle.big", "plus.o.big")
        .replace("times.circle.big", "times.o.big")
}

fn build_world(
    latex: &str,
    fonts: Vec<Font>,
    book: FontBook,
    library: &Library,
) -> Result<MemWorld, String> {
    let converted = mitex::convert_math(latex, None).map_err(|e| format!("mitex: {e}"))?;
    let text = wrap_doc(&compat_shim(&converted), true);
    let source = Source::new(main_id(), text);
    Ok(MemWorld::new(source, fonts, book, library.clone()))
}

fn compile_doc(world: &MemWorld) -> Result<(std::time::Duration, PagedDocument), String> {
    let t0 = Instant::now();
    let result = typst::compile::<PagedDocument>(world);
    let elapsed = t0.elapsed();
    result.output.map(|doc| (elapsed, doc)).map_err(|errors| {
        errors
            .iter()
            .map(|e| e.message.to_string())
            .collect::<Vec<_>>()
            .join("; ")
    })
}

fn compile_dts(world: &MemWorld) -> Result<std::time::Duration, String> {
    compile_doc(world).map(|(d, _)| d)
}

/// probe 模式：打印代表性公式的 mitex 转换产物，据此闭合 PRELUDE 缺口（铁律一：不猜）。
fn probe() {
    let samples: Vec<&str> = vec![
        r"\frac{a}{b}",
        r"\sqrt{2}",
        r"\sqrt[3]{8}",
        r"\sqrt[5]{32}",
        r"\begin{pmatrix}1&2\\3&4\end{pmatrix}",
        r"\begin{bmatrix}x&y\end{bmatrix}",
        r"\begin{vmatrix}a&b\\c&d\end{vmatrix}",
        r"\begin{matrix}1&0\\0&1\end{matrix}",
        r"\int_0^1 x\,dx",
        r"\iint_D f(x,y)\,dA",
        r"\iiint_V \rho\,dV",
        r"\oint_{\partial S} F\cdot dr",
        r"\int\!\!\int f\,dx\,dy",
        r"\sum_{i=1}^n i",
        r"\prod_{i=1}^n i",
        r"\bigcup_{i=1}^n A_i",
        r"\bigcap_{i=1}^n A_i",
        r"\lim_{n\to\infty} a_n",
        r"\max_{x\in S} f(x)",
        r"\min_{x\in S} f(x)",
        r"\sup_{x>0} f(x)",
        r"\inf_{x>0} f(x)",
        r"\sum_{\sigma\in S_n} \operatorname{sgn}(\sigma)",
        r"\bigoplus_{i=1}^n V_i",
        r"\bigotimes_{i=1}^n V_i",
        r"\coprod_{i=1}^n X_i",
        r"\int_C \nabla f\cdot d\vec r",
        r"\frac{dy}{dx}",
    ];
    for s in samples {
        match mitex::convert_math(s, None) {
            Ok(c) => println!("IN : {}\nOUT: {}\n", s, c),
            Err(e) => println!("IN : {}\nERR: {}\n", s, e),
        }
    }
}

fn main() {
    if std::env::args().nth(1).as_deref() == Some("probe") {
        probe();
        return;
    }
    let (fonts, book) = load_fonts();
    println!("[fonts] embedded faces loaded: {}", fonts.len());
    let library = Library::builder().build();

    let corpus = corpus();
    let total: usize = corpus.iter().map(|(_, v)| v.len()).sum();
    println!(
        "[corpus] {} categories x 20 = {} formulas",
        corpus.len(),
        total
    );

    let mut all_cold: Vec<u128> = Vec::new();
    let mut all_memo: Vec<u128> = Vec::new();
    let mut failures = 0usize;

    for (category, formulas) in &corpus {
        let mut colds: Vec<u128> = Vec::new();
        for latex in formulas {
            match build_world(latex, fonts.clone(), book.clone(), &library)
                .and_then(|w| compile_dts(&w))
            {
                Ok(d) => colds.push(d.as_nanos()),
                Err(e) => {
                    failures += 1;
                    eprintln!("[FAIL] {category} {latex} -> {e}");
                }
            }
        }
        let (median, p95, max) = stats(colds.clone());
        println!(
            "[cold] {category:<12} median={} p95={} max={} n={}",
            fmt_ms(median),
            fmt_ms(p95),
            fmt_ms(max),
            colds.len()
        );
        all_cold.extend(colds);

        // 缓存命中路径：同一 World 实例二次编译（comemo 记忆化），每类抽 4 条。
        let mut seconds: Vec<u128> = Vec::new();
        for latex in formulas.iter().take(4) {
            match build_world(latex, fonts.clone(), book.clone(), &library)
                .and_then(|w| {
                    compile_dts(&w)?;
                    Ok(w)
                })
                .and_then(|w| compile_dts(&w))
            {
                Ok(d) => seconds.push(d.as_nanos()),
                Err(e) => eprintln!("[MEMO-FAIL] {category} {latex} -> {e}"),
            }
        }
        if !seconds.is_empty() {
            all_memo.extend(seconds.clone());
            let (m2, _, mx2) = stats(seconds);
            println!(
                "[memo] {category:<12} median={} max={} (n<=4, warm re-compile)",
                fmt_ms(m2),
                fmt_ms(mx2)
            );
        }
    }

    let (med_c, p95_all, max_all) = stats(all_cold);
    let (med_m, _, max_m) = stats(all_memo);
    println!(
        "[OVERALL] cold median={} p95={} max={} | memo median={} max={} | failures={}",
        fmt_ms(med_c),
        fmt_ms(p95_all),
        fmt_ms(max_all),
        fmt_ms(med_m),
        fmt_ms(max_m),
        failures
    );

    // 端到端打样：公式 → mitex → typst → PagedDocument → SVG（渲染层可达性证明）。
    let sample = r"\frac{-b \pm \sqrt{b^2-4ac}}{2a}";
    match build_world(sample, fonts, book, &library).and_then(|w| {
        let (d, doc) = compile_doc(&w)?;
        let pages = doc.pages();
        if pages.is_empty() {
            return Err("no pages produced".to_string());
        }
        let opts = typst_svg::SvgOptions {
            render_bleed: false,
            pretty: false,
        };
        let svg = typst_svg::svg(&pages[0], &opts);
        std::fs::write("sample.svg", svg.as_bytes())
            .map_err(|e| format!("write sample.svg: {e}"))?;
        println!(
            "[e2e] compile {} | pages={} | svg_bytes={} -> sample.svg",
            fmt_ms(d.as_nanos()),
            pages.len(),
            svg.len()
        );
        Ok(())
    }) {
        Ok(()) => {}
        Err(e) => println!("[e2e] FAIL: {e}"),
    }
}
