mod util;
use std::time::{Instant, Duration};
use image::EncodableLayout;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::{io, path::PathBuf};
use std::sync::{Arc, Mutex};
use util::file_filter;
use util::image_processer;
use clap::{Parser, Arg, Command};
use futures::future::join_all;
use tokio::runtime::Builder;
use tokio::task::JoinHandle;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    path: String,
    #[arg(short, long)]
    output: Option<String>,
    #[arg(short, long, default_value_t = 85.0)]
    quality: f32,
    #[arg(short, long, default_value_t = 2)]
    threads: usize,
    #[arg(short, long)]
    lossless: bool
}

pub fn create_parent_dirs<P: AsRef<Path>>(file_path: P) -> io::Result<()> {
    let path = file_path.as_ref();
    // 获取文件所在的目录路径
    if let Some(parent_dir) = path.parent() {
        // 如果父目录不存在，则创建它（包括所有必要的父目录）
        if !parent_dir.exists() {
            fs::create_dir_all(parent_dir)?;
        }
    }
    Ok(())
}


// format!(
//     "{}/{}.webp",
//     webp_folder_path.to_string(),
//     filename_original_image
// );


async fn start() {
    let args = Args::parse();
    let mut path_to_process = PathBuf::from(&args.path);
    let mut path_to_output = PathBuf::new();

    match args.output {
        // 如果 output 为 None，生成新的路径并赋值
        None => {
            let mut arg_path_to_output = args.path;
            arg_path_to_output.push_str("-export");
            path_to_output = PathBuf::from(&arg_path_to_output);
        }
        // 如果 output 已提供，直接使用提供的路径
        Some(ref output_path) => {
            path_to_output = PathBuf::from(output_path);
        }
    }

    let img_vec = file_filter::get_image_files(&path_to_process).expect("get_image_files error");

    println!("输入路径：{:?}", &path_to_process);
    println!("图片数量：{}", img_vec.len());
    println!("输出路径：{:?}", &path_to_output);
    println!("无损：{:?}", args.lossless);
    if args.lossless==false {
        println!("质量：{}", args.quality);
    }
    println!("线程数：{}", args.threads);
    print!("Press Enter to continue...");
    io::stdout().flush().unwrap();  // 确保提示立即输出

    // 等待用户按回车
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    // 用户按下回车后继续执行程序
    println!("Continuing with program execution...");

    let start = Instant::now();

    // 单线程操作
    // let mut processed_count = 0;
    // let mut skipped_count = 0;
    // for ifo in &img_vec {
    //     println!("{:?}", ifo);
    //     let mut export_full_path = path_to_output.join(&ifo.relative_path);
    //     // println!("{:?}",export_full_path)
    //     println!("{} started", ifo.filename);
    //     create_parent_dirs(&export_full_path).expect("create_parent_dirs error");
    //     let ifo_full_path = PathBuf::from(&ifo.full_path);
    //     if export_full_path.exists() {
    //         println!("{:?} already exists, skipping.", export_full_path);
    //         skipped_count = skipped_count + 1;
    //         continue;  // 跳过当前文件
    //     } else {
    //         let result = image_processer::image_to_webp(&ifo_full_path, &export_full_path, args.quality);
    //         println!("{:?} completed", result);
    //         processed_count = processed_count + 1;
    //     }
    // }
    // 单线程操作

    let runtime = Builder::new_multi_thread()
        .worker_threads(args.threads) // 控制工作线程数量
        .enable_all() // 启用 I/O 和定时器等功能
        .build()
        .unwrap();


    let skipped_count = Arc::new(Mutex::new(0));
    let processed_count = Arc::new(Mutex::new(0));
    let tasks: Vec<JoinHandle<()>> = img_vec.into_iter().map(|ifo| {
        let skipped_count = Arc::clone(&skipped_count);
        let processed_count = Arc::clone(&processed_count);
        let path_to_output = path_to_output.clone();
        let ifo_full_path = PathBuf::from(&ifo.full_path);
        let export_full_path = path_to_output.join(&ifo.relative_path);

        // 在 Runtime 中运行异步任务
        runtime.spawn(async move {
            // 创建输出路径的父目录
            if let Err(e) = create_parent_dirs(&export_full_path) {
                eprintln!("Error creating directories for {:?}: {}", export_full_path, e);
                return;
            }

            // 如果文件已经存在，跳过处理
            if export_full_path.exists() {
                println!("{:?} already exists, skipping.", export_full_path);
                let mut skipped = skipped_count.lock().unwrap();
                *skipped += 1;
                return;
            }

            // 执行图像转换
            let result = image_processer::image_to_webp(&ifo_full_path, &export_full_path, args.quality, args.lossless);
            let mut processed = processed_count.lock().unwrap();
            *processed += 1;
            match result {
                Some(_) => {
                    println!("{:?} completed", export_full_path)
                }
                None => {
                    eprintln!("{:?} failed", export_full_path);
                }
            }
        })
    }).collect();

    // 等待所有任务完成
    let futures: Vec<_> = tasks.into_iter().map( |task| task).collect();
    join_all(futures).await;
    runtime.shutdown_background();
    let skipped = skipped_count.lock().unwrap();
    let processed = processed_count.lock().unwrap();
    let duration = start.elapsed();
    println!("Function execution took: {:?}", duration);
    println!("Completed! skipped: {}, processed: {}", *skipped, *processed);
}

#[tokio::main]
async fn main() {
    start().await;
}
