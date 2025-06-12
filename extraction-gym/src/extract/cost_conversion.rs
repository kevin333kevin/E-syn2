
/// egraph->eqn
use serde::ser::StdError;
use tempfile::NamedTempFile;
// use crate::extract::circuit_conversion::process_circuit_conversion_extraction_json;
use crate::extract::lib::Abc;
use std::io::Write;
use std::error::Error;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use crate::ExtractionResult;
use crate::extract::circuit_conversion::process_circuit_conversion;
use crate::extract::circuit_conversion::build_eqns_pipeline;
use rayon::iter::IntoParallelIterator;
use rand::distributions::WeightedIndex;
 use rayon::iter::ParallelIterator;
 use rayon::prelude::*;
 use std::sync::atomic::Ordering;
 use crate::faster_bottom_up::VERILOG_COUNTER;   // 路径按你的工程结构修改
 use std::process::Command;
 use std::process::Stdio;
 use rayon::iter::IndexedParallelIterator;
///          cost 
pub fn call_abc_external(
    eqn: &str,
) -> Result<(f32, String), Box<dyn Error + Send + Sync>> {
    /* 1. 写临时 .eqn 文件 --------------------------------------------------- */
    let mut file = NamedTempFile::new()?;
    file.write_all(eqn.as_bytes())?;
    let path = file.path();

    /* 1-b. 取得唯一的 .v 文件名 --------------------------------------------- */
    let idx  = VERILOG_COUNTER.fetch_add(1, Ordering::SeqCst);
    let vout = format!("tmp_{idx}.v");

    /* 2. ABC 脚本 ----------------------------------------------------------- */
    let script = format!(
        "read_eqn {p}; read_lib ../abc/asap7_clean.lib; \
         strash;dch; map; topo; upsize; dnsize; stime; write {v}",
        p = path.display(),
        v = vout
    );

    /* 3. 运行 abc ----------------------------------------------------------- */
    let out = Command::new("abc")
        .args(["-c", &script])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    if !out.status.success() {
        eprintln!(
            "abc exited with {}\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}",
            out.status
        );
        return Err(format!("abc exited with {}", out.status).into());
    }

    /* 4. 解析延迟 ----------------------------------------------------------- */
    match parse_delay_external(&stdout) {
        Some(ps) => Ok((ps, vout)),          // 把 v 文件名一并返回
        None     => Err("failed to parse delay".into()),
    }
}

pub fn call_abc(eqn_content: &str) -> Result<f32, Box<dyn std::error::Error>> {
    let mut temp_file = NamedTempFile::new()?;
    temp_file.write_all(eqn_content.as_bytes())?;
    let temp_path = temp_file.path().to_str().unwrap();

    let mut abc = Abc::new();

    //println!("Reading equation file...");
    abc.execute_command(&format!("read_eqn {}", temp_path));
    //println!("Reading library...");
    abc.execute_command(&format!("read_lib ../abc/asap7_clean.lib"));
    //println!("Performing structural hashing...");
    abc.execute_command(&format!("strash"));
    //println!("Performing dump the edgelist...");
   
   
   //~~~~~~~~~~~test maping

   //abc.execute_command(&format!("&get; &edgelist  -F src/extract/tmp/opt_1.el -f src/extract/tmp/opt-feats.csv -c src/extract/tmp/opt_1.json; &put"));
    //abc.execute_command(&format!("dch"));
   
   
   
    //println!("Performing technology mapping...");
    abc.execute_command(&format!("map"));
    //println!("Performing post-processing...(topo; gate sizing)");
    abc.execute_command(&format!("topo"));
    abc.execute_command(&format!("upsize"));
    abc.execute_command(&format!("dnsize"));

    //println!("Executing stime command...");
    let stime_output = abc.execute_command_with_output(&format!("stime -d"));

    if let Some(delay) = parse_delay(&stime_output) {
        let delay_ns = delay / 1000.0;
        //println!("Delay in nanoseconds: {} ns", delay_ns);
        Ok(delay)
    } else {
        Err("Failed to parse delay value".into())
    }
}
pub fn parse_delay(output: &str) -> Option<f32> {
    for line in output.lines() {
        if line.contains("Delay") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                return parts[1].parse::<f32>().ok();
            }
        }
    }
    None
}

pub fn parse_delay_external(text: &str) -> Option<f32> {
    for line in text.lines() {
        if let Some(pos) = line.find("Delay =") {
            // 找到 "Delay =" 后，从该位置截取字符串
            let remaining = &line[pos + "Delay =".len()..];
            // 按空格分割，取第一个数字
            if let Some(value_str) = remaining.split_whitespace().next() {
                // 尝试将数字字符串解析为 f32
                if let Ok(value) = value_str.parse::<f32>() {
                    return Some(value);
                }
            }
        }
    }
    None
}

