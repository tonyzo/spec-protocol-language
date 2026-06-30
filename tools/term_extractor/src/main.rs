//! CLI入口
//!
//! term_extractor - 标准术语提取与参数映射工具

use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod models;
mod docs;
mod rules;
mod extract;
mod diff;
mod validate;
mod ocr;
mod confidence;
mod graph;
mod batch;
mod conflicts;

fn main() -> anyhow::Result<()> {
    run()
}

/// 标准术语提取与参数映射工具
#[derive(Parser)]
#[command(name = "term_extractor")]
#[command(about = "将工程标准(PDF/Word/Markdown)整理为SPEC Protocol可用的术语表和参数映射", long_about = None)]
pub struct Cli {
    /// 启用详细日志
    #[arg(short, long)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 从标准原文提取术语
    Extract {
        /// 输入文件路径(PDF/Word/Markdown)
        #[arg(short, long)]
        input: PathBuf,

        /// 输出目录
        #[arg(short, long)]
        output: PathBuf,

        /// 标准号(如GB150, NB47012)
        #[arg(short, long)]
        standard: String,

        /// 标准版本(如2024, 2023)
        #[arg(short, long, default_value = "2024")]
        version: String,

        /// 标准分册(用逗号分隔,如"GB/T 150.1-2024 通用要求")
        #[arg(long)]
        parts: Option<String>,
    },

    /// 从param_mapping.json生成术语定义表Markdown文档
    GenerateDocs {
        /// 输入param_mapping.json
        #[arg(short, long)]
        mapping: PathBuf,

        /// 输出Markdown文件路径
        #[arg(short, long)]
        output: PathBuf,
    },

    /// 从param_mapping.json生成规则YAML骨架
    GenerateRules {
        /// 输入param_mapping.json
        #[arg(short, long)]
        mapping: PathBuf,

        /// 规则模板目录
        #[arg(short, long)]
        template: PathBuf,

        /// 输出目录
        #[arg(short, long)]
        output: PathBuf,
    },

    /// 比对两个版本的参数映射差异
    Diff {
        /// 旧版本param_mapping.json
        #[arg(long)]
        old: PathBuf,

        /// 新版本param_mapping.json
        #[arg(long)]
        new: PathBuf,

        /// 输出差异报告
        #[arg(short, long)]
        output: PathBuf,
    },

    /// 校验参数映射的一致性
    Validate {
        /// 规则库目录
        #[arg(short, long, num_args = 1..)]
        rules: Vec<PathBuf>,

        /// 生成一致性报告
        #[arg(long)]
        report: Option<PathBuf>,

        /// 仅检查,不生成报告
        #[arg(short, long)]
        check_only: bool,
    },

    /// OCR预处理 — 将PDF/Word转换为结构化文本
    Ocr {
        /// 输入文件路径(PDF/Word)
        #[arg(short, long)]
        input: PathBuf,

        /// 输出Markdown文件路径
        #[arg(short, long)]
        output: PathBuf,

        /// 输出置信度报告
        #[arg(long)]
        confidence: bool,
    },

    /// 批量提取 — 从配置文件批量处理多个标准
    BatchExtract {
        /// 批量处理配置文件(batch_config.json)
        #[arg(long)]
        config: PathBuf,

        /// 输出根目录
        #[arg(short, long)]
        output: PathBuf,
    },

    /// 跨标准冲突检测
    DetectConflicts {
        /// 标准目录列表(每个目录下需有param_mapping.json)
        #[arg(long, num_args = 1..)]
        standards: Vec<PathBuf>,

        /// 输出冲突报告路径
        #[arg(short, long)]
        output: PathBuf,
    },
}

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // 初始化日志
    let log_level = if cli.verbose {
        "term_extractor=debug,info"
    } else {
        "term_extractor=info"
    };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level)).init();

    match cli.command {
        Commands::Extract {
            input,
            output,
            standard,
            version,
            parts,
        } => {
            let parts = parts
                .map(|p| p.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();
            cmd_extract(&input, &output, &standard, &version, parts)?;
        }
        Commands::GenerateDocs { mapping, output } => {
            cmd_generate_docs(&mapping, &output)?;
        }
        Commands::GenerateRules {
            mapping,
            template,
            output,
        } => {
            cmd_generate_rules(&mapping, &template, &output)?;
        }
        Commands::Diff { old, new, output } => {
            cmd_diff(&old, &new, &output)?;
        }
        Commands::Validate {
            rules,
            report,
            check_only,
        } => {
            cmd_validate(rules.clone(), report.as_ref(), check_only)?;
        }
        Commands::Ocr {
            input,
            output,
            confidence,
        } => {
            cmd_ocr(&input, &output, confidence)?;
        }
        Commands::BatchExtract { config, output } => {
            cmd_batch_extract(&config, &output)?;
        }
        Commands::DetectConflicts { standards, output } => {
            cmd_detect_conflicts(&standards, &output)?;
        }
    }

    Ok(())
}

// 命令实现(占位,后续逐步实现)
fn cmd_extract(
    input: &PathBuf,
    output: &PathBuf,
    standard: &str,
    version: &str,
    parts: Vec<String>,
) -> anyhow::Result<()> {
    log::info!("提取术语功能");
    log::info!("输入: {:?}", input);
    log::info!("输出: {:?}", output);
    log::info!("标准: {} {}", standard, version);
    
    // 创建输出目录
    std::fs::create_dir_all(output)?;
    
    // 提取术语
    extract::extract_terms(input, output, standard, version, parts)?;
    
    Ok(())
}

