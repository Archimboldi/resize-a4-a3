use druid::widget::{Button, Flex, TextBox, Label};
use druid::{
    commands, AppDelegate, AppLauncher, Command, DelegateCtx, Env, FileDialogOptions, Lens,
    LocalizedString, Target, Widget, WidgetExt, WindowDesc, Data
};
use image::{GenericImage, GenericImageView, ImageBuffer};
use image::jpeg::*;
use std::io::BufWriter;
use std::fs::{self, File};
use std::thread;
use std::sync::{ Arc, RwLock };
const SIZE_W: u32 = 2100;
const SIZE_H: u32 = 2970;
const SIZE_X: u32 = 4200;
use image::Pixel;
struct Delegate;

#[derive(Data, Clone, Lens)]
struct State {
    #[lens(name = "src_lens")]
    src: String,
    #[lens(name = "dist_lens")]
    dist: String,
    count: u64,
    sord: bool
}
#[derive(Clone)]
struct SAndD {
    sr: String,
    dis: String
}

fn resize_a34(sr: &str, dis: &str) ->Result<(), image::error::ImageError> {
    let mut img = image::open(sr)?;
    let (w, h) = img.dimensions();
    let mut target = ImageBuffer::from_pixel(SIZE_W, SIZE_H, Pixel::from_channels(255, 255, 255, 255));
    if w < h {
        img = img.thumbnail(SIZE_W, SIZE_H);
        let (nw,nh) = img.dimensions();
        let x = (SIZE_W - nw)/2;
        let y = (SIZE_H - nh)/2;
        target.copy_from(&img, x, y).unwrap();
    }else {
        img = img.thumbnail(SIZE_X, SIZE_H);
        let (nw, nh) = img.dimensions();
        target = ImageBuffer::from_pixel(SIZE_X, SIZE_H, Pixel::from_channels(255, 255, 255, 255));
        let x = (SIZE_X - nw)/2;
        let y = (SIZE_H - nh)/2;
        target.copy_from(&img, x, y).unwrap();
    }
 
    let mut w = BufWriter::new(File::create(dis)?);
    let mut jer = JPEGEncoder::new_with_quality(&mut w, 40);
    jer.set_pixel_density(PixelDensity::dpi(300));
    jer.encode_image(&target)?;
    Ok(())
}
fn dir_fs(src: &str, dir: &str, dist: &str, count: &mut u64, done: &mut Vec<SAndD>) -> Result<Vec<String>, std::io::Error> {
    let mut res = Vec::new();
    let dirs = fs::read_dir(dir)?;
    for entry in dirs {
        if let Ok(path) = entry{
            if path.path().is_dir(){
                let ult = dir_fs(src, path.path().to_str().unwrap(), dist, count, done)?;
                for p in ult{
                    res.push(p);
                }
            }else {
                if path.path().to_str().unwrap().ends_with("jpg"){
                    if let Ok(p) = path.path().strip_prefix(src) {
                        let t = format!("{}\\{}", dist, p.to_str().unwrap());
                        if !std::path::Path::new(t.as_str()).exists() {
                            let mut l = "/";
                            if let Some(idx) = t.rfind('\\') {
                                l = &t[..idx];
                            };
                            let e = l.split("\\");
                            let mut ft = String::new();
                            for i in e {
                                ft.push_str(format!("{}\\", i).as_str());
                                if !std::path::Path::new(ft.as_str()).exists() {
                                    fs::create_dir(ft.as_str()).unwrap();
                                }
                            }
                            done.push(SAndD{ sr: path.path().to_str().unwrap().to_string(), dis: t.clone()});
                            *count +=1;
                        }
                        res.push(p.to_str().unwrap().to_string());
                    }
                }else if path.path().to_str().unwrap().ends_with("xlsx"){
                    if let Ok(p) = path.path().strip_prefix(src) {
                        let t = format!("{}\\{}", dist, p.to_str().unwrap());
                        fs::copy(path.path(), std::path::Path::new(&t)).unwrap();
                    }
                }
            }
        }
    }
    Ok(res)
}

