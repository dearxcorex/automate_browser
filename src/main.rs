use anyhow::Result;
use dotenvy; // Add this import for dotenvy
use std::env;
mod ocr_processor;
use ocr_processor::process_image;
use serde_json::Value as JsonValue;
use std::error::Error;
use std::time::Duration;
use thirtyfour::{components::SelectElement, prelude::*};

async fn setup_driver() -> Result<WebDriver> {
    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new("http://localhost:49332", caps).await?;

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

async fn get_modal_iframe(driver: &WebDriver) -> WebDriverResult<WebElement> {
    let search_model = driver
        .query(By::ClassName("modal-content"))
        .and_displayed()
        .single()
        .await?;
    let iframe = search_model
        .query(By::Tag("iframe"))
        .and_displayed()
        .single()
        .await?;
    Ok(iframe)
}

async fn automate_fm(driver: &WebDriver) -> Result<(), WebDriverError> {
    let search_box = driver
        .query(By::Css("button.btn.btn-primary.x-add"))
        .with_text("ค้นหา")
        .single()
        .await?;

    search_box.click().await?;

    let iframe = get_modal_iframe(&driver).await?;
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

    //write data fm detail panel
    let fm_detail_panel = driver.find(By::Id("DetAnt")).await?;
    let select_element = SelectElement::new(&fm_detail_panel).await?;
    select_element.select_by_index(1).await?;

    let fm_ant_panel = driver.find(By::Id("DetAerial")).await?;
    let select_ant = SelectElement::new(&fm_ant_panel).await?;

    select_ant.select_by_index(1).await?;

    //get text freq fm
    let get_feq_1 = driver.query(By::Id("FreqMhz")).single().await?;
    let freq_text = get_feq_1.text().await.unwrap();
    let trimmed_freq_text = freq_text.trim().to_string();

    let freq_input = driver.query(By::Id("DetFrq")).single().await?;
    freq_input.send_keys(&trimmed_freq_text).await?;

    //select Ant type
    let dropdown_ant_type = driver.find(By::Id("AntID")).await?;
    let select_ant_type = SelectElement::new(&dropdown_ant_type).await?;

    let first_text = select_ant_type.first_selected_option().await?;
    let get_text = first_text.text().await?;
    get_text.trim().to_string();

    if &get_text == "กรุณาเลือก" {
        select_ant_type.select_by_index(1).await?;
    }
    println!("{:?}", get_text);

    //cable
    let cable_type = driver.find(By::Id("CableID")).await?;
    let select_cable_type = SelectElement::new(&cable_type).await?;
    select_cable_type.select_by_index(5).await?;

    //power
    let fm_power = driver.find(By::Id("DetPow")).await?;
    let power_value = fm_power.attr("value").await.unwrap();

    if let Some(value_string) = power_value {
        if value_string.trim().is_empty() {
            fm_power.send_keys("500").await?;
        }
    }
    //gain
    let fm_gain = driver.find(By::Id("DetDBI")).await?;
    let gain_value = fm_gain.attr("value").await.unwrap();

    if let Some(value_string) = gain_value {
        if value_string.trim().is_empty() {
            fm_gain.send_keys("6").await?;
        }
    }

    //hight

    let hight = driver.find(By::Id("DetAntHeight")).await?;
    let hight_value = hight.attr("value").await.unwrap();
    if let Some(value_string) = hight_value {
        if value_string.trim().is_empty() {
            hight.send_keys("60").await?;
        }
    }

    Ok(())
}

async fn panel_2(driver: &WebDriver) -> Result<()> {
    
   for i in 1..=3 {
     
    let panel_2 = driver.query(By::Css("button[onclick='editFqItem(0)']")).single().await?;
    panel_2.click().await?;
    let iframe = get_modal_iframe(&driver).await?;
    iframe.clone().enter_frame().await?;

    let dropdown_patt = driver.find(By::Id("DiffPara")).await?;
    let select_element = SelectElement::new(&dropdown_patt).await?;
    select_element.select_by_index(i).await?; 

    let result_dropdown_patt = driver.find(By::Id("DiffRes")).await?; 
    let select_dropdown_patt = SelectElement::new(&result_dropdown_patt).await?;
    select_dropdown_patt.select_by_index(1).await?;

    //save 
    let save = driver.find(By::Css("button.btn.btn-primary i.iso-icon--save")).await?; 
    save.click().await?; 

    //exit frame
    tokio::time::sleep(Duration::from_millis(500)).await;
    driver.enter_parent_frame().await?;

   }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let image_path = "/Users/deardevx/Documents/my_stufF/rust_for_noob/automation_browser/src/796.png";
    println!("Processing image: {}", image_path);

    let ocr_result = process_image(image_path).await?;
    println!("OCR Result determined: {:?}", ocr_result);

    println!("\nProcess completed successfully!");

    //implement automate browser
    let driver = setup_driver().await?;
    setup_oper(&driver).await?;
    navigate_to_fm(&driver).await?;

    automate_fm(&driver).await?;
    panel_2(&driver).await?;
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    driver.quit().await?;

    Ok(())
}
