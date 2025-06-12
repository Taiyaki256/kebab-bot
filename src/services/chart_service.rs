use crate::entities::vote::Model as VoteModel;
use chrono::Datelike;
use plotters::prelude::*;
use std::collections::HashMap;

pub struct ChartService;

impl ChartService {
    /// 投票データから時系列グラフを生成
    pub async fn generate_vote_timeline_chart(
        votes: Vec<VoteModel>,
        output_path: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 日付ごとに投票数を集計
        let mut daily_counts: HashMap<String, HashMap<String, u32>> = HashMap::new();

        for vote in votes {
            let date = vote.updated_at.date_naive().format("%m/%d").to_string();
            let action_counts = daily_counts.entry(date).or_insert_with(HashMap::new);
            *action_counts.entry(vote.action).or_insert(0) += 1;
        }

        // 日付順にソート
        let mut sorted_dates: Vec<_> = daily_counts.keys().cloned().collect();
        sorted_dates.sort();

        if sorted_dates.is_empty() {
            return Err("投票データがありません".into());
        }

        // 画像サイズと出力設定
        let root = BitMapBackend::new(output_path, (800, 600)).into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption("ケバブ投票の推移", ("sans-serif", 40))
            .margin(10)
            .x_label_area_size(50)
            .y_label_area_size(50)
            .build_cartesian_2d(
                0f32..sorted_dates.len() as f32,
                0f32..20f32, // 最大投票数を想定
            )?;

        chart
            .configure_mesh()
            .x_desc("日付")
            .y_desc("投票数")
            .x_label_formatter(&|x| {
                let index = *x as usize;
                if index < sorted_dates.len() {
                    sorted_dates[index].clone()
                } else {
                    String::new()
                }
            })
            .draw()?;

        // アクションごとに線グラフを描画
        let actions = ["found", "not_found", "sold_out"];
        let colors = [&RED, &BLUE, &GREEN];
        let labels = ["営業してる", "いない", "売り切れた"];

        for (i, action) in actions.iter().enumerate() {
            let data: Vec<(f32, f32)> = sorted_dates
                .iter()
                .enumerate()
                .map(|(date_idx, date)| {
                    let count = daily_counts
                        .get(date)
                        .and_then(|counts| counts.get(*action))
                        .unwrap_or(&0);
                    (date_idx as f32, *count as f32)
                })
                .collect();

            chart
                .draw_series(LineSeries::new(data.clone(), colors[i]))?
                .label(labels[i])
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], colors[i]));

            // データポイントをマーク
            chart.draw_series(
                data.iter()
                    .map(|(x, y)| Circle::new((*x, *y), 3, colors[i].filled())),
            )?;
        }

        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()?;

        root.present()?;
        Ok(())
    }

    /// 円グラフで現在の投票結果を生成
    pub async fn generate_vote_pie_chart(
        found_count: u64,
        not_found_count: u64,
        sold_out_count: u64,
        output_path: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let total = found_count + not_found_count + sold_out_count;

        if total == 0 {
            return Err("投票データがありません".into());
        }

        let root = BitMapBackend::new(output_path, (600, 600)).into_drawing_area();
        root.fill(&WHITE)?;

        let mut chart = ChartBuilder::on(&root)
            .caption("現在の投票結果", ("sans-serif", 40))
            .margin(10)
            .build_cartesian_2d(-1.2f32..1.2f32, -1.2f32..1.2f32)?;

        let data = vec![
            ("営業してる", found_count as f32, &RED),
            ("いない", not_found_count as f32, &BLUE),
            ("売り切れた", sold_out_count as f32, &GREEN),
        ];

        let mut angle = 0.0;
        for (label, count, color) in data {
            if count > 0.0 {
                let slice_angle = (count / total as f32) * 360.0;
                let end_angle = angle + slice_angle;

                // パイスライスを描画
                let mut points = vec![(0.0, 0.0)];
                let steps = (slice_angle / 5.0).max(1.0) as i32;
                for i in 0..=steps {
                    let current_angle = angle + (slice_angle * i as f32 / steps as f32);
                    let rad = current_angle.to_radians();
                    points.push((rad.cos(), rad.sin()));
                }

                chart.draw_series(std::iter::once(Polygon::new(points, color.filled())))?;

                // ラベルを描画
                let label_angle = (angle + slice_angle / 2.0).to_radians();
                let label_x = label_angle.cos() * 0.7;
                let label_y = label_angle.sin() * 0.7;

                chart.draw_series(std::iter::once(Text::new(
                    format!("{}\n{}票", label, count),
                    (label_x, label_y),
                    ("sans-serif", 15),
                )))?;

                angle = end_angle;
            }
        }

        root.present()?;
        Ok(())
    }
}
