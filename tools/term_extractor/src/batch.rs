//! 批量处理模块
//!
//! 支持多标准并行处理和多文件批量处理

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

/// 批量处理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    /// 标准列表
    pub standards: Vec<StandardConfig>,
    /// 处理选项
    pub options: BatchOptions,
}

/// 标准配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardConfig {
    /// 标准名称
    pub name: String,
    /// 标准版本
    pub version: String,
    /// 文件列表
    pub files: Vec<String>,
    /// 输出目录
    pub output_dir: Option<String>,
}

/// 批量处理选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOptions {
    /// 是否启用并行处理
    pub parallel: bool,
    /// 是否启用OCR
    pub ocr_enabled: bool,
    /// 是否启用冲突检测
    pub conflict_detection: bool,
    /// 最大线程数
    pub max_threads: usize,
}

impl Default for BatchOptions {
    fn default() -> Self {
        BatchOptions {
            parallel: true,
            ocr_enabled: true,
            conflict_detection: true,
            max_threads: 4,
        }
    }
}

/// 批量处理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    /// 总标准数
    pub total_standards: usize,
    /// 成功数
    pub success_count: usize,
    /// 失败数
    pub failure_count: usize,
    /// 标准结果列表
    pub standard_results: Vec<StandardResult>,
    /// 总耗时(秒)
    pub total_time_secs: f64,
}

/// 标准处理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardResult {
    /// 标准名称
    pub standard_name: String,
    /// 版本
    pub version: String,
    /// 状态(success/failure)
    pub status: String,
    /// 处理文件数
    pub file_count: usize,
    /// 提取术语数
    pub term_count: usize,
    /// 错误信息(如果有)
    pub error_message: Option<String>,
    /// 处理时间(秒)
    pub time_secs: f64,
    /// 输出目录路径(用于后续冲突检测)
    pub output_path: Option<PathBuf>,
}

/// 批量处理器
pub struct BatchProcessor {
    config: BatchConfig,
    results: Arc<Mutex<Vec<StandardResult>>>,
    output_root: PathBuf,
}

impl BatchProcessor {
    /// 创建新的批量处理器
    pub fn new(config: BatchConfig, output_root: PathBuf) -> Self {
        BatchProcessor {
            config,
            results: Arc::new(Mutex::new(Vec::new())),
            output_root,
        }
    }

    /// 从配置文件加载
    pub fn from_config_file(config_path: &Path, output_root: PathBuf) -> anyhow::Result<Self> {
        let content = fs::read_to_string(config_path)?;
        let config: BatchConfig = serde_json::from_str(&content)?;
        Ok(BatchProcessor::new(config, output_root))
    }

    /// 执行批量处理
    pub fn execute(&self) -> anyhow::Result<BatchResult> {
        log::info!("开始批量处理...");
        log::info!("标准数: {}", self.config.standards.len());
        log::info!("并行处理: {}", self.config.options.parallel);
        log::info!("OCR启用: {}", self.config.options.ocr_enabled);
        log::info!("冲突检测: {}", self.config.options.conflict_detection);

        let start_time = std::time::Instant::now();

        if self.config.options.parallel {
            self.execute_parallel()?;
        } else {
            self.execute_sequential()?;
        }

        let total_time = start_time.elapsed().as_secs_f64();

        // 统计结果
        let results = self.results.lock().unwrap();
        let success_count = results.iter().filter(|r| r.status == "success").count();
        let failure_count = results.len() - success_count;

        // 自动冲突检测(当配置启用且至少2个标准成功)
        if self.config.options.conflict_detection && success_count >= 2 {
            log::info!("开始跨标准冲突检测...");
            let conflict_report_path = self.output_root.join("conflict_report.md");
            match self.run_conflict_detection(&results, &conflict_report_path) {
                Ok(conflict_count) => {
                    log::info!("冲突检测完成! 发现 {} 处冲突", conflict_count);
                }
                Err(e) => {
                    log::warn!("冲突检测失败: {}", e);
                }
            }
        }

        let batch_result = BatchResult {
            total_standards: self.config.standards.len(),
            success_count,
            failure_count,
            standard_results: results.clone(),
            total_time_secs: total_time,
        };

        log::info!("批量处理完成!");
        log::info!("成功: {}/{}", batch_result.success_count, batch_result.total_standards);
        log::info!("失败: {}/{}", batch_result.failure_count, batch_result.total_standards);
        log::info!("总耗时: {:.2}秒", batch_result.total_time_secs);

        Ok(batch_result)
    }

