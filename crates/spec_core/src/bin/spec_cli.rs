//! SPEC Protocol CLI —— 命令行合规校核工具
//!
//! 子命令：
//!   check    — 执行合规校核（使用已沉淀的审查点集合或直接指定规则文件）
//!   discover — 从规则库发现候选审查点
//!   confirm  — 确认并沉淀审查点集合
//!
//! 用法示例：
//!   # 首次审查：发现候选
//!   spec_cli discover --params input/params.json --output candidates.json
//!
//!   # 确认并沉淀（编辑 candidates.json，标记 confirmed: true）
//!   spec_cli confirm --input candidates.json --design-type 内压圆筒
//!
//!   # 后续审查：直接加载已沉淀的审查点集合
//!   spec_cli check --design-type 内压圆筒 --params input/params.json --format json
//!
//!   # 直接指定规则文件（向后兼容）
//!   spec_cli check --rules rules/library/GB150.3/5.3-内压圆筒.yaml --params input/params.json

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use spec_core::{
    CandidateReviewPoint, ComplianceEngine, ComplianceReport, DesignContext, DiscoveryEngine,
    JsonWriter, ParameterTable, ReportWriter, RvmWriter, SpecRule,
};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage(&args[0]);
        process::exit(1);
    }

    let cmd = &args[1];
    match cmd.as_str() {
        "check" => cmd_check(&args[2..]),
        "discover" => cmd_discover(&args[2..]),
        "confirm" => cmd_confirm(&args[2..]),
        "--help" | "-h" | "help" => {
            print_usage(&args[0]);
        }
        _ => {
            // 向后兼容：将整个参数列表作为 check 命令处理
            // 支持旧格式: spec_cli <rules.yaml> <params.json> [--format json|rvm]
            cmd_check(&args[1..]);
        }
    }
}

fn print_usage(prog: &str) {
    eprintln!("SPEC Protocol CLI — 合规校核工具");
    eprintln!();
    eprintln!("用法:");
    eprintln!("  {} check    --design-type <类型> --params <file> [--format json|rvm] [--rules-dir <dir>]", prog);
    eprintln!("  {} check    --rules <file> --params <file> [--format json|rvm]  (直接指定规则)", prog);
    eprintln!("  {} discover --params <file> [--rules-dir <dir>] [--output <file>]", prog);
    eprintln!("  {} confirm  --input <file> --design-type <类型> [--rules-dir <dir>] [--year <year>]", prog);
    eprintln!();
    eprintln!("示例:");
    eprintln!("  {} discover --params input/params.json --output candidates.json", prog);
    eprintln!("  {} confirm  --input candidates.json --design-type 内压圆筒 --year 2024", prog);
    eprintln!("  {} check    --design-type 内压圆筒 --params input/params.json --format json", prog);
}

// ─── check 子命令 ───

fn cmd_check(args: &[String]) {
    let design_type = get_arg(args, "--design-type");
    let rules_file = get_arg(args, "--rules");
    let params_file = get_arg(args, "--params").or_else(|| args.get(0).map(|s| s.as_str()));
    let format = get_arg(args, "--format").unwrap_or("json");
    let rules_dir = get_arg(args, "--rules-dir").unwrap_or("rules");

    let params_file = match params_file {
        Some(f) => f,
        None => {
            eprintln!("错误: 缺少 --params 参数");
            process::exit(1);
        }
    };

    // 加载参数表
    let input = load_input_file(params_file);

    // 获取规则
    let rules: Vec<SpecRule> = if let Some(rf) = rules_file {
        // 直接指定规则文件
        let yaml_text = read_file_or_exit(rf, "规则文件");
        match serde_yaml::from_str(&yaml_text) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("错误: 规则文件 YAML 解析失败: {}", e);
                process::exit(5);
            }
        }
    } else if let Some(dt) = design_type {
        // 从已沉淀的审查点集合加载
        let rules_dir_path = Path::new(rules_dir);
        let discovery = match DiscoveryEngine::from_rules_dir(rules_dir_path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("错误: 无法加载规则库: {}", e);
                process::exit(7);
            }
        };

        if !discovery.has_confirmed_set(dt) {
            eprintln!("错误: 设计类型 '{}' 无已确认审查点集合", dt);
            eprintln!("提示: 请先运行 discover + confirm 流程");
            process::exit(8);
        }

        match discovery.load_confirmed(dt) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("错误: 加载审查点集合失败: {}", e);
                process::exit(9);
            }
        }
    } else {
        eprintln!("错误: 需要指定 --design-type 或 --rules");
        process::exit(1);
    };

    // 构建引擎并执行校核
    let engine = ComplianceEngine::new(rules);
    let report: ComplianceReport =
        engine.check(&input.parameter_table, input.context.unwrap_or_default());

    // 输出报告
    let writer: Box<dyn ReportWriter> = match format {
        "rvm" => Box::new(RvmWriter),
        _ => Box::new(JsonWriter),
    };

    match writer.write(&report) {
        Ok(output) => println!("{}", output),
        Err(e) => {
            eprintln!("错误: 报告生成失败: {}", e);
            process::exit(6);
        }
    }

    // 以退出码反映结果
    let s = &report.summary;
    if s.errors > 0 {
        process::exit(10);
    } else if s.warnings > 0 {
        process::exit(11);
    }
}

// ─── discover 子命令 ───

