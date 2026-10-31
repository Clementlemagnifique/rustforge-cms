//! # Utilitaires de formatage de date et heure compatibles WordPress (WP-73)
//!
//! Permet de formater un horodatage avec la syntaxe de motifs standard de WordPress
//! (`j F Y`, `Y-m-d`, `H:i`, `g:i a`, etc.) avec prise en charge du multilingue (français, anglais, espagnol).

use chrono::{DateTime, Datelike, Timelike, Utc};

/// Formate un instant `DateTime<Utc>` selon un format compatible WordPress (`WP-73`).
///
/// Gère les jetons usuels :
/// - Année : `Y` (2026), `y` (26)
/// - Mois : `F` (octobre), `M` (oct.), `m` (10), `n` (10)
/// - Jour : `d` (29), `j` (29), `l` (jeudi), `D` (jeu.)
/// - Heure : `H` (14), `G` (14), `h` (02), `g` (2)
/// - Minute/Seconde : `i` (30), `s` (45)
/// - Méridien : `a` (pm), `A` (PM)
/// - Échappement : `\x` produit le caractère `x` littéralement.
pub fn format_wp_datetime(dt: &DateTime<Utc>, format_str: &str, locale: &str) -> String {
    let year = dt.year();
    let month = dt.month();
    let day = dt.day();
    let weekday = dt.weekday().num_days_from_sunday(); // 0 = Dimanche .. 6 = Samedi
    let hour = dt.hour();
    let minute = dt.minute();
    let second = dt.second();

    let is_pm = hour >= 12;
    let hour12 = match hour % 12 {
        0 => 12,
        h => h,
    };

    let mut out = String::with_capacity(format_str.len() * 2);
    let mut chars = format_str.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(escaped) = chars.next() {
                out.push(escaped);
            }
            continue;
        }

        match ch {
            'Y' => out.push_str(&format!("{year:04}")),
            'y' => out.push_str(&format!("{:02}", year % 100)),
            'm' => out.push_str(&format!("{month:02}")),
            'n' => out.push_str(&format!("{month}")),
            'F' => out.push_str(month_name(month, locale)),
            'M' => out.push_str(month_short(month, locale)),
            'd' => out.push_str(&format!("{day:02}")),
            'j' => out.push_str(&format!("{day}")),
            'l' => out.push_str(weekday_name(weekday, locale)),
            'D' => out.push_str(weekday_short(weekday, locale)),
            'H' => out.push_str(&format!("{hour:02}")),
            'G' => out.push_str(&format!("{hour}")),
            'h' => out.push_str(&format!("{hour12:02}")),
            'g' => out.push_str(&format!("{hour12}")),
            'i' => out.push_str(&format!("{minute:02}")),
            's' => out.push_str(&format!("{second:02}")),
            'a' => out.push_str(if is_pm { "pm" } else { "am" }),
            'A' => out.push_str(if is_pm { "PM" } else { "AM" }),
            other => out.push(other),
        }
    }

    out
}

fn month_name(month: u32, locale: &str) -> &'static str {
    match locale {
        "en" => match month {
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => "",
        },
        "es" => match month {
            1 => "enero",
            2 => "febrero",
            3 => "marzo",
            4 => "abril",
            5 => "mayo",
            6 => "junio",
            7 => "julio",
            8 => "agosto",
            9 => "septiembre",
            10 => "octubre",
            11 => "noviembre",
            12 => "diciembre",
            _ => "",
        },
        _ => match month {
            1 => "janvier",
            2 => "février",
            3 => "mars",
            4 => "avril",
            5 => "mai",
            6 => "juin",
            7 => "juillet",
            8 => "août",
            9 => "septembre",
            10 => "octobre",
            11 => "novembre",
            12 => "décembre",
            _ => "",
        },
    }
}

