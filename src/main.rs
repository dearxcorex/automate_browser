use anyhow::Result;
use dotenvy; // Add this import for dotenvy
use std::env;
mod ocr_processor;
// use ocr_processor::process_image;
use std::error::Error;
use std::time::Duration;
use thirtyfour::{components::SelectElement, prelude::*};
use serde_json::Value as JsonValue; 

async fn setup_driver() -> Result<WebDriver> {
    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new("http://localhost:50269", caps).await?;

    println!("Driver setup completed successfully!");

    Ok(driver)
}

async fn setup_oper(driver: &WebDriver) -> Result<()> {
    let username_nbtc = env::var("NBTC_USERNAME").expect("NBTC_USERNAME not set");
    let password_nbtc = env::var("NBTC_PASSWORD").expect("NBTC_PASSWORD not set");
    driver
        .goto("https://fmr.nbtc.go.th/NBTCROS/Login.aspx")
        .await?;
    let login = driver.find(By::Id("UserName")).await?;
    login.send_keys(&username_nbtc).await?;

    let password = driver.find(By::Id("Password")).await?;
    password.send_keys(&password_nbtc).await?;
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


async fn open_panel(driver: &WebDriver, idx: u8) -> WebDriverResult<()> {
    let selector = format!("p[href='#collapse_panel_{}']", idx);
    let elem = driver.find(By::Css(&selector)).await?;
 
    elem.scroll_into_view().await?;
    tokio::time::sleep(Duration::from_millis(500)).await;
    elem.click().await?;
    Ok(())
}

async fn automate_fm(driver: &WebDriver) -> Result<(), WebDriverError> {
    let search_box = driver
        .query(By::Css("button.btn.btn-primary.x-add"))
        .with_text("ค้นหา")
        .single()
        .await?;

    search_box.click().await?;

    //model popup
    let search_model = driver
        .query(By::ClassName("modal-content"))
        .and_displayed()
        .single()
        .await?;
    let iframe = &search_model
        .query(By::Tag("iframe"))
        .and_displayed()
        .single()
        .await?;

    iframe.clone().enter_frame().await?;

    let station_type = driver.find(By::Id("StnTypeID")).await?;
    let select_element = SelectElement::new(&station_type).await?;
    select_element.select_by_index(8).await?;

    let station_id = driver.find(By::Id("SiteCode")).await?;
    station_id.send_keys("05520402").await?;

    // search base data
    let search_base_data = driver.find(By::Id("SrcData")).await?;
    let select_search_base_data = SelectElement::new(&search_base_data).await?;
    select_search_base_data.select_by_index(0).await?;

    let click_list_fm = driver.find(By::ClassName("iso-icon--search")).await?;
    click_list_fm.click().await?;

    let list_fm = driver.query(By::XPath("//a[text()='1']")).single().await?;
    list_fm.wait_until().clickable().await?;
    list_fm.click().await?;

    //exit frame
    driver.enter_parent_frame().await?;

    // toggle and scroll into view 

    for i in 1..=4 {
        println!("Opening panel {}", i);
        open_panel(&driver, i).await?;
    }
    // let toggle_detail_station_1 = driver.find(By::Css("p[href='#collapse_panel_1']")).await?;
    // toggle_detail_station_1.scroll_into_view().await?;
    // toggle_detail_station_1.click().await?;
    // let toggle_detail_station_2 = driver.find(By::Css("p[href='#collapse_panel_2']")).await?;
    // toggle_detail_station_2.scroll_into_view().await?;
    // toggle_detail_station_2.click().await?;
    // let toggle_detail_station_3 = driver.find(By::Css("p[href='#collapse_panel_3']")).await?;
    // toggle_detail_station_3.scroll_into_view().await?;
    // toggle_detail_station_3.click().await?;

    // let toggle_detail_station_4 = driver.find(By::Css("p[href='#collapse_panel_4']")).await?;
    // toggle_detail_station_4.scroll_into_view().await?;
    // toggle_detail_station_4.click().await?;
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    // let image_path = "797.png";
    // println!("Processing image: {}", image_path);

    // let ocr_result = process_image(image_path).await?;
    // println!("OCR Result determined: {:?}", ocr_result);

    // println!("\nProcess completed successfully!");

    //implement automate browser
    let driver = setup_driver().await?;
    setup_oper(&driver).await?;
    navigate_to_fm(&driver).await?;

    automate_fm(&driver).await?;
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    driver.quit().await?;

    Ok(())
}
