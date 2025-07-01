use crate::entities::vote::Model as VoteModel;
use chrono::Timelike;
use chrono_tz::Asia::Tokyo;
use plotters::prelude::*;
use plotters::style::RGBColor;
use std::collections::HashMap;

pub struct ChartService;

impl ChartService {
    /// 投票データから時系列グラフを生成（時間ベース）
    pub async fn generate_vote_timeline_chart(
        votes: Vec<VoteModel>,
        output_path: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 12時から24時まで1分間隔の時間ラベルを準備
        let mut time_labels: Vec<String> = Vec::new();
        for hour in 14..20 {
            for minute in 0..60 {
                time_labels.push(format!("{:02}:{:02}", hour, minute));
            }
        }

        // 時間ごとに投票数を集計
        let mut hour_counts: HashMap<String, HashMap<String, u32>> = HashMap::new();

        for vote in votes {
            // 日本時間に変換
            let jst_dt = vote.updated_at.with_timezone(&Tokyo);
            let hour = jst_dt.hour();
            let minute = jst_dt.minute();

            // 12時から24時の範囲のみ処理
            if hour >= 14 && hour < 24 {
                // 1分単位で集計
                let time_label = format!("{:02}:{:02}", hour, minute);
                let action_counts = hour_counts.entry(time_label).or_insert_with(HashMap::new);
                *action_counts.entry(vote.action).or_insert(0) += 1;
            }
        }

        // 全ての時間ラベルに対してデータが存在するように初期化
        for hour_label in &time_labels {
            hour_counts
                .entry(hour_label.clone())
                .or_insert_with(HashMap::new);
        }

        // 画像サイズと出力設定
        let root = BitMapBackend::new(output_path, (1200, 600)).into_drawing_area();
        root.fill(&WHITE)?;

        // エリアを分割（上：グラフエリア、下：ラベルエリア）
        let (upper, lower) = root.split_vertically((94).percent());

        let mut chart = ChartBuilder::on(&upper)
            .caption("ケバブ屋", ("Noto Sans CJK JP, Liberation Sans, Arial, sans-serif", 40).into_font().color(&BLACK))
            .margin(20)
            .x_label_area_size(20)
            .y_label_area_size(60)
            .build_cartesian_2d(
                0f32..time_labels.len() as f32,
                0f32..20f32, // Y軸を15まで、より適切な目盛り間隔に
            )?;

        chart
            .configure_mesh()
            .y_desc("累積投票数")
            .y_max_light_lines(5)
            .y_label_formatter(&|y| format!("{}", *y as i32)) // 整数表示
            .y_labels(5)
            .label_style(("Noto Sans CJK JP, Liberation Sans, Arial, sans-serif", 15).into_font().color(&BLACK))
            .axis_desc_style(("Noto Sans CJK JP, Liberation Sans, Arial, sans-serif", 20).into_font().color(&BLACK))
            .x_labels(0)
            .draw()?;

        // 手動でX軸ラベルと縦線を描画（30分間隔）
        for hour in 14..20 {
            for minute in [0, 30] {
                let time_str = format!("{:02}:{:02}", hour, minute);
                if let Some(index) = time_labels.iter().position(|x| x == &time_str) {
                    let x_pos = index as f32;

                    // ラベルエリアに描画
                    let chart_width = 1200.0 - 20.0 * 2.0 - 60.0; // 全体幅 - マージン - Y軸ラベルエリア
                    let x_pixel = 60.0 + (x_pos / time_labels.len() as f32) * chart_width;

                    lower.draw(&Text::new(
                        time_str,
                        (x_pixel as i32, 0),
                        ("Noto Sans CJK JP, Liberation Sans, Arial, sans-serif", 14).into_font().color(&BLACK),
                    ))?;

                    // 縦のグリッド線を描画
                    chart.draw_series(std::iter::once(PathElement::new(
                        vec![(x_pos, 0.0), (x_pos, 20.0)],
                        RGBColor(128, 128, 128).mix(0.3).stroke_width(1),
                    )))?;
                }
            }
        }

        // アクションごとに累積折れ線グラフを描画
        let actions = ["found", "not_found", "sold_out"];
        let colors = [&GREEN, &BLUE, &RED];
        let labels = ["営業してる", "いない", "売り切れた"];

        for (action_idx, action) in actions.iter().enumerate() {
            let mut cumulative_count = 0u32;
            let mut data_with_changes: Vec<(f32, f32)> = Vec::new();
            let data: Vec<(f32, f32)> = time_labels
                .iter()
                .enumerate()
                .map(|(hour_idx, hour_label)| {
                    let count = hour_counts
                        .get(hour_label)
                        .and_then(|counts| counts.get(*action))
                        .unwrap_or(&0);
                    let prev_cumulative = cumulative_count;
                    cumulative_count += count;

                    // 値が変わった時のみdata_with_changesに追加
                    if *count > 0 {
                        data_with_changes.push((hour_idx as f32, cumulative_count as f32));
                    }

                    (hour_idx as f32, cumulative_count as f32)
                })
                .collect();

            // 折れ線グラフを描画
            chart
                .draw_series(LineSeries::new(data.clone(), colors[action_idx]))?
                .label(labels[action_idx])
                .legend(move |(x, y)| {
                    PathElement::new(vec![(x, y), (x + 10, y)], colors[action_idx])
                });

            // 値が変わったポイントのみマーク
            chart.draw_series(
                data_with_changes
                    .iter()
                    .map(|(x, y)| Circle::new((*x, *y), 3, colors[action_idx].filled())),
            )?;
        }

        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .label_font(("Noto Sans CJK JP, Liberation Sans, Arial, sans-serif", 15).into_font().color(&BLACK))
            .draw()?;

        root.present()?;
        Ok(())
    }
}
