use crate::models::{StitchSettings, WidthMode};
use anyhow::Result;
use image::{imageops::FilterType, Rgba, RgbaImage, DynamicImage};
use psd::Psd; // Import PSD crate
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use tauri::{AppHandle, Emitter};

// --- Helper: Split args respecting quotes ---
fn split_args(args: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_quote = false;
    for c in args.chars() {
        match c {
            '"' | '\'' => { in_quote = !in_quote; }
            ' ' | '\t' if !in_quote => {
                if !current.is_empty() { result.push(current.clone()); current.clear(); }
            }
            _ => { current.push(c); }
        }
    }
    if !current.is_empty() { result.push(current); }
    result
}

// --- Helper: Load Image (Handles PSD manually) ---
fn load_image(path: &Path) -> Result<DynamicImage> {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    
    if ext == "psd" {
        // PSD Handling: Read bytes -> Flatten -> Convert to RgbaImage -> DynamicImage
        let bytes = fs::read(path)?;
        let psd = Psd::from_bytes(&bytes).map_err(|e| anyhow::anyhow!("PSD Error: {}", e))?;
        let raw_pixels = psd.rgba();
        
        let buffer = RgbaImage::from_raw(psd.width(), psd.height(), raw_pixels)
            .ok_or_else(|| anyhow::anyhow!("Failed to create buffer from PSD"))?;
            
        Ok(DynamicImage::ImageRgba8(buffer))
    } else {
        // Standard handling (PNG, JPG, WEBP, AVIF, BMP)
        image::open(path).map_err(|e| anyhow::anyhow!("Image Load Error: {}", e))
    }
}

fn get_image_files(path: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(path)
        .unwrap_or_else(|_| panic!("Could not read dir"))
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
            // Added support for avif and psd here
            matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "webp" | "bmp" | "tiff" | "avif" | "psd")
        })
        .collect();

    files.sort_by(|a, b| {
        natord::compare(
            a.file_name().unwrap().to_str().unwrap(),
            b.file_name().unwrap().to_str().unwrap(),
        )
    });
    files
}

fn find_cut_line(img: &RgbaImage, start_y: u32, target_h: u32, settings: &StitchSettings) -> u32 {
    let max_h = img.height();
    let cut_y = start_y + target_h;

    if cut_y >= max_h { return max_h - start_y; }
    if settings.detector_type == 1 { return target_h; } 

    let threshold = (255.0 * (1.0 - (settings.sensitivity as f32 / 100.0))) as i16;
    let width = img.width();
    let margin = settings.ignorable_margin;
    let step = settings.scan_step;

    let search_limit_up = (target_h as f32 * 0.4) as u32;
    let mut current_offset = 0;
    let mut searching_up = true;
    let mut scan_y = cut_y;

    loop {
        if scan_y >= max_h { break; }
        let row_is_clean = {
            let mut clean = true;
            let mut prev_val = -1000;
            let start_x = if margin < width { margin } else { 0 };
            let end_x = if width > margin { width - margin } else { width };

            for x in start_x..end_x {
                let p = img.get_pixel(x, scan_y);
                let val = (p[0] as i16 + p[1] as i16 + p[2] as i16) / 3;
                if prev_val != -1000 && (val - prev_val).abs() > threshold {
                    clean = false; break;
                }
                prev_val = val;
            }
            clean
        };

        if row_is_clean { return scan_y - start_y; }

        if searching_up {
            if current_offset < (target_h - search_limit_up) {
                current_offset += step;
                scan_y = cut_y - current_offset;
            } else {
                searching_up = false;
                current_offset = 0;
                scan_y = cut_y;
            }
        } else {
            current_offset += step;
            scan_y = cut_y + current_offset;
            if current_offset > (target_h / 2) { return target_h; }
        }
    }
    target_h
}

