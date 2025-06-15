use anyhow::Result;

mod ocr_processor;
// use ocr_processor::process_image;
use std::error::Error;
use std::time::Duration;
use thirtyfour::prelude::*;

async fn setup_driver() -> Result<WebDriver> {
    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new("http://localhost:49250", caps).await?;

    println!("Driver setup completed successfully!");

    Ok(driver)
}

async fn setup_oper(driver: &WebDriver) -> Result<()> {
    driver
        .goto("https://fmr.nbtc.go.th/NBTCROS/Login.aspx")
        .await?;
    let login = driver.find(By::Id("UserName")).await?;
    login.send_keys("").await?;
    let password = driver.find(By::Id("Password")).await?;
    password.send_keys("").await?;
    let submit = driver.find(By::Id("bLogin")).await?;
    submit.click().await?;
    Ok(())
}

async fn navigate_to_fm(driver: &WebDriver) -> Result<()> {
    let click_oper_box = driver
        .query(By::Css("a.nbtcros-sectionpage--item[onclick*='Oper']"))
        .single()
        .await?;
    click_oper_box.wait_until().clickable().await?;
    click_oper_box.click().await?;

    let menu_items = driver
        .query(By::XPath("//a[contains(text(), 'งานตรวจสอบคลื่นความถี่')]"))
        .single()
        .await?;

    menu_items.wait_until().clickable().await?;
    menu_items.click().await?;

    let submenu_locator = By::XPath("//a[contains(text(), '4.การตรวจสอบมาตรฐานการแพร่')]");
    let submenu_item = menu_items.query(submenu_locator).single().await?;

    submenu_item.wait_until().clickable().await?;
    submenu_item.click().await?;

    let submenu_item_2 = driver
        .query(By::LinkText("การตรวจสอบมาตรฐานการแพร่"))
        .single()
        .await?;

    submenu_item_2.wait_until().clickable().await?;
    submenu_item_2.click().await?;

    let click_box_add = driver.query(By::Css(".iso-icon--plus")).single().await?;
    click_box_add.click().await?;

    Ok(())
}

// async fn automate_fm

#[tokio::main]
async fn main() -> Result<()> {
    // dotenvy::dotenv().ok();

    // let image_path = "../855.png";
    // println!("Processing image: {}", image_path);

    // let ocr_result = process_image(image_path).await?;
    // println!("OCR Result determined: {:?}", ocr_result);

    // println!("\nProcess completed successfully!");

    //implement automate browser
    let driver = setup_driver().await?;
    setup_oper(&driver).await?;
    navigate_to_fm(&driver).await?;
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    driver.quit().await?;

    Ok(())
}
