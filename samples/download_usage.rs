//! Download功能使用示例
//! 
//! 本示例展示了如何使用rust-caller库的下载功能

use caller::{download, init_config, DownloadResult};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化配置
    init_config()?;

    println!("=== rust-caller 下载功能示例 ===\n");

    // 示例1: 自动检测文件格式下载
    example_auto_detect().await?;

    // 示例2: 手动指定文件格式
    example_manual_extension().await?;

    // 示例3: 带参数的下载
    example_with_params().await?;

    // 示例4: 保存下载的文件
    example_save_file().await?;

    // 示例5: 下载并处理内容
    example_process_content().await?;

    Ok(())
}

/// 示例1: 自动检测文件格式下载
async fn example_auto_detect() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- 示例1: 自动检测文件格式 ---");
    
    // 下载文件，系统会根据Content-Type自动检测文件格式
    // let result = download("api.download", None, None).await?;
    
    println!("下载成功！");
    println!("  状态码: {}", 200); // result.status_code
    println!("  文件大小: {} bytes", 1024); // result.size()
    println!("  文件大小: {}", "1.00 KB"); // result.size_human()
    println!("  内容类型: {}", "application/json"); // result.content_type
    println!("  文件扩展名: {}", "json"); // result.file_extension
    
    println!();
    Ok(())
}

/// 示例2: 手动指定文件格式
async fn example_manual_extension() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- 示例2: 手动指定文件格式 ---");
    
    // 下载时手动指定文件扩展名
    // 即使服务器返回的Content-Type不正确，也可以强制指定扩展名
    let custom_extension = Some("csv".to_string());
    // let result = download("api.download", None, custom_extension).await?;
    
    println!("下载成功并指定扩展名为: {}", "csv");
    println!("  文件扩展名: {}", "csv"); // result.file_extension
    
    println!();
    Ok(())
}

/// 示例3: 带参数的下载
async fn example_with_params() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- 示例3: 带参数的下载 ---");
    
    // 下载时传递参数
    let params = HashMap::from([
        ("file_id".to_string(), "12345".to_string()),
        ("version".to_string(), "2.0".to_string()),
    ]);
    
    // let result = download("api.download_file", Some(params), None).await?;
    
    println!("下载成功！");
    println!("  传递的参数: file_id=12345, version=2.0");
    
    println!();
    Ok(())
}

/// 示例4: 保存下载的文件
async fn example_save_file() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- 示例4: 保存下载的文件 ---");
    
    // 下载文件
    // let result = download("api.download", None, None).await?;
    
    // 保存到指定目录，自动生成文件名
    // let filename = result.save("./downloads", "myfile")?;
    // println!("文件已保存: ./downloads/{}", filename);
    
    // 或者保存到指定路径
    // result.save_to_file("./downloads/myfile.pdf")?;
    // println!("文件已保存到: ./downloads/myfile.pdf");
    
    println!("文件已保存到指定目录");
    
    println!();
    Ok(())
}

/// 示例5: 下载并处理内容
async fn example_process_content() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- 示例5: 下载并处理内容 ---");
    
    // 下载文件
    // let result = download("api.download_text", None, None).await?;
    
    // 获取文件大小
    let size = 1024; // result.size()
    println!("文件大小: {} bytes", size);
    
    // 获取人类可读的文件大小
    let size_human = "1.00 KB"; // result.size_human()
    println!("人类可读大小: {}", size_human);
    
    // 如果是文本文件，可以读取内容
    // let text = result.as_text()?;
    // println!("文件内容（前100字符）: {}", &text[..100.min(text.len())]);
    
    // 直接访问二进制内容
    let content = b"示例内容"; // &result.content
    println!("二进制内容长度: {} bytes", content.len());
    
    println!();
    Ok(())
}

/// 高级示例: 处理Content-Disposition头
async fn example_content_disposition() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- 高级示例: Content-Disposition处理 ---");
    
    // 下载文件
    // let result = download("api.download", None, None).await?;
    
    // 如果服务器返回Content-Disposition头，会自动提取文件名
    // 例如: Content-Disposition: attachment; filename="report.pdf"
    // if let Some(filename) = &result.suggested_filename {
    //     println!("服务器建议的文件名: {}", filename);
    // }
    
    println!("自动处理Content-Disposition头中的文件名");
    
    println!();
    Ok(())
}

/// 高级示例: 错误处理
async fn example_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- 高级示例: 错误处理 ---");
    
    // 尝试下载不存在的API方法
    match download("invalid.api.method", None, None).await {
        Ok(result) => {
            println!("下载成功: {:?}", result);
        }
        Err(e) => {
            println!("下载失败: {}", e);
            // 可以根据错误类型进行不同处理
            // if e.is_network_error() { ... }
            // if e.is_config_error() { ... }
        }
    }
    
    println!();
    Ok(())
}