    /// 运行冲突检测
    fn run_conflict_detection(
        &self,
        results: &[StandardResult],
        output: &Path,
    ) -> anyhow::Result<usize> {
        // 收集所有成功标准的 param_mapping.json
        let mut mappings: Vec<(String, crate::models::ParamMapping)> = Vec::new();

        for result in results {
            if result.status != "success" {
                continue;
            }
            if let Some(ref output_path) = result.output_path {
                let mapping_path = output_path.join("param_mapping.json");
                if mapping_path.exists() {
                    let content = fs::read_to_string(&mapping_path)?;
                    let mapping: crate::models::ParamMapping = serde_json::from_str(&content)?;
                    let standard_name = mapping.standard.clone();
                    mappings.push((standard_name, mapping));
                }
            }
        }

        if mappings.len() < 2 {
            log::warn!("冲突检测需要至少2个标准,当前只有 {} 个", mappings.len());
            return Ok(0);
        }

        // 构造冲突检测输入
        let mapping_refs: Vec<(&str, &crate::models::ParamMapping)> = mappings
            .iter()
            .map(|(name, m)| (name.as_str(), m))
            .collect();

        // 运行冲突检测
        let detector = crate::conflicts::ConflictDetector::new(&mapping_refs);
        let conflict_result = detector.detect_all();

        // 生成冲突报告
        crate::conflicts::report::generate_conflict_report(&conflict_result, output)?;

        Ok(conflict_result.total_conflicts)
    }

    /// 顺序执行
    fn execute_sequential(&self) -> anyhow::Result<()> {
        for standard in &self.config.standards {
            let result = self.process_standard(standard, &self.output_root)?;
            self.results.lock().unwrap().push(result);
        }
        Ok(())
    }

    /// 并行执行
    fn execute_parallel(&self) -> anyhow::Result<()> {
        let mut handles = Vec::new();

        for standard in &self.config.standards {
            let standard_clone = standard.clone();
            let results_clone = Arc::clone(&self.results);
            let output_root_clone = self.output_root.clone();
            // 修复: 传入用户配置的 options 而非 default
            let options_clone = self.config.options.clone();

            let handle = thread::spawn(move || {
                let processor = BatchProcessor {
                    config: BatchConfig {
                        standards: vec![standard_clone.clone()],
                        options: options_clone,
                    },
                    results: results_clone,
                    output_root: output_root_clone,
                };

                match processor.process_standard(&standard_clone, &processor.output_root) {
                    Ok(result) => {
                        processor.results.lock().unwrap().push(result);
                    }
                    Err(e) => {
                        log::error!("处理标准 {} 失败: {}", standard_clone.name, e);
                        // 记录失败结果
                        let fail_result = StandardResult {
                            standard_name: standard_clone.name.clone(),
                            version: standard_clone.version.clone(),
                            status: "failure".to_string(),
                            file_count: 0,
                            term_count: 0,
                            error_message: Some(e.to_string()),
                            time_secs: 0.0,
                            output_path: None,
                        };
                        processor.results.lock().unwrap().push(fail_result);
                    }
                }
            });

            handles.push(handle);
        }

        // 等待所有线程完成
        for handle in handles {
            handle.join().map_err(|_| anyhow::anyhow!("线程执行失败"))?;
        }

        Ok(())
    }

    /// 处理单个标准
    fn process_standard(&self, standard: &StandardConfig, output_root: &Path) -> anyhow::Result<StandardResult> {
        log::info!("处理标准: {} {}", standard.name, standard.version);

        let start_time = std::time::Instant::now();
        let mut term_count = 0;

        // 创建标准输出目录
        let standard_output_dir = output_root.join(&standard.name);
        fs::create_dir_all(&standard_output_dir)?;

        // 处理每个文件
        for file in &standard.files {
            let file_path = Path::new(file);
            if !file_path.exists() {
                log::warn!("文件不存在: {}", file);
                continue;
            }

            let extension = file_path.extension().and_then(|s| s.to_str()).unwrap_or("");

            match extension {
                "pdf" | "docx" | "doc" => {
                    log::info!("处理 {}: {}", extension.to_uppercase(), file);

                    // 步骤1: OCR预处理
                    if self.config.options.ocr_enabled {
                        let ocr_output = standard_output_dir.join(
                            format!("{}.md", file_path.file_stem().and_then(|s| s.to_str()).unwrap_or("output"))
                        );

                        let ocr_config = crate::ocr::OcrConfig {
                            input: file.clone(),
                            output: ocr_output.to_string_lossy().to_string(),
                            output_confidence: true,
                            batch_size: 1,
                        };

                        let ocr_processor = crate::ocr::OcrProcessor::new(ocr_config);

                        // TODO: 集成真实Unlimited OCR — 当前返回模拟数据
                        let ocr_result = if extension == "pdf" {
                            ocr_processor.process_pdf()?
                        } else {
                            ocr_processor.process_word()?
                        };

                        log::info!("  OCR完成, 置信度: {:.2}", ocr_result.confidence);

                        // 步骤2: 术语提取
                        crate::extract::extract_terms(
                            &ocr_output,
                            &standard_output_dir,
                            &standard.name,
                            &standard.version,
                            vec![],
                        )?;
                    }
                }
                "md" => {
                    log::info!("处理Markdown: {}", file);

                    // 直接术语提取(无需OCR)
                    crate::extract::extract_terms(
                        file_path,
                        &standard_output_dir,
                        &standard.name,
                        &standard.version,
                        vec![],
                    )?;
                }
                _ => {
                    log::warn!("不支持的文件格式: {}", file);
                }
            }
        }

        // 步骤3: 加载提取结果
        let mapping_path = standard_output_dir.join("param_mapping.json");
        let mapping = if mapping_path.exists() {
            let content = fs::read_to_string(&mapping_path)?;
            let mapping: crate::models::ParamMapping = serde_json::from_str(&content)?;
            term_count = mapping.terms.len();
            Some(mapping)
        } else {
            None
        };

        // 步骤4: 三层置信度计算
        if let Some(ref mapping) = mapping {
            // TODO: 集成真实LLM置信度 — 当前使用默认值
            let confidence = crate::confidence::TripleConfidence::from_ocr(0.95);
            log::info!("  置信度: {:.2} (等级: {})",
                confidence.extraction_confidence,
                confidence.get_confidence_level());

            if confidence.needs_manual_review() {
                log::warn!("  ⚠️ 综合置信度低于0.7,建议人工确认");
            }

            // 步骤5: 知识图谱构建
            let graph = crate::graph::KnowledgeGraphBuilder::build_from_param_mapping(mapping);
            let graph_path = standard_output_dir.join("knowledge_graph.json");
            fs::write(&graph_path, graph.to_json())?;
            log::info!("  知识图谱: {} 个节点", graph.nodes.len());
        }

        let time_secs = start_time.elapsed().as_secs_f64();

        let result = StandardResult {
            standard_name: standard.name.clone(),
            version: standard.version.clone(),
            status: "success".to_string(),
            file_count: standard.files.len(),
            term_count,
            error_message: None,
            time_secs,
            output_path: Some(standard_output_dir),
        };

        log::info!("标准 {} 处理完成, 提取 {} 个术语, 耗时 {:.2}秒",
            result.standard_name, result.term_count, result.time_secs);

        Ok(result)
    }

