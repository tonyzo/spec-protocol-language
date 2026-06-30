//! SPEC Protocol 统一 CLI 入口
//!
//! 将 spec_cli（合规引擎）和 term_extractor（术语提取）统一为 `spec` 一个命令。
//!
//! 命令分组：
//!   合规引擎: check, discover, confirm
//!   术语提取: extract, generate-rules, generate-docs, validate, diff, ocr, batch-extract, detect-conflicts

use std::process::{exit, Command};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_usage();
        exit(1);
    }

    let cmd = &args[1];
    let rest = &args[2..];

    match cmd.as_str() {
        // 合规引擎命令 → spec_cli
        "check" | "discover" | "confirm" => {
            delegate("spec_cli", cmd, rest);
        }
        // 术语提取命令 → term_extractor
        "extract" | "generate-rules" | "generate-docs" | "validate" | "diff" | "ocr"
        | "batch-extract" | "detect-conflicts" => {
            delegate("term_extractor", cmd, rest);
        }
        "--help" | "-h" | "help" => print_usage(),
        _ => {
            eprintln!("未知命令: {}", cmd);
            print_usage();
            exit(1);
        }
    }
}

/// 将命令委托给对应的二进制（spec_cli 或 term_extractor）
fn delegate(bin: &str, cmd: &str, args: &[String]) {
    let status = Command::new(bin)
        .arg(cmd)
        .args(args)
        .status()
        .unwrap_or_else(|e| {
            eprintln!(
                "错误: 无法执行 '{}'。请确保已运行 cargo install 或在 PATH 中可用。",
                bin
            );
            eprintln!("详情: {}", e);
            exit(127);
        });
    exit(status.code().unwrap_or(1));
}

fn print_usage() {
    eprintln!("SPEC Protocol 统一 CLI");
    eprintln!();
    eprintln!("用法: spec <command> [options]");
    eprintln!();
    eprintln!("合规引擎命令 (→ spec_cli):");
    eprintln!("  check            执行合规校核");
    eprintln!("  discover         发现候选审查点");
    eprintln!("  confirm          确认并沉淀审查点");
    eprintln!();
    eprintln!("术语提取命令 (→ term_extractor):");
    eprintln!("  extract          从标准文档提取术语");
    eprintln!("  generate-rules   从参数映射生成规则 YAML");
    eprintln!("  generate-docs    生成术语表文档");
    eprintln!("  validate         验证规则 YAML");
    eprintln!("  diff             比较参数映射差异");
    eprintln!("  ocr              OCR 预处理");
    eprintln!("  batch-extract    批量提取");
    eprintln!("  detect-conflicts 跨标准冲突检测");
    eprintln!();
    eprintln!("示例:");
    eprintln!("  spec extract -i GB150.3.pdf -o output/ --standard GB150.3");
    eprintln!("  spec generate-rules -i output/param_mapping.json -o rules/library/GB150.3/");
    eprintln!("  spec validate -i rules/library/GB150.3/5.3-内压圆筒.yaml");
    eprintln!("  spec check --design-type 内压圆筒 --params input/params.json --format json");
    eprintln!("  spec detect-conflicts --standards output/GB150.3 output/NB47012 -o report.md");
}