pub fn call_abc_ml(eqn_content: &str) -> Result<(), Box<dyn Error>> {
    let mut temp_file = NamedTempFile::new()?;
    temp_file.write_all(eqn_content.as_bytes())?;
    let temp_path = temp_file.path().to_str().unwrap();

    let mut abc = Abc::new();

    // Execute commands
    abc.execute_command(&format!("read_eqn {}", temp_path));
    //abc.execute_command(&format!("read_lib ../abc/asap7_clean.lib"));
    abc.execute_command("strash");
   
    // Perform mapping and extract data
    abc.execute_command("&get; &edgelist -F src/extract/tmp/opt_1.el -f src/extract/tmp/opt-feats.csv -c src/extract/tmp/opt_1.json; &put");



    Ok(())
}


pub fn calculate_abc_cost_or_dump(
    result: &ExtractionResult,
    saturated_graph_json: &str,
    prefix_mapping_path: &str,
    dump_to_file: bool,
) -> f64 {
    let eqn_content = match process_circuit_conversion(
        result,
        saturated_graph_json,
        prefix_mapping_path,
        false,
    ) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error in circuit conversion: {}", e);
            return f64::INFINITY;
        }
    };
    // dump file mode
    if dump_to_file {
        if let Err(e) = std::fs::write("src/extract/tmp/output.eqn", &eqn_content) {
            eprintln!("Error writing to file: {}", e);
            // Handle the error appropriately
        }
        return f64::INFINITY;
    // abc cost calculation mode
    } else {
        match call_abc(&eqn_content) {
            Ok(delay) => delay as f64,
            Err(e) => {
                eprintln!("Error in ABC processing: {}", e);
                f64::INFINITY
            }
        }
    }
}

pub fn calculate_abc_costs_parallel(
    mut results: Vec<ExtractionResult>,
    saturated_graph_json: &str,
    prefix_mapping_path: &str,
    dump_to_file: bool,
    init:bool,
) -> Result<(ExtractionResult, String, f64), String> {
    // 1. 先跑四段流水线，得到每个 ExtractionResult 的 (final_json, eqn)
    let pipeline_out = build_eqns_pipeline(
        results.clone(),         // 需要 .clone()；若 ExtractionResult 不大可直接派生 Clone
        saturated_graph_json,
        prefix_mapping_path,
        false,                   // is_large
        false,                   // dump_intermediate
    )?;                          // Vec<(final_json, eqn)>

    // 2. 与原 ExtractionResult 一一对应，再并行调用 ABC
    let cost_results: Vec<Result<(ExtractionResult, String, f64), String>> =
        results
            .into_par_iter()
            .zip(pipeline_out.into_par_iter())       // 保证索引一致
            .enumerate()
            .map(|(idx, (extraction, (final_json, eqn_content)))| {
                // —— dump eqn （可选）——
                // if dump_to_file {
                //     let path = format!("src/extract/tmp/output_{idx}.eqn");
                //     std::fs::write(&path, &eqn_content)
                //         .map_err(|e| format!("File write error: {e}"))?;
                //     return Ok((extraction, final_json, f64::INFINITY));
                // }

                // —— 调 ABC —— //
                let (delay_ps, _v_file) = call_abc_external(&eqn_content)
                .map_err(|e| format!("ABC processing error: {e}"))?;
            
            Ok((extraction, final_json, delay_ps as f64))
            })
            .collect();

    // 3. 过滤失败并按 cost 排序
    let mut valid: Vec<(ExtractionResult, String, f64)> =
        cost_results.into_iter().filter_map(Result::ok).collect();

    // 打印所有有效结果的组号和延迟
    // println!("Valid results (group index and delay):");
    // for (i, (_, _, delay)) in valid.iter().enumerate() {
    //     println!("Group {}: Delay = {} ns", i, delay);
    // }
    if init{

    println!("Valid results (group index and delay):");
    for (i, (_, _, delay)) in valid.iter().enumerate() {
        println!("Group {}: Delay = {} ns", i, delay);
    }

    }

    // 按延迟排序
    valid.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

    // 选择最佳结果并打印
    if let Some(best) = valid.first() {
        println!(
            "Best result: Group 0 (sorted) with delay = {} ns",
            best.2
        );
    }

    valid.into_iter().next().ok_or_else(|| "No valid results found".to_owned())
}


pub fn evaluate_candidates_parallel(
    results: Vec<ExtractionResult>,
    sat_json: &str,
    prefix_path: &str,
) -> Result<Vec<(ExtractionResult, String, f64, String)>, String> {
    let pipeline_out = build_eqns_pipeline(
        results.clone(),
        sat_json,
        prefix_path,
        false,
        false,
    )?;

    results
        .into_par_iter()
        .zip(pipeline_out.into_par_iter())
        .map(|(extraction, (final_json, eqn))| {
            let (delay, v_file) = call_abc_external(&eqn)
                .map_err(|e| format!("ABC error: {e}"))?;
            Ok::<_, String>((extraction, final_json, delay as f64, v_file))
        })
        .collect::<Result<Vec<(ExtractionResult, String, f64, String)>, String>>() 
        .map(|mut v| {
            v.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());
            v
        })
}