use crate::{AnsiColor, AnsiIntensity, AnsiStyle, UnderlineStyle};
use vte::Params;

pub(crate) fn apply_sgr(params: &Params, style: &mut AnsiStyle) {
    let params: Vec<Vec<u16>> = params
        .iter()
        .map(|param| {
            if param.is_empty() {
                vec![0]
            } else {
                param.to_vec()
            }
        })
        .collect();

    if params.is_empty() {
        style.reset();
        return;
    }

    let mut i = 0;
    while i < params.len() {
        let param = &params[i];
        let code = param.first().copied().unwrap_or(0);

        if param.len() > 1 {
            match code {
                4 => {
                    style.underline = underline_from_subparam(param.get(1).copied().unwrap_or(1));
                }
                38 | 48 | 58 => {
                    if let Some(color) = extended_color_from_subparams(&param[1..]) {
                        apply_extended_color(style, code, color);
                    }
                }
                _ => apply_simple_sgr(style, code),
            }
            i += 1;
            continue;
        }

        match code {
            38 | 48 | 58 => {
                let consumed = apply_semicolon_extended_color(&params, i, style, code);
                i += consumed.max(1);
            }
            _ => {
                apply_simple_sgr(style, code);
                i += 1;
            }
        }
    }
}

fn apply_simple_sgr(style: &mut AnsiStyle, code: u16) {
    match code {
        0 => style.reset(),
        1 => style.intensity = AnsiIntensity::Bold,
        2 => style.intensity = AnsiIntensity::Faint,
        3 => style.italic = true,
        4 => style.underline = UnderlineStyle::Single,
        7 => style.reverse = true,
        8 => style.hidden = true,
        9 => style.strikethrough = true,
        21 => style.underline = UnderlineStyle::Double,
        22 => style.intensity = AnsiIntensity::Normal,
        23 => style.italic = false,
        24 => style.underline = UnderlineStyle::None,
        27 => style.reverse = false,
        28 => style.hidden = false,
        29 => style.strikethrough = false,
        30..=37 => style.foreground = AnsiColor::Indexed((code - 30) as u8),
        40..=47 => style.background = AnsiColor::Indexed((code - 40) as u8),
        90..=97 => style.foreground = AnsiColor::Indexed((code - 90 + 8) as u8),
        100..=107 => style.background = AnsiColor::Indexed((code - 100 + 8) as u8),
        39 => style.foreground = AnsiColor::Default,
        49 => style.background = AnsiColor::Default,
        59 => style.underline_color = None,
        _ => {}
    }
}

fn apply_semicolon_extended_color(
    params: &[Vec<u16>],
    index: usize,
    style: &mut AnsiStyle,
    target: u16,
) -> usize {
    let Some(mode) = params
        .get(index + 1)
        .and_then(|param| param.first())
        .copied()
    else {
        return 1;
    };

    match mode {
        5 => {
            if let Some(color_code) = params
                .get(index + 2)
                .and_then(|param| param.first())
                .and_then(|value| u8::try_from(*value).ok())
            {
                apply_extended_color(style, target, AnsiColor::Indexed(color_code));
                3
            } else {
                1
            }
        }
        2 => {
            let rgb = [
                params
                    .get(index + 2)
                    .and_then(|param| param.first())
                    .copied(),
                params
                    .get(index + 3)
                    .and_then(|param| param.first())
                    .copied(),
                params
                    .get(index + 4)
                    .and_then(|param| param.first())
                    .copied(),
            ];

            if let [Some(r), Some(g), Some(b)] = rgb
                && let Some(color) = rgb_color(r, g, b)
            {
                apply_extended_color(style, target, color);
                return 5;
            }

            1
        }
        _ => 1,
    }
}

fn extended_color_from_subparams(subparams: &[u16]) -> Option<AnsiColor> {
    let mode = subparams.first().copied()?;

    match mode {
        5 => subparams
            .get(1)
            .and_then(|value| u8::try_from(*value).ok())
            .map(AnsiColor::Indexed),
        2 => {
            if subparams.len() < 4 {
                return None;
            }

            let rgb_start = subparams.len() - 3;
            rgb_color(
                subparams[rgb_start],
                subparams[rgb_start + 1],
                subparams[rgb_start + 2],
            )
        }
        _ => None,
    }
}

fn rgb_color(r: u16, g: u16, b: u16) -> Option<AnsiColor> {
    Some(AnsiColor::Rgb(
        u8::try_from(r).ok()?,
        u8::try_from(g).ok()?,
        u8::try_from(b).ok()?,
    ))
}

fn apply_extended_color(style: &mut AnsiStyle, target: u16, color: AnsiColor) {
    match target {
        38 => style.foreground = color,
        48 => style.background = color,
        58 => style.underline_color = Some(color),
        _ => {}
    }
}

fn underline_from_subparam(value: u16) -> UnderlineStyle {
    match value {
        0 => UnderlineStyle::None,
        1 => UnderlineStyle::Single,
        2 => UnderlineStyle::Double,
        3 => UnderlineStyle::Curly,
        4 => UnderlineStyle::Dotted,
        5 => UnderlineStyle::Dashed,
        _ => UnderlineStyle::Single,
    }
}