fn cmd_generate_docs(mapping: &PathBuf, output: &PathBuf) -> anyhow::Result<()> {
    log::info!("生成术语文档: {:?} -> {:?}", mapping, output);
    
    // 1. 读取param_mapping.json
    let content = std::fs::read_to_string(mapping)?;
    let param_mapping: models::ParamMapping = serde_json::from_str(&content)?;
    
    // 2. 生成Markdown文档
    docs::generate_docs(&param_mapping, output)?;
    
    log::info!("文档生成完成!");
    Ok(())
}

fn cmd_generate_rules(mapping: &PathBuf, _template: &PathBuf, output: &PathBuf) -> anyhow::Result<()> {
    log::info!("生成规则YAML: {:?} -> {:?}", mapping, output);
    
    // 1. 读取param_mapping.json
    let content = std::fs::read_to_string(mapping)?;
    let param_mapping: models::ParamMapping = serde_json::from_str(&content)?;
    
    // 2. 生成规则YAML骨架
    rules::generate_rules(&param_mapping, output)?;
    
    log::info!("规则YAML骨架生成完成!");
    Ok(())
}

fn cmd_diff(old: &PathBuf, new: &PathBuf, output: &PathBuf) -> anyhow::Result<()> {
    log::info!("比对差异: {:?} vs {:?} -> {:?}", old, new, output);
    
    // 比对
    let result = diff::compare_mappings(old, new, Some(output))?;
    
    log::info!("差异报告已生成: {:?}", output);
    Ok(())
}

fn cmd_validate(rules: Vec<PathBuf>, report: Option<&PathBuf>, check_only: bool) -> anyhow::Result<()> {
    log::info!("校验一致性: {:?}", rules);
    
    // 分离mappings和rules_dirs
    let mut mappings = Vec::new();
    let mut rules_dirs = Vec::new();
    
    for rule in &rules {
        if rule.is_file() {
            mappings.push(rule.as_path());
        } else if rule.is_dir() {
            rules_dirs.push(rule.as_path());
        }
    }
    
    // 校验
    let result = validate::validate_mappings(
        &mappings,
        &rules_dirs,
        report.map(|v| v.as_path()),
        check_only,
    )?;
    
    log::info!("校验完成! 发现 {} 个问题", result.issues.len());
    Ok(())
}

/// OCR预处理命令
fn cmd_ocr(input: &PathBuf, output: &PathBuf, output_confidence: bool) -> anyhow::Result<()> {
    log::info!("OCR预处理: {:?} -> {:?}", input, output);

    let config = ocr::OcrConfig {
        input: input.to_string_lossy().to_string(),
        output: output.to_string_lossy().to_string(),
        output_confidence,
        batch_size: 1,
    };

    let processor = ocr::OcrProcessor::new(config);

    // 根据文件扩展名选择处理方式
    let extension = input.extension().and_then(|s| s.to_str()).unwrap_or("");
    match extension {
        "pdf" => {
            let result = processor.process_pdf()?;
            log::info!("OCR完成! 置信度: {:.2}", result.confidence);
        }
        "docx" | "doc" => {
            let result = processor.process_word()?;
            log::info!("OCR完成! 置信度: {:.2}", result.confidence);
        }
        _ => {
            return Err(anyhow::anyhow!("不支持的文件格式: {}", extension));
        }
    }

    log::info!("OCR预处理完成!");
    Ok(())
}

/// 批量提取命令
fn cmd_batch_extract(config: &PathBuf, output: &PathBuf) -> anyhow::Result<()> {
    log::info!("批量提取: config={:?}, output={:?}", config, output);

    // 创建输出根目录
    std::fs::create_dir_all(output)?;

    // 加载配置并执行批量处理
    let processor = batch::BatchProcessor::from_config_file(config, output.clone())?;
    let result = processor.execute()?;

    // 生成批量处理报告
    let report_path = output.join("batch_report.md");
    batch::BatchProcessor::generate_report(&result, &report_path)?;

    log::info!("批量提取完成! 成功: {}/{}", result.success_count, result.total_standards);
    Ok(())
}

/// 跨标准冲突检测命令
fn cmd_detect_conflicts(standards: &[PathBuf], output: &PathBuf) -> anyhow::Result<()> {
    log::info!("冲突检测: {:?} -> {:?}", standards, output);

    // 加载每个标准目录下的 param_mapping.json
    let mut mappings: Vec<(String, models::ParamMapping)> = Vec::new();

    for dir in standards {
        let mapping_path = dir.join("param_mapping.json");
        if !mapping_path.exists() {
            log::warn!("目录下未找到 param_mapping.json: {:?}", dir);
            continue;
        }

        let content = std::fs::read_to_string(&mapping_path)?;
        let mapping: models::ParamMapping = serde_json::from_str(&content)?;
        let standard_name = mapping.standard.clone();

        log::info!("加载标准: {} ({} 个术语)", standard_name, mapping.terms.len());
        mappings.push((standard_name, mapping));
    }

    if mappings.len() < 2 {
        return Err(anyhow::anyhow!("冲突检测需要至少2个标准,当前只有 {} 个", mappings.len()));
    }

    // 构造冲突检测输入
    let mapping_refs: Vec<(&str, &models::ParamMapping)> = mappings
        .iter()
        .map(|(name, m)| (name.as_str(), m))
        .collect();

    // 运行冲突检测
    let detector = conflicts::ConflictDetector::new(&mapping_refs);
    let result = detector.detect_all();

    // 生成冲突报告
    conflicts::report::generate_conflict_report(&result, output)?;

    log::info!(
        "冲突检测完成! 发现 {} 处冲突 (术语:{}, 参数:{}, 规则:{}, 单位:{})",
        result.total_conflicts,
        result.terminology_conflicts,
        result.parameter_conflicts,
        result.rule_conflicts,
        result.unit_conflicts
    );
    Ok(())
}
