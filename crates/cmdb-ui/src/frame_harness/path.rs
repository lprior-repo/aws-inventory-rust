use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PathState {
    pub zoom: f64,
    pub pan: f64,
    pub clicks: u8,
}

pub fn at(elapsed: Duration, duration: Duration) -> PathState {
    let progress = (elapsed.as_secs_f64() / duration.as_secs_f64().max(1.0)).clamp(0.0, 1.0);
    let zoom = if progress <= 0.25 {
        progress * 4.0
    } else {
        (2.0 - progress * 4.0).max(0.0)
    };
    PathState {
        zoom,
        pan: ((progress - 0.5) * 2.0).clamp(0.0, 1.0),
        clicks: [0.2, 0.6, 0.9].iter().fold(0_u8, |count, &threshold| {
            count.saturating_add(u8::from(progress >= threshold))
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_elapsed_time_when_sampling_then_path_is_repeatable() {
        let duration = Duration::from_secs(20);
        for millis in 0..=20_000 {
            let elapsed = Duration::from_millis(millis);
            assert_eq!(at(elapsed, duration), at(elapsed, duration));
        }
    }

    #[test]
    fn given_whole_view_when_path_advances_then_zoom_returns_and_pan_crosses() {
        let duration = Duration::from_secs(20);
        assert_eq!(
            at(Duration::ZERO, duration),
            PathState {
                zoom: 0.0,
                pan: 0.0,
                clicks: 0
            }
        );
        assert_eq!(at(Duration::from_secs(5), duration).zoom, 1.0);
        assert_eq!(at(Duration::from_secs(10), duration).zoom, 0.0);
        assert_eq!(
            at(duration, duration),
            PathState {
                zoom: 0.0,
                pan: 1.0,
                clicks: 3
            }
        );
        assert_eq!(
            at(Duration::from_secs(40), duration),
            at(duration, duration)
        );
    }

    #[test]
    fn given_skipped_updates_when_resuming_then_all_three_clicks_remain_due() {
        let duration = Duration::from_secs(10);
        assert_eq!(at(Duration::from_millis(1_999), duration).clicks, 0);
        assert_eq!(at(Duration::from_secs(2), duration).clicks, 1);
        assert_eq!(at(Duration::from_secs(6), duration).clicks, 2);
        assert_eq!(at(Duration::from_secs(9), duration).clicks, 3);
    }
}
