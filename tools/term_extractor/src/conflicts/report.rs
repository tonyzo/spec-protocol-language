//! 冲突报告生成
//!
//! 参数化渲染: 一个 render_section 函数替代四个重复 if-block

use super::{ConflictResult, ConflictType};
use std::fs;
use std::path::Path;

/// 生成冲突检测报告
pub fn generate_conflict_report(
    result: &ConflictResult,
    output: &Path,
) -> anyhow::Result<()> {
    let mut content = String::new();

    content.push_str("# 跨标准冲突检测报告\n\n");
    content.push_str(&format!(
        "**生成时间**: {}\n\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    ));

    // 统计
    content.push_str("## 📊 冲突统计\n\n");
    content.push_str(&format!("- **总冲突数**: {}\n", result.total_conflicts));
    content.push_str(&format!("- **术语冲突**: {}\n", result.terminology_conflicts));
    content.push_str(&format!("- **参数冲突**: {}\n", result.parameter_conflicts));
    content.push_str(&format!("- **规则冲突**: {}\n", result.rule_conflicts));
    content.push_str(&format!("- **单位冲突**: {}\n\n", result.unit_conflicts));

    if result.total_conflicts == 0 {
        content.push_str("**状态**: ✅ 无冲突,所有标准一致\n\n");
    } else {
        content.push_str(&format!(
            "**状态**: ⚠️ 发现 {} 处冲突,需要人工确认\n\n",
            result.total_conflicts
        ));
    }

    // 参数化: 一个循环替代四个重复 if-block
    let sections: &[(ConflictType, &str, &str, &str)] = &[
        (ConflictType::Terminology, "📝", "术语冲突", "同一概念在不同标准中术语不同:"),
        (ConflictType::Parameter, "🔧", "参数冲突", "同一参数在不同标准中定义不同:"),
        (ConflictType::Rule, "⚙️", "规则冲突", "同一场景在不同标准中要求不同:"),
        (ConflictType::Unit, "📏", "单位冲突", "同一参数在不同标准中单位不同:"),
    ];

    for (conflict_type, emoji, title, desc) in sections {
        let count = match conflict_type {
            ConflictType::Terminology => result.terminology_conflicts,
            ConflictType::Parameter => result.parameter_conflicts,
            ConflictType::Rule => result.rule_conflicts,
            ConflictType::Unit => result.unit_conflicts,
        };
        if count > 0 {
            content.push_str(&render_section(result, conflict_type, emoji, title, desc));
        }
    }

    fs::write(output, content)?;
    log::info!("冲突检测报告已生成: {:?}", output);

    Ok(())
}

/// 渲染单个冲突类型的章节
fn render_section(
    result: &ConflictResult,
    conflict_type: &ConflictType,
    emoji: &str,
    title: &str,
    desc: &str,
) -> String {
    let mut content = String::new();

    content.push_str(&format!("## {} {}\n\n", emoji, title));
    content.push_str(&format!("{}\n\n", desc));

    for conflict in &result.conflicts {
        if &conflict.conflict_type == conflict_type {
            content.push_str(&format!("### {}\n\n", conflict.details.lines().next().unwrap_or("")));
            content.push_str(&format!("- **涉及标准**: {}\n", conflict.standards.join(", ")));
            content.push_str(&format!("- **详情**: {}\n", conflict.details));
            content.push_str(&format!("- **解决建议**: {}\n\n", conflict.resolution));
        }
    }

    content
}
