#![allow(non_snake_case)]
#![allow(unused_assignments)]
#![windows_subsystem = "windows"]

slint::include_modules!();

#[derive(Default)]
struct ImgPath {
    text: slint::SharedString,
}

use core::cell::RefCell;
use opencv::{core::*, imgcodecs, imgproc};
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use std::fs::{self, File};
use std::io::Read;
use toml::Value;
use tracing::{debug, error, info, warn};
use imgShow3::*;

use chrono::{Local, Timelike, Duration};

fn main() {
    //日志初始化
    let file_appender = tracing_appender::rolling::daily("./log", "tracing.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    logging(non_blocking);
    // 根据输入的条码信息查找所有文件路径，并返回前端展示
    let app = App::new().unwrap();
    let weak = app.as_weak(); // as_weak避免内存泄露
    let img_path = Rc::new(RefCell::new(ImgPath::default()));
    app.global::<im>().on_confirm(move || {
        info!("Text accepted!");
        let app = weak.unwrap();
        app.set_info("".into());
        let mut img_n = img_path.borrow_mut();
        let n = app.get_name();
        let pl = n.as_str().len();
        info!("The QR code is {}", &n);
        if pl == 24 {
            app.set_value(n.clone());
            img_n.text = n.clone();
            // 获取NG图片地址
            match find_images(&img_n.text) {
                Ok(Some(v)) => {
                    if v.len() == 0 {
                        info!("There is no NG files.");
                        app.set_info("[INFO] => This SN is all OK.".into());
                    }

                    if v.len() >= 1 {
                        info!("There is at least 1 NG files.");
                        let src = v[0].to_str().unwrap(); 
                        debug!("The src path is {}", src);
                        if let Ok(im1) = img_source(src){
                            app.set_img1(im1);
                            debug!("The img1 has been set.");
                        };
                        
                        
                    }
                    if v.len() >= 2 {
                        info!("There is at least 2 NG files.");
                        let src = v[1].to_str().unwrap(); 
                        debug!("The src path is {}", src);
                        if let Ok(im2) = img_source(src){
                            app.set_img2(im2);
                            debug!("The img2 has been set.");
                        };
                        
                    }
                    if v.len() >= 3 {
                        info!("There is at least 3 NG files.");
                        let src = v[2].to_str().unwrap(); 
                        debug!("The src path is {}", src);
                        if let Ok(im3) = img_source(src){
                            app.set_img3(im3);
                            debug!("The img3 has been set.");
                        };
                        
                    }
                    if v.len() >= 4 {
                        info!("There is at least 4 NG files.");
                        let src = v[3].to_str().unwrap(); 
                        debug!("The src path is {}", src);
                        if let Ok(im4) = img_source(src){
                            app.set_img4(im4);
                            debug!("The img4 has been set.");
                        };
                        
                    }
                },
                Ok(None)=> {
                    app.set_info("[Warning!!!] => Can't find the path according to config file.".into());
                },
                _ => {
                    // unreachable
                    app.set_info("[Warning!!!] => The config file parse Error. Abort!".into());
                }
            }
        } else {
            warn!("[Warning!!!] => The QR code length is not right.");
            app.set_info("[Warning!!!] => The QR code length is not right.".into());
        };
        info!("app confirm finished.");
    });
    app.run().unwrap();
}

fn find_images(s: &str) -> Result<Option<Vec<PathBuf>>> {
    // 根据提供的条码查找NG文件地址
    let mut p: Vec<PathBuf> = vec![];
    match File::open("config.toml") {
        Ok(mut file) => {
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            info!("The config content is : {}", contents);
            let value = contents.parse::<Value>()?;

            let path = value["img"]["path"].as_str().unwrap();
            info!("The path is : {}", path);

            // 获得当前时间确定文件夹名称

            let now = Local::now();
            let now_date = now.format("%Y%m%d");
            let now_time = now.hour();
            let before_date = (now - Duration::days(1)).format("%Y%m%d");
            let mut time_str = String::new();

            if now_time < 8 {
                time_str = format!("{}{}",before_date.to_string(), "夜班");
            } else if now_time >= 8 && now_time < 20 {
                time_str = format!("{}{}",now_date.to_string(), "白班");
            } else {
                time_str = format!("{}{}",now_date.to_string(), "夜班");
            }

            
            info!("The current dir is : {}\n", time_str);
            let ng_path = Path::new(path).join(time_str).join(s).join("ResultImage");
            info!("The ng path is {}", ng_path.display());

            match fs::read_dir(ng_path) {
                Ok(paths) => {
                    for path in paths {
                        let file_path = path?.path();
                        let file_name = file_path.display().to_string();
                        if file_name.contains("_NG_") {
                            p.push(file_path);
                            }
                        }
                    },
                Err(_r) => {
                    error!("Read ng path failed.");
                    return Err(anyhow!("Read ng path failed."));
                },
                }
            Ok(Some(p))
            },
        Err(_e) => {
            error!("Read config failed.");
            Err(anyhow!("Read config failed."))
        }
    }
}

fn img_source(src: &str) -> Result<slint::Image> { 
    let mut frame_rgba = Mat::default();
    info!("Start Read images.");
    if let Ok(frame_bgr) = imgcodecs::imread(src, imgcodecs::IMREAD_COLOR){
        debug!("Read image success.");
        let width = frame_bgr.cols() as u32;
        let height = frame_bgr.rows() as u32;
        imgproc::cvt_color(&frame_bgr, &mut frame_rgba, imgproc::COLOR_BGR2RGBA, 0)?;
        let mut frame_data = vec![0; (width * height * 4) as usize];
        let frame_rgba_data = frame_rgba.data_bytes()?.to_vec();
        frame_data.copy_from_slice(&frame_rgba_data);
        let v= slint::Image::from_rgba8(slint::SharedPixelBuffer::clone_from_slice(
            frame_data.as_slice(),
            width,
            height,
        ));
        Ok(v)
    } else {
        debug!("Read image failed.");
        Err(anyhow!("Read image failed."))
    }
    
}
