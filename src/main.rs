// src/main.rs

use anyhow::Result;

// ประกาศให้ Rust รู้จักโมดูลที่เราสร้างขึ้น
mod ocr_processor;
// เราจะคอมเมนต์ส่วนนี้ไว้ก่อนตามโค้ดที่คุณให้มา แต่สามารถเปิดใช้ทีหลังได้
// mod file_handler;

// นำเข้าฟังก์ชันและ enum ที่เราจะใช้
use ocr_processor::process_image;
// use file_handler::handle_file_based_on_result;

#[tokio::main]
async fn main() -> Result<()> {
    // โหลด .env file
    dotenvy::dotenv().ok();

    // 1. กำหนดไฟล์ที่จะประมวลผล
    let image_path = "../855.png";
    println!("Processing image: {}", image_path);

    // 2. เรียกใช้โมดูล OCR เพื่อรับผลลัพธ์
    // `.await` ใช้ได้เพราะ `process_image` เป็น `async`
    let ocr_result = process_image(image_path).await?;
    println!("OCR Result determined: {:?}", ocr_result);

    // คุณสามารถเพิ่มโค้ดเพื่อจัดการกับ ocr_result ตรงนี้ได้เลย

    println!("\nProcess completed successfully!");

    // Ok(()) ต้องเป็น statement สุดท้ายของฟังก์ชันที่คืนค่า Result
    Ok(())
}