fn month_short(month: u32, locale: &str) -> &'static str {
    match locale {
        "en" => match month {
            1 => "Jan",
            2 => "Feb",
            3 => "Mar",
            4 => "Apr",
            5 => "May",
            6 => "Jun",
            7 => "Jul",
            8 => "Aug",
            9 => "Sep",
            10 => "Oct",
            11 => "Nov",
            12 => "Dec",
            _ => "",
        },
        "es" => match month {
            1 => "ene.",
            2 => "feb.",
            3 => "mar.",
            4 => "abr.",
            5 => "may.",
            6 => "jun.",
            7 => "jul.",
            8 => "ago.",
            9 => "sep.",
            10 => "oct.",
            11 => "nov.",
            12 => "dic.",
            _ => "",
        },
        _ => match month {
            1 => "janv.",
            2 => "févr.",
            3 => "mars",
            4 => "avr.",
            5 => "mai",
            6 => "juin",
            7 => "juil.",
            8 => "août",
            9 => "sept.",
            10 => "oct.",
            11 => "nov.",
            12 => "déc.",
            _ => "",
        },
    }
}

fn weekday_name(weekday: u32, locale: &str) -> &'static str {
    match locale {
        "en" => match weekday {
            0 => "Sunday",
            1 => "Monday",
            2 => "Tuesday",
            3 => "Wednesday",
            4 => "Thursday",
            5 => "Friday",
            6 => "Saturday",
            _ => "",
        },
        "es" => match weekday {
            0 => "domingo",
            1 => "lunes",
            2 => "martes",
            3 => "miércoles",
            4 => "jueves",
            5 => "viernes",
            6 => "sábado",
            _ => "",
        },
        _ => match weekday {
            0 => "dimanche",
            1 => "lundi",
            2 => "mardi",
            3 => "mercredi",
            4 => "jeudi",
            5 => "vendredi",
            6 => "samedi",
            _ => "",
        },
    }
}

fn weekday_short(weekday: u32, locale: &str) -> &'static str {
    match locale {
        "en" => match weekday {
            0 => "Sun",
            1 => "Mon",
            2 => "Tue",
            3 => "Wed",
            4 => "Thu",
            5 => "Fri",
            6 => "Sat",
            _ => "",
        },
        "es" => match weekday {
            0 => "dom.",
            1 => "lun.",
            2 => "mar.",
            3 => "mié.",
            4 => "jue.",
            5 => "vie.",
            6 => "sáb.",
            _ => "",
        },
        _ => match weekday {
            0 => "dim.",
            1 => "lun.",
            2 => "mar.",
            3 => "mer.",
            4 => "jeu.",
            5 => "ven.",
            6 => "sam.",
            _ => "",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_format_wp_datetime_french_and_english() {
        // 2026-10-29 14:35:08 UTC (Jeudi)
        let dt = Utc.with_ymd_and_hms(2026, 10, 29, 14, 35, 8).unwrap();

        // Standard WordPress formats
        assert_eq!(format_wp_datetime(&dt, "j F Y", "fr"), "29 octobre 2026");
        assert_eq!(format_wp_datetime(&dt, "j F Y", "en"), "29 October 2026");
        assert_eq!(format_wp_datetime(&dt, "Y-m-d", "fr"), "2026-10-29");
        assert_eq!(format_wp_datetime(&dt, "d/m/Y", "fr"), "29/10/2026");
        assert_eq!(
            format_wp_datetime(&dt, "l j F Y", "fr"),
            "jeudi 29 octobre 2026"
        );

        // Standard time formats
        assert_eq!(format_wp_datetime(&dt, "H:i", "fr"), "14:35");
        assert_eq!(format_wp_datetime(&dt, "g:i a", "fr"), "2:35 pm");
        assert_eq!(format_wp_datetime(&dt, "g:i A", "fr"), "2:35 PM");
        assert_eq!(
            format_wp_datetime(&dt, r"H \h i \m\i\n", "fr"),
            "14 h 35 min"
        );
    }
}