fn process_single_folder(
    app: &AppHandle,
    input_path: &Path,
    output_root: &Path,
    settings: &StitchSettings,
    folder_idx: usize,
    total_folders: usize,
) -> Result<String> {
    let input_files = get_image_files(input_path);
    if input_files.is_empty() { return Ok("Skipped".to_string()); }

    let folder_name = input_path.file_name().unwrap_or_default().to_string_lossy();
    let out_folder = if settings.batch_mode {
        if settings.output_path.is_empty() {
             input_path.parent().unwrap_or(Path::new(".")).join(format!("{} [Stitched]", folder_name))
        } else {
             output_root.join(format!("{} [Stitched]", folder_name))
        }
    } else {
        if settings.output_path.is_empty() {
            input_path.parent().unwrap_or(Path::new(".")).join(format!("{} [Stitched]", folder_name))
        } else {
            output_root.to_path_buf()
        }
    };

    fs::create_dir_all(&out_folder)?;

    // Use load_image helper
    let first_img = load_image(&input_files[0])?;
    let width_mode = match settings.width_enforce_type {
        0 => WidthMode::NoEnforcement,
        1 => WidthMode::AutoUniform,
        2 => WidthMode::MatchMin,
        3 => WidthMode::Custom,
        4 => WidthMode::MatchMax,
        _ => WidthMode::AutoUniform,
    };

    let mut target_width = match width_mode {
        WidthMode::NoEnforcement | WidthMode::AutoUniform => first_img.width(),
        WidthMode::MatchMin => input_files.iter().filter_map(|p| load_image(p).ok()).map(|i| i.width()).min().unwrap_or(first_img.width()),
        WidthMode::Custom => settings.custom_width,
        WidthMode::MatchMax => input_files.iter().filter_map(|p| load_image(p).ok()).map(|i| i.width()).max().unwrap_or(first_img.width()),
    };

    let fill_pixel = if settings.fill_color == 1 { Rgba([255, 255, 255, 255]) } else { Rgba([0, 0, 0, 255]) };

    let mut buffer = RgbaImage::from_pixel(target_width, 0, fill_pixel);
    let mut file_count = 1;
    let total_files = input_files.len();

    for (idx, path) in input_files.iter().enumerate() {
        let prefix = if total_folders > 1 { format!("[Folder {}/{}] ", folder_idx + 1, total_folders) } else { "".into() };
        app.emit("status", format!("{}Processing {}/{}", prefix, idx + 1, total_files))?;
        app.emit("progress", (idx as f64 / total_files as f64) * 100.0)?;

        let mut img = load_image(path)?;
        let mut current_img_width = img.width();

        if width_mode == WidthMode::NoEnforcement {
             if buffer.height() > 0 && current_img_width != buffer.width() {
                let filename = format!("{:02}{}", file_count, settings.output_type);
                buffer.save(out_folder.join(filename))?;
                file_count += 1;
                target_width = current_img_width;
                buffer = RgbaImage::from_pixel(target_width, 0, fill_pixel);
             }
        } else if width_mode != WidthMode::MatchMax && current_img_width != target_width {
            let ratio = target_width as f64 / current_img_width as f64;
            let new_h = (img.height() as f64 * ratio) as u32;
            img = img.resize(target_width, new_h, FilterType::Lanczos3);
            current_img_width = target_width;
        }

        let img_rgba = img.to_rgba8();
        let x_offset = if width_mode == WidthMode::MatchMax { (target_width - current_img_width) / 2 } else { 0 };

        let new_height = buffer.height() + img_rgba.height();
        let mut new_buffer = RgbaImage::from_pixel(target_width, new_height, fill_pixel);
        image::imageops::overlay(&mut new_buffer, &buffer, 0, 0);
        image::imageops::overlay(&mut new_buffer, &img_rgba, x_offset as i64, buffer.height() as i64);
        buffer = new_buffer;

        while buffer.height() >= settings.split_height {
            let cut_height = find_cut_line(&buffer, 0, settings.split_height, settings);
            let part = image::imageops::crop_imm(&buffer, 0, 0, target_width, cut_height).to_image();
            let filename = format!("{:02}{}", file_count, settings.output_type);
            part.save(out_folder.join(filename))?;
            file_count += 1;

            let remaining_h = buffer.height() - cut_height;
            let mut next_buffer = RgbaImage::from_pixel(target_width, remaining_h, fill_pixel);
            let remaining_part = image::imageops::crop_imm(&buffer, 0, cut_height, target_width, remaining_h).to_image();
            image::imageops::overlay(&mut next_buffer, &remaining_part, 0, 0);
            buffer = next_buffer;
        }
    }

    if buffer.height() > 0 {
        let filename = format!("{:02}{}", file_count, settings.output_type);
        buffer.save(out_folder.join(filename))?;
    }
    Ok(out_folder.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn run_smart_stitch(app: AppHandle, settings: StitchSettings) -> Result<(), String> {
    let res = std::thread::spawn(move || {
        let start_time = Instant::now();
        let root_input = PathBuf::from(&settings.input_path);
        let mut folders_to_process = Vec::new();

        if settings.batch_mode {
            if let Ok(entries) = fs::read_dir(&root_input) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() && !get_image_files(&entry.path()).is_empty() {
                        folders_to_process.push(entry.path());
                    }
                }
            }
            folders_to_process.sort_by(|a, b| natord::compare(a.file_name().unwrap().to_str().unwrap(), b.file_name().unwrap().to_str().unwrap()));
        } 
        
        if folders_to_process.is_empty() { folders_to_process.push(root_input); }
        let total = folders_to_process.len();

        for (i, folder) in folders_to_process.iter().enumerate() {
            let out_root = if settings.output_path.is_empty() { PathBuf::new() } else { PathBuf::from(&settings.output_path) };
            
            let current_output_path = match process_single_folder(&app, folder, &out_root, &settings, i, total) {
                Ok(path) => path,
                Err(e) => { 
                    let _ = app.emit("status", format!("Error in {:?}: {}", folder, e)); 
                    return Err(anyhow::anyhow!(e)); 
                }
            };

            if settings.enable_post_process && !settings.post_process_path.is_empty() {
                let _ = app.emit("status", "Running Post Process...");
                let template_args = split_args(&settings.post_process_args);
                let final_args: Vec<String> = template_args.iter()
                    .map(|arg| arg.replace("{output}", &current_output_path))
                    .collect();

                let output = Command::new(&settings.post_process_path).args(&final_args).output();
                match output {
                    Ok(out) => {
                         if !out.status.success() {
                             let err_msg = String::from_utf8_lossy(&out.stderr);
                             let _ = app.emit("status", format!("Post Process Failed: {}", err_msg));
                         } else {
                             let _ = app.emit("status", "Post Process finished.");
                         }
                    }
                    Err(e) => { let _ = app.emit("status", format!("Could not run script: {}", e)); }
                }
            }
        }

        let duration = start_time.elapsed();
        let _ = app.emit("status", format!("Done in {:.2}s", duration.as_secs_f64()));
        let _ = app.emit("progress", 100.0);
        Ok(())
    }).join();

    match res {
        Ok(result) => result.map_err(|e| e.to_string()),
        Err(_) => Err("Thread panicked".to_string()),
    }
}