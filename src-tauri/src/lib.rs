use anyhow::Result;
use image::{imageops::FilterType, RgbaImage};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};

#[derive(serde::Deserialize, Clone, Debug)]
struct StitchSettings {
    input_path: String,
    output_path: String,
    output_type: String, 
    split_height: u32,
    // 0=No Enforcement, 1=Auto(First), 2=Min, 3=Custom
    width_enforce_type: u8, 
    custom_width: u32,
    sensitivity: u8,
    scan_step: u32,
    ignorable_margin: u32,
}

fn get_sorted_images(path: &str) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(path)
        .unwrap_or_else(|_| panic!("Could not read dir"))
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
            matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "webp" | "bmp" | "tiff")
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

fn find_smart_cut(img: &RgbaImage, start_y: u32, target_h: u32, sensitivity: u8, margin: u32, step: u32) -> u32 {
    let max_h = img.height();
    let threshold = (255.0 * (1.0 - (sensitivity as f32 / 100.0))) as i16;
    let width = img.width();
    let cut_y = start_y + target_h;
    
    if cut_y >= max_h { return max_h - start_y; }

    let search_limit_up = (target_h as f32 * 0.4) as u32; 
    let mut current_offset = 0;
    let mut searching_up = true;
    let mut scan_y = cut_y;

    loop {
        if scan_y >= max_h { break; } 
        let row_is_clean = {
            let mut clean = true;
            let mut prev_val = -1000;
            for x in margin..(width - margin) {
                let p = img.get_pixel(x, scan_y);
                let val = (p[0] as i16 + p[1] as i16 + p[2] as i16) / 3;
                if prev_val != -1000 && (val - prev_val).abs() > threshold {
                    clean = false;
                    break;
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

#[tauri::command]
async fn run_smart_stitch(app: AppHandle, settings: StitchSettings) -> Result<(), String> {
    stitch_logic(&app, settings).map_err(|e| e.to_string())
}

fn stitch_logic(app: &AppHandle, settings: StitchSettings) -> Result<()> {
    let input_files = get_sorted_images(&settings.input_path);
    if input_files.is_empty() {
        app.emit("status", "No images found")?;
        return Ok(());
    }

    // 1. Output Folder Logic (Sibling of input)
    let out_folder = if settings.output_path.is_empty() {
        let in_path = Path::new(&settings.input_path);
        let parent = in_path.parent().unwrap_or(Path::new("."));
        let dir_name = in_path.file_name().unwrap_or_default().to_string_lossy();
        parent.join(format!("{} [Stitched]", dir_name))
    } else {
        PathBuf::from(&settings.output_path)
    };
    fs::create_dir_all(&out_folder)?;

    // 2. Initial Buffer Setup
    let first_img = image::open(&input_files[0])?;
    
    // Determine target width logic
    let mut target_width = match settings.width_enforce_type {
        0 => first_img.width(), // No Enforce (Dynamic)
        1 => first_img.width(), // Auto Uniform (Match First)
        2 => input_files.iter().filter_map(|p| image::image_dimensions(p).ok()).map(|(w, _)| w).min().unwrap_or(first_img.width()),
        3 => settings.custom_width,
        _ => first_img.width(),
    };

    let mut buffer: RgbaImage = RgbaImage::new(target_width, 0);
    let mut file_count = 1;
    let total_files = input_files.len();

    // 3. Processing Loop
    for (idx, path) in input_files.iter().enumerate() {
        app.emit("status", format!("Processing {}/{}", idx + 1, total_files))?;
        app.emit("progress", (idx as f64 / total_files as f64) * 100.0)?;

        let mut img = image::open(path)?;

        // --- MODE 0: NO ENFORCEMENT LOGIC ---
        if settings.width_enforce_type == 0 {
            // If width changes and buffer isn't empty, we must flush the buffer completely
            if buffer.height() > 0 && img.width() != buffer.width() {
                // Save remaining buffer immediately
                let filename = format!("{:02}{}", file_count, settings.output_type);
                buffer.save(out_folder.join(filename))?;
                file_count += 1;
                
                // Reset buffer with NEW width
                target_width = img.width();
                buffer = RgbaImage::new(target_width, 0);
            }
        } 
        // --- OTHER MODES: RESIZE LOGIC ---
        else if img.width() != target_width {
             let ratio = target_width as f64 / img.width() as f64;
             let new_h = (img.height() as f64 * ratio) as u32;
             img = img.resize(target_width, new_h, FilterType::Lanczos3);
        }

        let img_rgba = img.to_rgba8();

        // Append to Buffer
        let new_height = buffer.height() + img_rgba.height();
        let mut new_buffer = RgbaImage::new(target_width, new_height);
        image::imageops::overlay(&mut new_buffer, &buffer, 0, 0);
        image::imageops::overlay(&mut new_buffer, &img_rgba, 0, buffer.height() as i64);
        buffer = new_buffer;

        // Cut loop
        while buffer.height() >= settings.split_height {
            let cut_height = find_smart_cut(&buffer, 0, settings.split_height, settings.sensitivity, settings.ignorable_margin, settings.scan_step);

            let part = image::imageops::crop_imm(&buffer, 0, 0, target_width, cut_height).to_image();
            let filename = format!("{:02}{}", file_count, settings.output_type);
            part.save(out_folder.join(filename))?;
            file_count += 1;

            let remaining_h = buffer.height() - cut_height;
            let mut next_buffer = RgbaImage::new(target_width, remaining_h);
            let remaining_part = image::imageops::crop_imm(&buffer, 0, cut_height, target_width, remaining_h).to_image();
            image::imageops::overlay(&mut next_buffer, &remaining_part, 0, 0);
            buffer = next_buffer;
        }
    }

    // Save final remainder
    if buffer.height() > 0 {
        let filename = format!("{:02}{}", file_count, settings.output_type);
        buffer.save(out_folder.join(filename))?;
    }

    app.emit("status", "Done!")?;
    app.emit("progress", 100.0)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![run_smart_stitch])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}