use crate::diagnostics::CheckResult;
use crate::sysinfo::SystemInfo;
use std::fs;

pub fn generate(
    info: &SystemInfo,
    checks: &[CheckResult],
    score: u32,
    verdict: &str,
) -> anyhow::Result<()> {
    let mut checks_html = String::new();
    for c in checks {
        let class = if c.status { "pass" } else { "fail" };
        let text = if c.status { "PASS" } else { "FAIL" };
        checks_html.push_str(&format!(
            r#"<tr><td>{}</td><td class="{}">{}</td><td>{}</td><td>{}</td></tr>"#,
            c.name, class, text, c.weight, c.tip
        ));
    }

    let html = format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>Reiop Report</title>
<style>
  @import url('https://fonts.googleapis.com/css2?family=Roboto:wght@400;500;700&display=swap');
  
  body {{ 
    background-color: #0b1a26;
    color: #b0c4de; 
    font-family: 'Roboto', 'Segoe UI', Tahoma, sans-serif; 
    margin: 0; 
    padding: 40px; 
    min-height: 100vh;
    overflow-x: hidden;
  }}
  
  .fish-container {{
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    z-index: -1;
    pointer-events: none;
  }}
  
  .fish {{
    position: absolute;
    width: 120px;
    opacity: 0.15;
    animation: swim 25s linear infinite;
  }}
  
  .fish1 {{ top: 10%; animation-duration: 20s; }}
  .fish2 {{ top: 40%; animation-duration: 35s; animation-delay: -5s; width: 80px; }}
  .fish3 {{ top: 70%; animation-duration: 28s; animation-delay: -15s; width: 150px; }}
  .fish4 {{ top: 85%; animation-duration: 40s; animation-delay: -25s; width: 60px; }}
  
  @keyframes swim {{
    0% {{ transform: translateX(-20vw) translateY(0) scaleX(-1); }}
    49% {{ transform: translateX(120vw) translateY(20px) scaleX(-1); }}
    50% {{ transform: translateX(120vw) translateY(20px) scaleX(1); }}
    99% {{ transform: translateX(-20vw) translateY(0) scaleX(1); }}
    100% {{ transform: translateX(-20vw) translateY(0) scaleX(-1); }}
  }}
  
  .container {{ 
    max-width: 1000px; 
    margin: 0 auto; 
    background: rgba(16, 33, 50, 0.85); 
    padding: 40px; 
    border-radius: 12px; 
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3); 
    backdrop-filter: blur(8px); 
    border: 1px solid #1e3a5f; 
  }}
  
  h1 {{ 
    color: #e0e0e0; 
    text-align: center; 
    margin-bottom: 30px; 
    font-weight: 700;
    letter-spacing: 1px;
  }}
  
  h2 {{ 
    color: #e0e0e0; 
    border-bottom: 2px solid #1e3a5f; 
    padding-bottom: 10px; 
    margin-top: 40px;
    font-weight: 500;
  }}
  
  .score {{ 
    font-size: 48px; 
    font-weight: 700; 
    color: #4fc3f7; 
  }}
  
  .specs {{ 
    display: grid; 
    grid-template-columns: 1fr 1fr; 
    gap: 15px; 
    background: rgba(16, 33, 50, 0.9); 
    padding: 25px; 
    border-radius: 8px; 
    margin-bottom: 30px; 
    border-left: 4px solid #4fc3f7; 
  }}
  
  .specs p {{ margin: 8px 0; font-size: 15px; }}
  .specs strong {{ color: #e0e0e0; }}
  
  table {{ 
    width: 100%; 
    border-collapse: collapse; 
    margin-top: 20px; 
    background: rgba(11, 26, 38, 0.9); 
    border-radius: 8px; 
    overflow: hidden; 
  }}
  
  th, td {{ 
    border: 1px solid #1e3a5f; 
    padding: 15px 20px; 
    text-align: left; 
  }}
  
  th {{ 
    background: rgba(16, 33, 50, 1); 
    color: #e0e0e0; 
    font-weight: 500; 
    text-transform: uppercase; 
    letter-spacing: 1px; 
    font-size: 13px; 
  }}
  
  tr:nth-child(even) {{ background: rgba(16, 33, 50, 0.4); }}
  
  .pass {{ color: #81c784; font-weight: 700; }}
  .fail {{ color: #e57373; font-weight: 700; }}
  
  footer {{ 
    text-align: center; 
    color: #78909c; 
    margin-top: 50px; 
    font-size: 14px; 
  }}
</style>
</head>
<body>

<div class="fish-container">
  <svg class="fish fish1" viewBox="0 0 100 50" xmlns="http://www.w3.org/2000/svg">
    <path d="M10,25 C30,10 60,10 80,25 C60,40 30,40 10,25 Z" fill="#4fc3f7"/>
    <path d="M80,25 L95,10 L95,40 Z" fill="#4fc3f7"/>
    <circle cx="25" cy="20" r="3" fill="#ffffff"/>
  </svg>
  <svg class="fish fish2" viewBox="0 0 100 50" xmlns="http://www.w3.org/2000/svg">
    <path d="M10,25 C30,10 60,10 80,25 C60,40 30,40 10,25 Z" fill="#29b6f6"/>
    <path d="M80,25 L95,10 L95,40 Z" fill="#29b6f6"/>
    <circle cx="25" cy="20" r="3" fill="#ffffff"/>
  </svg>
  <svg class="fish fish3" viewBox="0 0 100 50" xmlns="http://www.w3.org/2000/svg">
    <path d="M10,25 C30,10 60,10 80,25 C60,40 30,40 10,25 Z" fill="#03a9f4"/>
    <path d="M80,25 L95,10 L95,40 Z" fill="#03a9f4"/>
    <circle cx="25" cy="20" r="3" fill="#ffffff"/>
  </svg>
  <svg class="fish fish4" viewBox="0 0 100 50" xmlns="http://www.w3.org/2000/svg">
    <path d="M10,25 C30,10 60,10 80,25 C60,40 30,40 10,25 Z" fill="#81d4fa"/>
    <path d="M80,25 L95,10 L95,40 Z" fill="#81d4fa"/>
    <circle cx="25" cy="20" r="3" fill="#ffffff"/>
  </svg>
</div>

<div class="container">
  <h1>Reiop System Report</h1>
  
  <div class="specs">
    <p><strong>OS:</strong> {} Build {}</p>
    <p><strong>CPU:</strong> {} cores / {} threads</p>
    <p><strong>RAM:</strong> {:.1} GB {}</p>
    <p><strong>GPU:</strong> {} ({}) - {:.1} GB VRAM</p>
    <p><strong>Disk:</strong> {}</p>
    <p><strong>Refresh Rate:</strong> {} Hz</p>
  </div>
  
  <h2>Performance Score: <span class="score">{}</span> / 100 - {}</h2>
  
  <table>
    <tr><th>Check</th><th>Status</th><th>Weight</th><th>Tip</th></tr>
    {}
  </table>
  
  <footer>Generated by Reiop<br>Discord &amp; Instagram: arkanquixote</footer>
</div>
</body>
</html>"##,
        info.os_name,
        info.os_version,
        info.cpu_cores,
        info.cpu_threads,
        info.ram_gb,
        info.ram_type,
        info.gpu_name,
        info.gpu_vendor,
        info.vram_gb,
        info.disk_type,
        info.refresh_rate,
        score,
        verdict,
        checks_html
    );

    fs::write("Reiop_Report.html", html)?;
    let _ = open::that("Reiop_Report.html");
    Ok(())
}