fn main() {
    let main_window = WindowDesc::new(ui_builder)
        .window_size((400., 270.))
        .resizable(false)
        .title(LocalizedString::new("resize-a4/a3").with_placeholder("Resize-A4/A3"));
    let data = State {
        src: "".to_owned(),
        dist: "".to_owned(),
        count: 0_u64,
        sord: true
    };
    AppLauncher::with_window(main_window)
        .delegate(Delegate)
        .use_simple_logger()
        .launch(data)
        .expect("launch failed");
}

fn ui_builder() -> impl Widget<State> {
    let open_dialog_options = FileDialogOptions::new()
        .select_directories();
    let open2_dialog_options = open_dialog_options.clone();
    let input = TextBox::new().with_placeholder("请选择源文件存放路径...").lens(State::src_lens).fix_width(265.);
    let output = TextBox::new().with_placeholder("请选择调整后保存路径...").lens(State::dist_lens).fix_width(265.);
    let done = Label::new(|d: &State, _: &Env| format!("标准化图片数: {}", d.count)).fix_width(190.);
    let open = Button::new("浏览源.").on_click(move |ctx, data: &mut State, _| {
        data.sord = true;
        ctx.submit_command(
            Command::new(
                druid::commands::SHOW_OPEN_PANEL,
                open_dialog_options.clone(),
            ),
            None,
        )
    });
    let save = Button::new("保存至.").on_click(move |ctx, data: &mut State, _| {
        data.sord = false;
        ctx.submit_command(
            Command::new(
                druid::commands::SHOW_OPEN_PANEL,
                open2_dialog_options.clone(),
            ),
            None,
        )
    });
    let doit = Button::new("开始").on_click(move |_ , data: &mut State, _| {
        let mut don: Vec<SAndD> = Vec::new();
        data.count = 0;
        dir_fs(&data.src, &data.src, &data.dist, &mut data.count, &mut don).unwrap();
        let count_ = Arc::new(RwLock::new(1_usize));
        let mut ths = Vec::new();
        let num = num_cpus::get();
        for _ in 0..num+1 {
            let don_ = don.clone();
            let count = data.count + 1;
            let ount = count_.clone();
            let h = thread::spawn(move ||{
                loop {
                    let mut c = 0;
                    {
                        if let Ok(r) = ount.try_read(){
                            c = *r;
                        }
                    };
                    if c != 0 {
                        if c <= count as usize {
                            let e = resize_a34(&don_[c-1].sr, &don_[c-1].dis);
                            if let Ok(()) = e {
                                let mut w = ount.write().unwrap();
                                *w +=1;
                            }
                        }else{
                            break;
                        }
                    }
                }
            });
            ths.push(h);
        }
        for h in ths{
            let _ = h.join();
        }
    });
    let row1 = Flex::row()
        .with_child(input)
        .with_spacer(7.0)
        .with_child(open);
    let row2 = Flex::row()
        .with_child(output)
        .with_spacer(7.0)
        .with_child(save);
    let row3 = Flex::row()
        .with_child(done)
        .with_spacer(7.0)
        .with_child(doit);
    Flex::column()
        .with_spacer(52.0)
        .with_flex_child(
            row1, 
            1.0,
        )
        .with_spacer(24.0)
        .with_flex_child(
            row2, 
            1.0,
        )
        .with_spacer(34.0)
        .with_flex_child(
            row3,
            1.0,
        )
        .center()
}

impl AppDelegate<State> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut State,
        _env: &Env,
    ) -> bool {
        if let Some(path_info) = cmd.get(commands::OPEN_FILE) {
            match path_info.path().is_dir() {
                true => {
                    let dir = path_info.path().to_str().unwrap();
                    
                    if data.sord {
                        data.src = dir.to_string();
                    }else {
                        data.dist = dir.to_string();
                    }
                    
                },
                false => {
                    println!("选择的路径不是一个目录！");
                }
            }
            return true;
        }
        false
    }
}