fn cmd_discover(args: &[String]) {
    let params_file = match get_arg(args, "--params") {
        Some(f) => f,
        None => {
            eprintln!("错误: 缺少 --params 参数");
            process::exit(1);
        }
    };
    let rules_dir = get_arg(args, "--rules-dir").unwrap_or("rules");
    let output_file = get_arg(args, "--output").unwrap_or("candidates.json");

    // 加载参数表
    let input = load_input_file(params_file);

    // 创建发现引擎
    let discovery = match DiscoveryEngine::from_rules_dir(Path::new(rules_dir)) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: 无法加载规则库: {}", e);
            process::exit(7);
        }
    };

    // 发现候选审查点
    let candidates = discovery.discover(&input.parameter_table);

    // 提取设计类型（从参数表的 attrs.element_type 或 context.element_type）
    let design_type = input
        .parameter_table
        .attrs
        .get("element_type")
        .cloned()
        .or_else(|| input.context.as_ref().and_then(|c| c.element_type.clone()))
        .unwrap_or_else(|| "unknown".into());

    // 检查是否已有审查点集合
    if discovery.has_confirmed_set(&design_type) {
        eprintln!("提示: 设计类型 '{}' 已有已确认审查点集合", design_type);
        eprintln!("      如需重新发现，请先清理 rules/confirmed/{}/ 目录", design_type);
    }

    // 输出候选列表（带 confirmed: false 字段供用户编辑）
    let output = DiscoverOutput {
        design_type: design_type.clone(),
        candidates: candidates
            .iter()
            .map(|c| DiscoverCandidate {
                rule: c.rule.clone(),
                match_status: c.match_status.clone(),
                reason: c.reason.clone(),
                confirmed: false,
            })
            .collect(),
    };

    let json = match serde_json::to_string_pretty(&output) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("错误: JSON 序列化失败: {}", e);
            process::exit(12);
        }
    };

    match fs::write(output_file, &json) {
        Ok(_) => {
            eprintln!("已输出 {} 条候选审查点到 {}", candidates.len(), output_file);
            eprintln!("设计类型: {}", design_type);
            eprintln!();
            eprintln!("请编辑 {} 标记 confirmed: true，然后运行:", output_file);
            eprintln!("  spec_cli confirm --input {} --design-type {}", output_file, design_type);

            // 也输出到 stdout 供管道使用
            println!("{}", json);
        }
        Err(e) => {
            eprintln!("错误: 无法写入输出文件 {}: {}", output_file, e);
            process::exit(13);
        }
    }
}

// ─── confirm 子命令 ───

fn cmd_confirm(args: &[String]) {
    let input_file = match get_arg(args, "--input") {
        Some(f) => f,
        None => {
            eprintln!("错误: 缺少 --input 参数");
            process::exit(1);
        }
    };
    let design_type = match get_arg(args, "--design-type") {
        Some(d) => d,
        None => {
            eprintln!("错误: 缺少 --design-type 参数");
            process::exit(1);
        }
    };
    let rules_dir = get_arg(args, "--rules-dir").unwrap_or("rules");
    let year = get_arg(args, "--year").unwrap_or("2024");

    // 加载候选文件（用户已编辑，标记了 confirmed: true）
    let json_text = read_file_or_exit(input_file, "候选文件");
    let output: DiscoverOutput = match serde_json::from_str(&json_text) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("错误: 候选文件 JSON 解析失败: {}", e);
            process::exit(14);
        }
    };

    // 筛选已确认的规则
    let confirmed_rules: Vec<SpecRule> = output
        .candidates
        .iter()
        .filter(|c| c.confirmed)
        .map(|c| c.rule.clone())
        .collect();

    if confirmed_rules.is_empty() {
        eprintln!("错误: 没有标记为 confirmed: true 的审查点");
        eprintln!("提示: 请编辑 {} 并将需要确认的规则的 confirmed 设为 true", input_file);
        process::exit(15);
    }

    eprintln!("已确认 {} 条审查点", confirmed_rules.len());

    // 创建发现引擎并执行沉淀
    let mut discovery = match DiscoveryEngine::from_rules_dir(Path::new(rules_dir)) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("错误: 无法加载规则库: {}", e);
            process::exit(7);
        }
    };

    match discovery.sediment(&design_type, &confirmed_rules, year) {
        Ok(files) => {
            eprintln!("已沉淀 {} 个审查点文件:", files.len());
            for f in &files {
                eprintln!("  {}", f.display());
            }
            eprintln!("注册表已更新: {}/registry.json", rules_dir);
            eprintln!();
            eprintln!("后续审查可直接运行:");
            eprintln!("  spec_cli check --design-type {} --params <params.json>", design_type);
        }
        Err(e) => {
            eprintln!("错误: 沉淀失败: {}", e);
            process::exit(16);
        }
    }
}

// ─── 辅助函数 ───

fn get_arg<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
}

fn read_file_or_exit(path: &str, desc: &str) -> String {
    match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("错误: 无法读取{} {}: {}", desc, path, e);
            process::exit(2);
        }
    }
}

#[derive(serde::Deserialize)]
struct InputFile {
    context: Option<DesignContext>,
    parameter_table: ParameterTable,
}

fn load_input_file(path: &str) -> InputFile {
    let json_text = read_file_or_exit(path, "参数文件");
    match serde_json::from_str(&json_text) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("错误: 参数文件 JSON 解析失败: {}", e);
            process::exit(4);
        }
    }
}

// ─── discover/confirm 用的数据结构 ───

#[derive(serde::Serialize, serde::Deserialize)]
struct DiscoverOutput {
    design_type: String,
    candidates: Vec<DiscoverCandidate>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct DiscoverCandidate {
    rule: SpecRule,
    match_status: spec_core::CandidateMatchStatus,
    reason: String,
    /// 用户编辑此字段为 true 表示确认该审查点
    confirmed: bool,
}
