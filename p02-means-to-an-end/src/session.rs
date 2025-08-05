pub struct Session {
    data: Vec<(i32, i32)>, //(timestamp, price)
}

impl Session {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn insert(&mut self, timestamp: i32, price: i32) {
        self.data.push((timestamp, price));
    }

    pub fn query(&self, mintime: i32, maxtime: i32) -> i32 {
        let mut sum = 0i64;
        let mut count = 0;
        for &(ts, price) in &self.data {
            if ts >= mintime && ts <= maxtime {
                sum += price as i64;
                count += 1;
            }
        }

        if count == 0 { 0 } else { (sum / count) as i32 }
    }
}
