/// Plot raw implied volatility smiles from Kaggle SPY option data.
///
/// For each CSV file in data/kaggle/spy/, groups rows by expiry (DTE),
/// picks a representative set of slices, and plots IV vs log-moneyness.
/// Uses P_IV when log_moneyness < 0, C_IV when log_moneyness >= 0.
use plotters::prelude::*;
use std::collections::BTreeMap;
use std::fs;

struct DataPoint {
    log_m: f64,
    iv: f64,
}

fn parse_csv(path: &str) -> BTreeMap<String, Vec<DataPoint>> {
    let content = fs::read_to_string(path).expect("read csv");
    let mut lines = content.lines();
    let header: Vec<&str> = lines.next().expect("header").split(',').collect();

    let col = |name: &str| -> usize {
        header
            .iter()
            .position(|&h| h.trim() == name)
            .unwrap_or_else(|| panic!("missing column: {}", name))
    };
    let i_dte = col("DTE");
    let i_expire = col("EXPIRE_DATE");
    let i_c_iv = col("C_IV");
    let i_p_iv = col("P_IV");
    let i_log_m = col("LOG_MONEYNESS");

    let mut slices: BTreeMap<String, Vec<DataPoint>> = BTreeMap::new();

    for line in lines {
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() <= i_log_m {
            continue;
        }
        let dte: f64 = match fields[i_dte].trim().parse() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let log_m: f64 = match fields[i_log_m].trim().parse() {
            Ok(v) => v,
            Err(_) => continue,
        };

        // Use P_IV for log_m < -0.1, mean(P_IV,C_IV) for [-0.1,0.1], C_IV for > 0.1
        let iv: f64 = if log_m < -0.1 {
            match fields[i_p_iv].trim().parse::<f64>() {
                Ok(v) if v > 0.0 => v,
                _ => continue,
            }
        } else if log_m > 0.1 {
            match fields[i_c_iv].trim().parse::<f64>() {
                Ok(v) if v > 0.0 => v,
                _ => continue,
            }
        } else {
            let p: f64 = match fields[i_p_iv].trim().parse() {
                Ok(v) if v > 0.0 => v,
                _ => continue,
            };
            let c: f64 = match fields[i_c_iv].trim().parse() {
                Ok(v) if v > 0.0 => v,
                _ => continue,
            };
            (p + c) / 2.0
        };

        // Skip 0-DTE
        if dte < 1.0 {
            continue;
        }

        let label = format!("DTE={} ({})", dte as u32, fields[i_expire].trim());
        slices.entry(label).or_default().push(DataPoint { log_m, iv });
    }

    slices
}

/// Pick a spread of slices: short, medium, long dated.
fn select_slices(all: &BTreeMap<String, Vec<DataPoint>>) -> Vec<(&str, &Vec<DataPoint>)> {
    let keys: Vec<&String> = all.keys().collect();
    if keys.len() <= 8 {
        return all.iter().map(|(k, v)| (k.as_str(), v)).collect();
    }
    // Pick ~8 evenly spaced
    let n = keys.len();
    let step = n as f64 / 8.0;
    let mut selected = Vec::new();
    for i in 0..8 {
        let idx = (i as f64 * step) as usize;
        let key = keys[idx];
        selected.push((key.as_str(), all.get(key).unwrap()));
    }
    selected
}

const PALETTE: &[RGBColor] = &[
    RGBColor(31, 119, 180),
    RGBColor(255, 127, 14),
    RGBColor(44, 160, 44),
    RGBColor(214, 39, 40),
    RGBColor(148, 103, 189),
    RGBColor(140, 86, 75),
    RGBColor(227, 119, 194),
    RGBColor(127, 127, 127),
    RGBColor(188, 189, 34),
    RGBColor(23, 190, 207),
];

fn plot_slices(
    slices: &[(&str, &Vec<DataPoint>)],
    path: &str,
    title: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = SVGBackend::new(path, (900, 560)).into_drawing_area();
    root.fill(&WHITE)?;

    // Compute axis ranges from all data
    let mut x_min = f64::INFINITY;
    let mut x_max = f64::NEG_INFINITY;
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;
    for (_, pts) in slices {
        for p in *pts {
            x_min = x_min.min(p.log_m);
            x_max = x_max.max(p.log_m);
            y_min = y_min.min(p.iv);
            y_max = y_max.max(p.iv);
        }
    }
    let x_pad = (x_max - x_min) * 0.05;
    let y_pad = (y_max - y_min) * 0.08;

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 16))
        .margin(10)
        .x_label_area_size(35)
        .y_label_area_size(55)
        .right_y_label_area_size(10)
        .build_cartesian_2d(
            (x_min - x_pad)..(x_max + x_pad),
            (y_min - y_pad)..(y_max + y_pad),
        )?;

    chart
        .configure_mesh()
        .x_desc("log-moneyness  ln(K/F)")
        .y_desc("implied volatility")
        .draw()?;

    for (i, (label, pts)) in slices.iter().enumerate() {
        let color = PALETTE[i % PALETTE.len()];
        chart
            .draw_series(
                pts.iter()
                    .map(|p| Circle::new((p.log_m, p.iv), 2, color.filled())),
            )?
            .label(*label)
            .legend(move |(x, y)| Circle::new((x + 10, y), 3, color.filled()));
    }

    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::UpperRight)
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .label_font(("sans-serif", 11))
        .draw()?;

    root.present()?;
    Ok(())
}

fn main() {
    let data_dir = "data/kaggle/spy";
    let plot_dir = "documents/data-plots";
    fs::create_dir_all(plot_dir).expect("create plot dir");

    let mut csv_files: Vec<_> = fs::read_dir(data_dir)
        .expect("read data dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "csv"))
        .map(|e| e.path())
        .collect();
    csv_files.sort();

    for csv_path in &csv_files {
        let fname = csv_path.file_stem().unwrap().to_str().unwrap();
        println!("Processing {}...", csv_path.display());

        let all_slices = parse_csv(csv_path.to_str().unwrap());
        println!("  Found {} expiry slices", all_slices.len());

        let selected = select_slices(&all_slices);
        let out_path = format!("{}/smile_{}.svg", plot_dir, fname);
        let title = format!("SPY IV Smile — {}", fname);

        match plot_slices(&selected, &out_path, &title) {
            Ok(()) => println!("  Wrote {}", out_path),
            Err(e) => eprintln!("  Plot error: {}", e),
        }
    }
}
