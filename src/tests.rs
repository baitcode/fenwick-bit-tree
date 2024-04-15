/// WIP

#[cfg(test)]
mod more_complex_tests {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use rand::{seq::IteratorRandom, Rng};

    enum FlightStatus {
        LANDED,
        FUELUP,
    }

    struct LogRecord {
        timestamp: u64,
        flight: String,
        fuel_delta: i32,
        status: FlightStatus,
    }

    fn generate_flights(start: SystemTime, flight: String, segment_count: usize) -> Vec<LogRecord> {
        let mut res = vec![];

        let mut fuel = 0;

        let mut current_timestamp = start.duration_since(UNIX_EPOCH).unwrap().as_secs();

        let mut rng = rand::thread_rng();
        for _i in 0..segment_count {
            let next_flight_fuel = (rng.gen::<f32>() * 10.0) as i32 * 100;
            let some_additional = (rng.gen::<f32>() * 10.0) as i32 * 100;
            let time_delta = Duration::from_secs(next_flight_fuel as u64 * 72);

            if next_flight_fuel < fuel {
                let fuel_delta = next_flight_fuel + some_additional;

                res.push(LogRecord {
                    timestamp: current_timestamp,
                    flight: flight.clone(),
                    status: FlightStatus::FUELUP,
                    fuel_delta,
                });
                fuel += fuel_delta
            }

            current_timestamp += time_delta.as_secs();

            res.push(LogRecord {
                timestamp: current_timestamp,
                flight: flight.clone(),
                status: FlightStatus::LANDED,
                fuel_delta: -next_flight_fuel,
            });
            fuel -= next_flight_fuel
        }

        return res;
    }

    fn generate_flight_log(flight_count: usize, segment_count: usize) -> Vec<LogRecord> {
        let mut res = vec![];
        let letters = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

        let mut rng = rand::thread_rng();
        let start = SystemTime::now();

        for i in 0..flight_count {
            let num = (rng.gen::<f32>() * 100.0) as i32;
            let flight_name = format!("{:?}{:?}", letters.chars().choose(&mut rng), num);
            res.append(&mut generate_flights(start, flight_name, segment_count));
        }

        return res;
    }

    #[test]
    fn complex_case() {}
}