    /// 生成批量处理报告
    pub fn generate_report(result: &BatchResult, output: &Path) -> anyhow::Result<()> {
        let mut content = String::new();

        content.push_str("# 批量处理报告\n\n");
        content.push_str(&format!("**生成时间**: {}\n\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));

        // 统计
        content.push_str("## 📊 处理统计\n\n");
        content.push_str(&format!("- **总标准数**: {}\n", result.total_standards));
        content.push_str(&format!("- **成功数**: {}\n", result.success_count));
        content.push_str(&format!("- **失败数**: {}\n", result.failure_count));
        content.push_str(&format!("- **总耗时**: {:.2}秒\n\n", result.total_time_secs));

        // 成功率
        let success_rate = if result.total_standards > 0 {
            (result.success_count as f64 / result.total_standards as f64) * 100.0
        } else {
            0.0
        };
        content.push_str(&format!("**成功率**: {:.1}%\n\n", success_rate));

        // 详细结果
        content.push_str("## 📝 详细结果\n\n");
        for standard_result in &result.standard_results {
            content.push_str(&format!("### {}\n\n", standard_result.standard_name));
            content.push_str(&format!("- **版本**: {}\n", standard_result.version));
            content.push_str(&format!("- **状态**: {}\n", standard_result.status));
            content.push_str(&format!("- **文件数**: {}\n", standard_result.file_count));
            content.push_str(&format!("- **术语数**: {}\n", standard_result.term_count));
            content.push_str(&format!("- **耗时**: {:.2}秒\n", standard_result.time_secs));

            if let Some(error) = &standard_result.error_message {
                content.push_str(&format!("- **错误**: {}\n", error));
            }
            content.push_str("\n");
        }

        fs::write(output, content)?;
        log::info!("批量处理报告已生成: {:?}", output);

        Ok(())
    }
}

/// 批量处理配置构建器
pub struct BatchConfigBuilder {
    config: BatchConfig,
}

impl BatchConfigBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        BatchConfigBuilder {
            config: BatchConfig {
                standards: Vec::new(),
                options: BatchOptions::default(),
            },
        }
    }

    /// 添加标准
    pub fn add_standard(mut self, standard: StandardConfig) -> Self {
        self.config.standards.push(standard);
        self
    }

    /// 设置并行处理
    pub fn parallel(mut self, enabled: bool) -> Self {
        self.config.options.parallel = enabled;
        self
    }

    /// 设置OCR
    pub fn ocr(mut self, enabled: bool) -> Self {
        self.config.options.ocr_enabled = enabled;
        self
    }

    /// 设置冲突检测
    pub fn conflict_detection(mut self, enabled: bool) -> Self {
        self.config.options.conflict_detection = enabled;
        self
    }

    /// 设置最大线程数
    pub fn max_threads(mut self, threads: usize) -> Self {
        self.config.options.max_threads = threads;
        self
    }

    /// 构建配置
    pub fn build(self) -> BatchConfig {
        self.config
    }
}
