use anyhow::Result;
use qrcode::render::unicode;
use qrcode::QrCode;

pub async fn print_as_qr_code(data: &str) -> Result<()> {
    let code = QrCode::new(data)?;
    let image = code
        .render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Dark)
        .light_color(unicode::Dense1x2::Light)
        .build();
    println!("{}", image);
    Ok(())
}
