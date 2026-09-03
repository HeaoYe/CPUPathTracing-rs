#[derive(Default, Clone, Copy)]
struct Item {
    q: f64,
    p: f64,
    alias: usize,
}

#[derive(Default)]
pub struct AliasTable {
    pmfs: Vec<f32>,
    items: Vec<Item>,
}

pub struct AliasTableSample {
    pub index: usize,
    pub pmf: f32,
}

impl AliasTable {
    pub fn new(values: &[f32]) -> Self {
        let mut sum = 0.0;
        for &value in values {
            sum += value as f64;
        }

        let mut pmfs = vec![0.0; values.len()];
        let mut items = vec![Item::default(); values.len()];

        let mut less = Vec::new();
        let mut greater = Vec::new();

        for (i, &value) in values.iter().enumerate() {
            pmfs[i] = (value as f64 / sum) as f32;
            items[i].q = 1.0;
            items[i].p = value as f64 * (items.len() as f64 / sum);

            if items[i].p < 1.0 {
                less.push(i);
            } else if items[i].p > 1.0 {
                greater.push(i);
            }
        }

        while !less.is_empty() && !greater.is_empty() {
            let item_less = &mut items[less.pop().unwrap()];
            let item_greater_index = greater.pop().unwrap();

            let p = item_less.p;
            item_less.q = p;
            item_less.alias = item_greater_index;

            let item_greater = &mut items[item_greater_index];
            item_greater.p -= 1.0 - p;

            if item_greater.p < 1.0 {
                less.push(item_greater_index);
            } else if item_greater.p > 1.0 {
                greater.push(item_greater_index);
            }
        }

        Self { pmfs, items }
    }

    pub fn sample(&self, u: f32) -> Option<AliasTableSample> {
        if self.items.is_empty() {
            return None;
        }

        let idx = ((u * self.items.len() as f32).floor() as usize).clamp(0, self.items.len() - 1);
        let u = (u * self.items.len() as f32 - idx as f32).clamp(0.0, 1.0);
        let item = self.items[idx];
        if item.q == 1.0 || u < item.q as f32 {
            Some(AliasTableSample {
                index: idx,
                pmf: self.pmfs[idx],
            })
        } else {
            Some(AliasTableSample {
                index: item.alias,
                pmf: self.pmfs[item.alias],
            })
        }
    }

    pub fn pmf(&self, index: usize) -> f32 {
        self.pmfs[index]
    }
}
